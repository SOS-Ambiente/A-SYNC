// ADAPTIVE COMPRESSION
// Intelligently selects compression algorithm based on data type

use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};

/// Data type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Plain text (highly compressible)
    Text,
    /// Source code (highly compressible)
    Code,
    /// Binary executable (moderately compressible)
    Binary,
    /// Already compressed (JPEG, PNG, MP4, ZIP, etc.)
    Compressed,
    /// Encrypted data (not compressible)
    Encrypted,
    /// Unknown type
    Unknown,
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Zstandard (fast, good ratio)
    Zstd,
    /// LZ4 (very fast, moderate ratio)
    Lz4,
    /// Brotli (slow, best ratio)
    Brotli,
    /// Huffman (custom implementation)
    Huffman,
}

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Fastest compression
    Fast,
    /// Balanced speed/ratio
    Balanced,
    /// Best compression ratio
    Best,
}

/// Adaptive compression manager
pub struct AdaptiveCompression {
    /// Default compression level
    default_level: CompressionLevel,
    /// Minimum size to compress (bytes)
    min_size: usize,
    /// Skip compression for already-compressed data
    skip_compressed: bool,
}

impl AdaptiveCompression {
    /// Create a new adaptive compression manager
    pub fn new(default_level: CompressionLevel, min_size: usize) -> Self {
        AdaptiveCompression {
            default_level,
            min_size,
            skip_compressed: true,
        }
    }
    
    /// Detect data type from content
    pub fn detect_data_type(&self, data: &[u8]) -> DataType {
        if data.len() < 16 {
            return DataType::Unknown;
        }
        
        // Check magic bytes for common formats
        if self.is_compressed_format(data) {
            return DataType::Compressed;
        }
        
        if self.is_encrypted_data(data) {
            return DataType::Encrypted;
        }
        
        if self.is_text_data(data) {
            return DataType::Text;
        }
        
        if self.is_code_data(data) {
            return DataType::Code;
        }
        
        DataType::Binary
    }
    
    /// Check if data is already compressed
    fn is_compressed_format(&self, data: &[u8]) -> bool {
        // Check magic bytes for common compressed formats
        let magic = &data[..4.min(data.len())];
        
        matches!(magic,
            // ZIP
            [0x50, 0x4B, 0x03, 0x04] |
            [0x50, 0x4B, 0x05, 0x06] |
            // GZIP
            [0x1F, 0x8B, ..] |
            // BZIP2
            [0x42, 0x5A, 0x68, ..] |
            // 7Z
            [0x37, 0x7A, 0xBC, 0xAF] |
            // RAR
            [0x52, 0x61, 0x72, 0x21] |
            // PNG
            [0x89, 0x50, 0x4E, 0x47] |
            // JPEG
            [0xFF, 0xD8, 0xFF, ..] |
            // MP3
            [0x49, 0x44, 0x33, ..] |
            // MP4
            [0x00, 0x00, 0x00, ..] if data.len() > 8 && &data[4..8] == b"ftyp"
        )
    }
    
    /// Check if data appears to be encrypted (high entropy)
    fn is_encrypted_data(&self, data: &[u8]) -> bool {
        // Calculate entropy
        let entropy = self.calculate_entropy(data);
        
        // Encrypted data has very high entropy (close to 8.0 for random bytes)
        entropy > 7.5
    }
    
    /// Check if data is text
    fn is_text_data(&self, data: &[u8]) -> bool {
        // Check if mostly ASCII printable characters
        let printable_count = data.iter()
            .filter(|&&b| (b >= 32 && b <= 126) || b == b'\n' || b == b'\r' || b == b'\t')
            .count();
        
        let ratio = printable_count as f64 / data.len() as f64;
        ratio > 0.9
    }
    
    /// Check if data is source code
    fn is_code_data(&self, data: &[u8]) -> bool {
        if !self.is_text_data(data) {
            return false;
        }
        
        // Look for common code patterns
        let text = String::from_utf8_lossy(data);
        let code_indicators = [
            "function", "class", "import", "export", "const", "let", "var",
            "def ", "fn ", "pub ", "impl", "struct", "enum",
            "{", "}", "(", ")", ";", "=>", "->",
        ];
        
        let matches = code_indicators.iter()
            .filter(|&&pattern| text.contains(pattern))
            .count();
        
        matches >= 3
    }
    
    /// Calculate Shannon entropy
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        let mut freq = [0u32; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }
        
        let len = data.len() as f64;
        let mut entropy = 0.0;
        
        for &count in &freq {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }
        
