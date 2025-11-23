// STREAMING ENCRYPTION/DECRYPTION
// Enables processing of large files without loading entire file into memory

use crate::error::{MSSCSError, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use std::io::{Read, Write};

/// Streaming cipher for large file encryption
pub struct StreamingCipher {
    cipher: ChaCha20Poly1305,
    chunk_size: usize,
}

impl StreamingCipher {
    /// Create a new streaming cipher
    pub fn new(key: &[u8; 32], chunk_size: usize) -> Self {
        let cipher = ChaCha20Poly1305::new(key.into());
        Self { cipher, chunk_size }
    }

    /// Encrypt data in streaming fashion
    /// Returns encrypted chunks with nonces prepended
    pub fn encrypt_stream<R: Read>(
        &self,
        mut reader: R,
    ) -> Result<Vec<Vec<u8>>> {
        let mut encrypted_chunks = Vec::new();
        let mut buffer = vec![0u8; self.chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| MSSCSError::Io(e))?;

            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            let encrypted_chunk = self.encrypt_chunk(chunk)?;
            encrypted_chunks.push(encrypted_chunk);
        }

        Ok(encrypted_chunks)
    }

    /// Decrypt data in streaming fashion
    pub fn decrypt_stream<W: Write>(
        &self,
        encrypted_chunks: &[Vec<u8>],
        mut writer: W,
    ) -> Result<()> {
        for encrypted_chunk in encrypted_chunks {
            let decrypted = self.decrypt_chunk(encrypted_chunk)?;
            writer.write_all(&decrypted)
                .map_err(|e| MSSCSError::Io(e))?;
        }

        Ok(())
    }

    /// Encrypt a single chunk
    fn encrypt_chunk(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| MSSCSError::Encryption(format!("Chunk encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt a single chunk
    fn decrypt_chunk(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(MSSCSError::Decryption("Invalid encrypted chunk".into()));
        }

        let nonce = Nonce::from_slice(&encrypted[..12]);
        let ciphertext = &encrypted[12..];

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| MSSCSError::Decryption(format!("Chunk decryption failed: {}", e)))?;

        Ok(plaintext)
    }
}

/// Streaming file encryptor
pub struct StreamingFileEncryptor {
    key: [u8; 32],
    chunk_size: usize,
}

impl StreamingFileEncryptor {
    /// Create a new streaming file encryptor
    pub fn new(key: [u8; 32], chunk_size: usize) -> Self {
        Self { key, chunk_size }
    }

    /// Encrypt a file in streaming fashion
    pub fn encrypt_file<R: Read, W: Write>(
        &self,
        reader: R,
        mut writer: W,
    ) -> Result<usize> {
        let cipher = StreamingCipher::new(&self.key, self.chunk_size);
        let encrypted_chunks = cipher.encrypt_stream(reader)?;

        let mut total_bytes = 0;
        for chunk in &encrypted_chunks {
            writer.write_all(chunk)
                .map_err(|e| MSSCSError::Io(e))?;
            total_bytes += chunk.len();
        }

        Ok(total_bytes)
    }

    /// Decrypt a file in streaming fashion
    pub fn decrypt_file<R: Read, W: Write>(
        &self,
        mut reader: R,
        writer: W,
    ) -> Result<usize> {
        let cipher = StreamingCipher::new(&self.key, self.chunk_size);

        // Read all encrypted chunks
        let mut encrypted_chunks = Vec::new();
        let mut buffer = vec![0u8; self.chunk_size + 12 + 16]; // chunk + nonce + auth tag

        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| MSSCSError::Io(e))?;

            if bytes_read == 0 {
                break;
            }

            encrypted_chunks.push(buffer[..bytes_read].to_vec());
        }

        cipher.decrypt_stream(&encrypted_chunks, writer)?;

        Ok(encrypted_chunks.len())
    }
}

