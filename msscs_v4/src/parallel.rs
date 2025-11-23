// PARALLEL BLOCK PROCESSING
// Concurrent encryption, compression, and network operations for maximum performance

use crate::error::{MSSCSError, Result};
use crate::quantum_block::QuantumDataBlock;
use crate::identity::UnlockedIdentity;
use rayon::prelude::*;
use std::sync::Arc;
use tokio::task;

/// Parallel block processor for high-performance operations
pub struct ParallelBlockProcessor {
    /// Number of worker threads
    pub worker_threads: usize,
    /// Chunk size for parallel processing
    pub chunk_size: usize,
}

impl ParallelBlockProcessor {
    /// Create a new parallel block processor
    pub fn new(worker_threads: usize, chunk_size: usize) -> Self {
        ParallelBlockProcessor {
            worker_threads,
            chunk_size,
        }
    }
    
    /// Encrypt multiple blocks in parallel
    /// 
    /// This uses Rayon for CPU-bound parallel encryption operations.
    /// Can achieve 10x speedup on multi-core systems.
    pub fn encrypt_blocks_parallel(
        &self,
        data_chunks: Vec<Vec<u8>>,
        identity: Arc<UnlockedIdentity>,
    ) -> Result<Vec<QuantumDataBlock>> {
        tracing::info!("⚡ Encrypting {} blocks in parallel ({} threads)", 
            data_chunks.len(), self.worker_threads);
        
        let start = std::time::Instant::now();
        
        // Configure Rayon thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.worker_threads)
            .build()
            .map_err(|e| MSSCSError::Config(format!("Thread pool creation failed: {}", e)))?;
        
        // Parallel encryption
        let blocks: Result<Vec<_>> = pool.install(|| {
            data_chunks
                .par_iter()
                .enumerate()
                .map(|(i, chunk)| {
                    QuantumDataBlock::new(
                        chunk,
                        i as u64,
                        None,
                        [0u8; 32],
                        &identity,
                        Some("application/octet-stream".to_string()),
                    )
                })
                .collect()
        });
        
        let elapsed = start.elapsed();
        let blocks = blocks?;
        
        tracing::info!("   ✅ Encrypted {} blocks in {:.2}s ({:.0} blocks/s)", 
            blocks.len(), 
            elapsed.as_secs_f64(),
            blocks.len() as f64 / elapsed.as_secs_f64()
        );
        
        Ok(blocks)
    }
    
    /// Decrypt multiple blocks in parallel
    pub fn decrypt_blocks_parallel(
        &self,
        blocks: Vec<QuantumDataBlock>,
        identity: Arc<UnlockedIdentity>,
    ) -> Result<Vec<Vec<u8>>> {
        tracing::info!("⚡ Decrypting {} blocks in parallel ({} threads)", 
            blocks.len(), self.worker_threads);
        
        let start = std::time::Instant::now();
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.worker_threads)
            .build()
            .map_err(|e| MSSCSError::Config(format!("Thread pool creation failed: {}", e)))?;
        
        // Parallel decryption
        let data: Result<Vec<_>> = pool.install(|| {
            blocks
                .par_iter()
                .map(|block| block.decode(&identity))
                .collect()
        });
        
        let elapsed = start.elapsed();
        let data = data?;
        
        tracing::info!("   ✅ Decrypted {} blocks in {:.2}s ({:.0} blocks/s)", 
            data.len(), 
            elapsed.as_secs_f64(),
            data.len() as f64 / elapsed.as_secs_f64()
        );
        