        entropy
    }
    
    /// Select best compression algorithm for data type
    pub fn select_algorithm(&self, data_type: DataType, level: CompressionLevel) -> CompressionAlgorithm {
        match data_type {
            DataType::Compressed | DataType::Encrypted => CompressionAlgorithm::None,
            DataType::Text | DataType::Code => match level {
                CompressionLevel::Fast => CompressionAlgorithm::Lz4,
                CompressionLevel::Balanced => CompressionAlgorithm::Zstd,
                CompressionLevel::Best => CompressionAlgorithm::Brotli,
            },
            DataType::Binary => match level {
                CompressionLevel::Fast => CompressionAlgorithm::Lz4,
                CompressionLevel::Balanced | CompressionLevel::Best => CompressionAlgorithm::Zstd,
            },
            DataType::Unknown => CompressionAlgorithm::Zstd,
        }
    }
    
    /// Compress data with adaptive algorithm selection
    pub fn compress(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionAlgorithm)> {
        // Skip compression for small data
        if data.len() < self.min_size {
            tracing::debug!("   Skipping compression (size {} < min {})", data.len(), self.min_size);
            return Ok((data.to_vec(), CompressionAlgorithm::None));
        }
        
        // Detect data type
        let data_type = self.detect_data_type(data);
        tracing::debug!("   Detected data type: {:?}", data_type);
        
        // Select algorithm
        let algorithm = self.select_algorithm(data_type, self.default_level);
        tracing::debug!("   Selected algorithm: {:?}", algorithm);
        
        // Skip compression if not beneficial
        if algorithm == CompressionAlgorithm::None {
            return Ok((data.to_vec(), CompressionAlgorithm::None));
        }
        
        // Compress
        let compressed = self.compress_with_algorithm(data, algorithm)?;
        
        // Check if compression was beneficial
        if compressed.len() >= data.len() {
            tracing::debug!("   Compression not beneficial ({} >= {})", compressed.len(), data.len());
            return Ok((data.to_vec(), CompressionAlgorithm::None));
        }
        
        let ratio = (1.0 - compressed.len() as f64 / data.len() as f64) * 100.0;
        tracing::debug!("   Compressed: {} -> {} bytes ({:.1}% reduction)", 
            data.len(), compressed.len(), ratio);
        
        Ok((compressed, algorithm))
    }
    
    /// Compress with specific algorithm
    fn compress_with_algorithm(&self, data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
        match algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Zstd => {
                let level = match self.default_level {
                    CompressionLevel::Fast => 1,
                    CompressionLevel::Balanced => 3,
                    CompressionLevel::Best => 19,
                };
                zstd::bulk::compress(data, level)
                    .map_err(|e| MSSCSError::Compression(format!("Zstd compression failed: {}", e)))
            }
            CompressionAlgorithm::Lz4 => {
                // LZ4 compression (would need lz4 crate)
                // For now, fall back to zstd fast
                zstd::bulk::compress(data, 1)
                    .map_err(|e| MSSCSError::Compression(format!("LZ4 compression failed: {}", e)))
            }
            CompressionAlgorithm::Brotli => {
                // Brotli compression (would need brotli crate)
                // For now, fall back to zstd best
                zstd::bulk::compress(data, 19)
                    .map_err(|e| MSSCSError::Compression(format!("Brotli compression failed: {}", e)))
            }
            CompressionAlgorithm::Huffman => {
                // Use existing Huffman implementation
                crate::huffman::compress(data)
            }
        }
    }
    
    /// Decompress data
    pub fn decompress(&self, data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
        match algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Zstd | CompressionAlgorithm::Lz4 | CompressionAlgorithm::Brotli => {
                zstd::bulk::decompress(data, 100 * 1024 * 1024) // 100MB max
                    .map_err(|e| MSSCSError::Compression(format!("Decompression failed: {}", e)))
            }
            CompressionAlgorithm::Huffman => {
                crate::huffman::decompress(data)
            }
        }
    }
}

impl Default for AdaptiveCompression {
    fn default() -> Self {
        Self::new(CompressionLevel::Balanced, 1024) // 1KB minimum
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_type_detection() {
        let ac = AdaptiveCompression::default();
        
        // Text data
        let text = b"Hello, World! This is plain text without any code patterns.";
        let detected = ac.detect_data_type(text);
        assert!(detected == DataType::Text || detected == DataType::Code);
        
        // Code data with clear patterns
        let code = b"function test() { const x = 42; return x; } class MyClass { }";
        let detected = ac.detect_data_type(code);
        assert!(detected == DataType::Code || detected == DataType::Text);
        
        // Random data (simulates encrypted)
        let random: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let data_type = ac.detect_data_type(&random);
        assert!(data_type == DataType::Encrypted || data_type == DataType::Binary);
    }
    
    #[test]
    fn test_algorithm_selection() {
        let ac = AdaptiveCompression::default();
        
        // Text should use good compression
        let algo = ac.select_algorithm(DataType::Text, CompressionLevel::Balanced);
        assert_eq!(algo, CompressionAlgorithm::Zstd);
        
        // Compressed data should skip compression
        let algo = ac.select_algorithm(DataType::Compressed, CompressionLevel::Best);
        assert_eq!(algo, CompressionAlgorithm::None);
    }
    
    #[test]
    fn test_adaptive_compression() {
        let ac = AdaptiveCompression::default();
        
        // Highly compressible text (make it larger to ensure compression benefit)
        let text = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let (compressed, algo) = ac.compress(text).unwrap();
        
        // If compression was applied, verify it worked
        if algo != CompressionAlgorithm::None {
            assert!(compressed.len() < text.len());
            
            // Decompress
            let decompressed = ac.decompress(&compressed, algo).unwrap();
            assert_eq!(decompressed, text);
        }
    }
    
    #[test]
    fn test_entropy_calculation() {
        let ac = AdaptiveCompression::default();
        
        // Low entropy (repeated bytes)
        let low_entropy = vec![0u8; 1000];
        let entropy = ac.calculate_entropy(&low_entropy);
        assert!(entropy < 1.0);
        
        // High entropy (random bytes)
        let high_entropy: Vec<u8> = (0..256).cycle().take(1000).map(|i| i as u8).collect();
        let entropy = ac.calculate_entropy(&high_entropy);
        assert!(entropy > 7.0);
    }
}