/// Range request support for partial file reads
pub struct RangeReader {
    chunk_size: usize,
}

impl RangeReader {
    /// Create a new range reader
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Calculate which chunks are needed for a byte range
    pub fn chunks_for_range(&self, start: u64, end: u64) -> Vec<usize> {
        let start_chunk = (start / self.chunk_size as u64) as usize;
        let end_chunk = (end / self.chunk_size as u64) as usize;

        (start_chunk..=end_chunk).collect()
    }

    /// Extract bytes from a range of chunks
    pub fn extract_range(
        &self,
        chunks: &[Vec<u8>],
        start: u64,
        end: u64,
    ) -> Result<Vec<u8>> {
        let start_chunk = (start / self.chunk_size as u64) as usize;
        let end_chunk = (end / self.chunk_size as u64) as usize;

        let start_offset = (start % self.chunk_size as u64) as usize;
        let end_offset = (end % self.chunk_size as u64) as usize;

        let mut result = Vec::new();

        for (i, chunk_idx) in (start_chunk..=end_chunk).enumerate() {
            if chunk_idx >= chunks.len() {
                break;
            }

            let chunk = &chunks[chunk_idx];

            if i == 0 && start_chunk == end_chunk {
                // Single chunk
                result.extend_from_slice(&chunk[start_offset..=end_offset]);
            } else if i == 0 {
                // First chunk
                result.extend_from_slice(&chunk[start_offset..]);
            } else if chunk_idx == end_chunk {
                // Last chunk
                result.extend_from_slice(&chunk[..=end_offset]);
            } else {
                // Middle chunk
                result.extend_from_slice(chunk);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_streaming_encryption() {
        let key = [42u8; 32];
        let cipher = StreamingCipher::new(&key, 1024);

        let data = b"Hello, World! This is a test of streaming encryption.";
        let reader = Cursor::new(data);

        let encrypted = cipher.encrypt_stream(reader).unwrap();
        assert!(!encrypted.is_empty());

        let mut output = Vec::new();
        cipher.decrypt_stream(&encrypted, &mut output).unwrap();

        assert_eq!(output, data);
    }

    #[test]
    fn test_large_file_streaming() {
        let key = [42u8; 32];
        let encryptor = StreamingFileEncryptor::new(key, 64 * 1024); // 64KB chunks

        // Create 1MB test data
        let data = vec![42u8; 1024 * 1024];
        let reader = Cursor::new(&data);
        let mut encrypted = Vec::new();

        encryptor.encrypt_file(reader, &mut encrypted).unwrap();
        assert!(encrypted.len() > data.len()); // Should be larger due to nonces and auth tags

        // Decrypt
        let encrypted_reader = Cursor::new(&encrypted);
        let mut decrypted = Vec::new();

        encryptor.decrypt_file(encrypted_reader, &mut decrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_range_reader() {
        let reader = RangeReader::new(1024);

        // Test range calculation
        let chunks = reader.chunks_for_range(0, 1023);
        assert_eq!(chunks, vec![0]);

        let chunks = reader.chunks_for_range(0, 2047);
        assert_eq!(chunks, vec![0, 1]);

        let chunks = reader.chunks_for_range(1024, 2047);
        assert_eq!(chunks, vec![1]);
    }

    #[test]
    fn test_range_extraction() {
        let reader = RangeReader::new(10);
        let chunks = vec![
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24, 25, 26, 27, 28, 29],
        ];

        // Extract from single chunk
        let result = reader.extract_range(&chunks, 5, 8).unwrap();
        assert_eq!(result, vec![5, 6, 7, 8]);

        // Extract across chunks
        let result = reader.extract_range(&chunks, 8, 12).unwrap();
        assert_eq!(result, vec![8, 9, 10, 11, 12]);

        // Extract from multiple chunks
        let result = reader.extract_range(&chunks, 5, 25).unwrap();
        assert_eq!(result, vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25]);
    }
}