        Ok(data)
    }
    
    /// Compress multiple chunks in parallel
    pub fn compress_parallel(&self, chunks: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        tracing::debug!("⚡ Compressing {} chunks in parallel", chunks.len());
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.worker_threads)
            .build()
            .map_err(|e| MSSCSError::Config(format!("Thread pool creation failed: {}", e)))?;
        
        let compressed = pool.install(|| {
            chunks
                .par_iter()
                .map(|chunk| {
                    // Use zstd for better compression
                    zstd::bulk::compress(chunk, 3)
                        .map_err(|e| MSSCSError::Compression(format!("Compression failed: {}", e)))
                })
                .collect::<Result<Vec<_>>>()
        });
        
        compressed
    }
    
    /// Decompress multiple chunks in parallel
    pub fn decompress_parallel(&self, chunks: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        tracing::debug!("⚡ Decompressing {} chunks in parallel", chunks.len());
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.worker_threads)
            .build()
            .map_err(|e| MSSCSError::Config(format!("Thread pool creation failed: {}", e)))?;
        
        let decompressed = pool.install(|| {
            chunks
                .par_iter()
                .map(|chunk| {
                    zstd::bulk::decompress(chunk, 10 * 1024 * 1024) // 10MB max
                        .map_err(|e| MSSCSError::Compression(format!("Decompression failed: {}", e)))
                })
                .collect::<Result<Vec<_>>>()
        });
        
        decompressed
    }
    
    /// Split data into chunks for parallel processing
    pub fn split_into_chunks(&self, data: &[u8]) -> Vec<Vec<u8>> {
        data.chunks(self.chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
    
    /// Combine chunks back into single data
    pub fn combine_chunks(&self, chunks: Vec<Vec<u8>>) -> Vec<u8> {
        chunks.into_iter().flatten().collect()
    }
}

impl Default for ParallelBlockProcessor {
    fn default() -> Self {
        let worker_threads = num_cpus::get();
        Self::new(worker_threads, 1024 * 1024) // 1MB chunks
    }
}

/// Async parallel operations for network I/O
pub struct AsyncParallelProcessor;

impl AsyncParallelProcessor {
    /// Upload multiple blocks concurrently
    pub async fn upload_blocks_concurrent<F, Fut>(
        blocks: Vec<(String, Vec<u8>)>,
        upload_fn: F,
    ) -> Result<Vec<String>>
    where
        F: Fn(String, Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<String>> + Send,
    {
        tracing::info!("⚡ Uploading {} blocks concurrently", blocks.len());
        
        let start = std::time::Instant::now();
        let upload_fn = Arc::new(upload_fn);
        
        // Create concurrent upload tasks
        let mut tasks = Vec::new();
        for (block_id, block_data) in blocks {
            let upload_fn = upload_fn.clone();
            let task = task::spawn(async move {
                upload_fn(block_id, block_data).await
            });
            tasks.push(task);
        }
        
        // Wait for all uploads to complete
        let mut results = Vec::new();
        for task in tasks {
            let result = task.await
                .map_err(|e| MSSCSError::Network(format!("Task join failed: {}", e)))??;
            results.push(result);
        }
        
        let elapsed = start.elapsed();
        tracing::info!("   ✅ Uploaded {} blocks in {:.2}s ({:.0} blocks/s)", 
            results.len(), 
            elapsed.as_secs_f64(),
            results.len() as f64 / elapsed.as_secs_f64()
        );
        
        Ok(results)
    }
    
    /// Download multiple blocks concurrently
    pub async fn download_blocks_concurrent<F, Fut>(
        block_ids: Vec<String>,
        download_fn: F,
    ) -> Result<Vec<Vec<u8>>>
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Vec<u8>>> + Send,
    {
        tracing::info!("⚡ Downloading {} blocks concurrently", block_ids.len());
        
        let start = std::time::Instant::now();
        let download_fn = Arc::new(download_fn);
        
        // Create concurrent download tasks
        let mut tasks = Vec::new();
        for block_id in block_ids {
            let download_fn = download_fn.clone();
            let task = task::spawn(async move {
                download_fn(block_id).await
            });
            tasks.push(task);
        }
        
        // Wait for all downloads to complete
        let mut results = Vec::new();
        for task in tasks {
            let result = task.await
                .map_err(|e| MSSCSError::Network(format!("Task join failed: {}", e)))??;
            results.push(result);
        }
        
        let elapsed = start.elapsed();
        tracing::info!("   ✅ Downloaded {} blocks in {:.2}s ({:.0} blocks/s)", 
            results.len(), 
            elapsed.as_secs_f64(),
            results.len() as f64 / elapsed.as_secs_f64()
        );
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::QuantumIdentity;
    
    #[test]
    fn test_parallel_processor_creation() {
        let processor = ParallelBlockProcessor::new(4, 1024);
        assert_eq!(processor.worker_threads, 4);
        assert_eq!(processor.chunk_size, 1024);
    }
    
    #[test]
    fn test_split_combine_chunks() {
        let processor = ParallelBlockProcessor::new(4, 10);
        let data = b"Hello, World! This is a test.";
        
        let chunks = processor.split_into_chunks(data);
        assert!(chunks.len() > 1);
        
        let combined = processor.combine_chunks(chunks);
        assert_eq!(combined, data);
    }
    
    #[test]
    fn test_parallel_encryption() {
        let processor = ParallelBlockProcessor::new(2, 1024);
        let identity = QuantumIdentity::new("test-pass").unwrap();
        let unlocked = Arc::new(identity.unlock("test-pass").unwrap());
        
        let chunks = vec![
            b"Chunk 1".to_vec(),
            b"Chunk 2".to_vec(),
            b"Chunk 3".to_vec(),
        ];
        
        let blocks = processor.encrypt_blocks_parallel(chunks, unlocked.clone()).unwrap();
        assert_eq!(blocks.len(), 3);
        
        let decrypted = processor.decrypt_blocks_parallel(blocks, unlocked).unwrap();
        assert_eq!(decrypted.len(), 3);
        assert_eq!(decrypted[0], b"Chunk 1");
        assert_eq!(decrypted[1], b"Chunk 2");
        assert_eq!(decrypted[2], b"Chunk 3");
    }
    
    #[tokio::test]
    async fn test_async_concurrent_operations() {
        // Mock upload function
        let upload_fn = |block_id: String, _data: Vec<u8>| async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            Ok(block_id)
        };
        
        let blocks = vec![
            ("block1".to_string(), vec![1, 2, 3]),
            ("block2".to_string(), vec![4, 5, 6]),
            ("block3".to_string(), vec![7, 8, 9]),
        ];
        
        let results = AsyncParallelProcessor::upload_blocks_concurrent(blocks, upload_fn).await.unwrap();
        assert_eq!(results.len(), 3);
    }
}
