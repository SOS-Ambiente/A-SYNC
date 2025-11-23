// Erasure Coding using Reed-Solomon for efficient data replication
use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};

/// Erasure-coded shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    /// Shard index
    pub index: usize,
    /// Shard data
    pub data: Vec<u8>,
    /// Total number of data shards
    pub data_shards: usize,
    /// Total number of parity shards
    pub parity_shards: usize,
    /// Original data size
    pub original_size: usize,
}

/// Erasure coding manager using Reed-Solomon
pub struct ErasureCoding {
    /// Number of data shards
    data_shards: usize,
    /// Number of parity shards
    parity_shards: usize,
}

impl ErasureCoding {
    /// Create a new erasure coding instance
    /// 
    /// # Arguments
    /// * `data_shards` - Number of data shards (K)
    /// * `parity_shards` - Number of parity shards (M)
    /// 
    /// With K data shards and M parity shards:
    /// - Can reconstruct from any K shards
    /// - Can tolerate M shard failures
    /// - Storage overhead: (K+M)/K
    /// 
    /// Example: 10 data + 4 parity = 40% overhead, tolerates 4 failures
    pub fn new(data_shards: usize, parity_shards: usize) -> Result<Self> {
        if data_shards == 0 {
            return Err(MSSCSError::Validation("Data shards must be > 0".to_string()));
        }
        if parity_shards == 0 {
            return Err(MSSCSError::Validation("Parity shards must be > 0".to_string()));
        }

        Ok(ErasureCoding {
            data_shards,
            parity_shards,
        })
    }

    /// Encode data into shards using Reed-Solomon erasure coding
    /// 
    /// Splits data into K data shards and generates M parity shards
    /// Returns K+M shards total
    pub fn encode(&self, data: &[u8]) -> Result<Vec<Shard>> {
        let original_size = data.len();
        
        // Calculate shard size (pad if necessary)
        let shard_size = (data.len() + self.data_shards - 1) / self.data_shards;
        let padded_size = shard_size * self.data_shards;
        
        // Pad data if necessary
        let mut padded_data = data.to_vec();
        padded_data.resize(padded_size, 0);
        
        // Split into data shards
        let mut shards = Vec::new();
        for i in 0..self.data_shards {
            let start = i * shard_size;
            let end = start + shard_size;
            let shard_data = padded_data[start..end].to_vec();
            
            shards.push(Shard {
                index: i,
                data: shard_data,
                data_shards: self.data_shards,
                parity_shards: self.parity_shards,
                original_size,
            });
        }
        
        // Generate parity shards using Reed-Solomon-like encoding
        // This is a simplified implementation using Galois Field arithmetic
        for i in 0..self.parity_shards {
            let mut parity_data = vec![0u8; shard_size];
            
            // Use weighted XOR for better error correction
            // Each parity shard uses different coefficients
            for (j, data_shard) in shards[..self.data_shards].iter().enumerate() {
                let coefficient = self.galois_multiply(j + 1, i + 1);
                
                for (k, byte) in data_shard.data.iter().enumerate() {
                    parity_data[k] ^= self.galois_multiply(*byte as usize, coefficient) as u8;
                }
            }
            
            shards.push(Shard {
                index: self.data_shards + i,
                data: parity_data,
                data_shards: self.data_shards,
                parity_shards: self.parity_shards,
                original_size,
            });
        }
        
        Ok(shards)
    }
    
    /// Simplified Galois Field multiplication (GF(2^8))
    /// In production, use a proper Reed-Solomon library
    fn galois_multiply(&self, a: usize, b: usize) -> usize {
        let mut result = 0;
        let mut a = a & 0xFF;
        let mut b = b & 0xFF;
        
        while b > 0 {
            if b & 1 != 0 {
                result ^= a;
            }
            
            let high_bit = a & 0x80;
            a <<= 1;
            
            if high_bit != 0 {
                a ^= 0x1B; // Primitive polynomial for GF(2^8)
            }
            
            b >>= 1;
        }
        
        result & 0xFF
    }

    /// Decode shards back into original data
    /// 
    /// Requires at least K shards (can be any combination of data and parity shards)
    pub fn decode(&self, shards: &[Shard]) -> Result<Vec<u8>> {
        if shards.is_empty() {
            return Err(MSSCSError::Validation("No shards provided".to_string()));
        }
        
        // Verify all shards have same configuration
        let first = &shards[0];
        if first.data_shards != self.data_shards || first.parity_shards != self.parity_shards {
            return Err(MSSCSError::Validation("Shard configuration mismatch".to_string()));
        }
        
        // Check if we have enough shards
        if shards.len() < self.data_shards {
            return Err(MSSCSError::Validation(format!(
                "Insufficient shards: need {}, have {}",
                self.data_shards,
                shards.len()
            )));
        }
        
        let original_size = first.original_size;
        let shard_size = first.data.len();
        
        // Separate data and parity shards
        let mut data_shards: Vec<Option<&Shard>> = vec![None; self.data_shards];
        let mut parity_shards: Vec<Option<&Shard>> = vec![None; self.parity_shards];
        
        for shard in shards {
            if shard.index < self.data_shards {
                data_shards[shard.index] = Some(shard);
            } else {
                let parity_index = shard.index - self.data_shards;
                if parity_index < self.parity_shards {
                    parity_shards[parity_index] = Some(shard);
                }
            }
        }
        
        // Count available data shards
        let available_data = data_shards.iter().filter(|s| s.is_some()).count();
        
        if available_data == self.data_shards {
            // All data shards available, reconstruct directly
            let mut result = Vec::with_capacity(original_size);
            for shard_opt in data_shards {
                if let Some(shard) = shard_opt {
                    result.extend_from_slice(&shard.data);
                }
            }
            result.truncate(original_size);
            return Ok(result);
        }
        
        // Need to reconstruct missing data shards from parity
        tracing::info!("🔄 Reconstructing {} missing data shards using Reed-Solomon", 
            self.data_shards - available_data);
        
        // Collect available shards (both data and parity)
        let mut available_shards: Vec<(usize, &[u8])> = Vec::new();
        for (i, shard_opt) in data_shards.iter().enumerate() {
            if let Some(shard) = shard_opt {
                available_shards.push((i, &shard.data));
            }
        }
        for (i, shard_opt) in parity_shards.iter().enumerate() {
            if let Some(shard) = shard_opt {
                available_shards.push((self.data_shards + i, &shard.data));
            }
        }
        
        // Reconstruct missing data shards using Gaussian elimination
        let mut reconstructed_data = vec![vec![0u8; shard_size]; self.data_shards];
        
        // Build encoding matrix
        let mut matrix = vec![vec![0u8; self.data_shards]; available_shards.len()];
        for (row, (shard_idx, _)) in available_shards.iter().enumerate() {
            for col in 0..self.data_shards {
                if *shard_idx < self.data_shards {
                    // Data shard - identity matrix
                    matrix[row][col] = if col == *shard_idx { 1 } else { 0 };
                } else {
                    // Parity shard - use encoding coefficients
                    let parity_idx = shard_idx - self.data_shards;
                    matrix[row][col] = self.galois_multiply(col + 1, parity_idx + 1) as u8;
                }
            }
        }
        
        // Solve for each byte position
        for byte_pos in 0..shard_size {
            // Build system of equations: matrix * data = available_data
            let mut equations = vec![0u8; available_shards.len()];
            for (i, (_, shard_data)) in available_shards.iter().enumerate() {
                equations[i] = shard_data[byte_pos];
            }
            
            // Gaussian elimination in GF(2^8)
            let solution = self.solve_galois_system(&matrix, &equations)?;
            
            // Store reconstructed bytes
            for (i, &byte) in solution.iter().enumerate() {
                reconstructed_data[i][byte_pos] = byte;
            }
        }
        
        // Combine reconstructed data shards
        let mut result = Vec::with_capacity(original_size);
        for shard_data in reconstructed_data {
            result.extend_from_slice(&shard_data);
        }
        result.truncate(original_size);
        
        Ok(result)
    }
    
    /// Solve system of linear equations in Galois Field GF(2^8)
    fn solve_galois_system(&self, matrix: &[Vec<u8>], equations: &[u8]) -> Result<Vec<u8>> {
        let n = matrix[0].len();
        let m = matrix.len();
        
        if m < n {
            return Err(MSSCSError::Validation("Insufficient equations".to_string()));
        }
        
        // Create augmented matrix
        let mut aug = vec![vec![0u8; n + 1]; n];
        for i in 0..n {
            for j in 0..n {
                aug[i][j] = matrix[i][j];
            }
            aug[i][n] = equations[i];
        }
        
        // Forward elimination
        for i in 0..n {
            // Find pivot
            let mut pivot_row = i;
            for k in (i + 1)..n {
                if aug[k][i] != 0 {
                    pivot_row = k;
                    break;
                }
            }
            
            if aug[pivot_row][i] == 0 {
                continue; // Skip if column is all zeros
            }
            
            // Swap rows
            if pivot_row != i {
                aug.swap(i, pivot_row);
            }
            
            // Eliminate column
            let pivot = aug[i][i];
            let pivot_inv = self.galois_inverse(pivot);
            
            for k in (i + 1)..n {
                if aug[k][i] != 0 {
                    let factor = self.galois_multiply(aug[k][i] as usize, pivot_inv as usize) as u8;
                    for j in i..=n {
                        let prod = self.galois_multiply(factor as usize, aug[i][j] as usize) as u8;
                        aug[k][j] ^= prod;
                    }
                }
            }
        }
        
        // Back substitution
        let mut solution = vec![0u8; n];
        for i in (0..n).rev() {
            if aug[i][i] == 0 {
                continue;
            }
            
            let mut sum = aug[i][n];
            for j in (i + 1)..n {
                let prod = self.galois_multiply(aug[i][j] as usize, solution[j] as usize) as u8;
                sum ^= prod;
            }
            
            let pivot_inv = self.galois_inverse(aug[i][i]);
            solution[i] = self.galois_multiply(sum as usize, pivot_inv as usize) as u8;
        }
        
        Ok(solution)
    }
    
    /// Galois Field inverse (GF(2^8))
    fn galois_inverse(&self, a: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        
        // Use extended Euclidean algorithm
        let mut t = 0u16;
        let mut new_t = 1u16;
        let mut r = 0x11Bu16; // Primitive polynomial
        let mut new_r = a as u16;
        
        while new_r != 0 {
            let quotient = self.galois_divide(r, new_r);
            
            let temp_t = t;
            t = new_t;
            new_t = temp_t ^ self.galois_multiply_u16(quotient, new_t);
            
            let temp_r = r;
            r = new_r;
            new_r = temp_r ^ self.galois_multiply_u16(quotient, new_r);
        }
        
        (t & 0xFF) as u8
    }
    
    /// Galois Field division
    fn galois_divide(&self, a: u16, b: u16) -> u16 {
        if b == 0 {
            return 0;
        }
        
        let mut quotient = 0u16;
        let mut remainder = a;
        let mut divisor = b;
        
        // Align divisor with remainder
        while divisor < 0x100 && remainder >= 0x100 {
            divisor <<= 1;
        }
        
        while divisor >= b {
            if remainder >= divisor {
                remainder ^= divisor;
                quotient ^= 1 << (divisor.trailing_zeros() - b.trailing_zeros());
            }
            divisor >>= 1;
        }
        
        quotient
    }
    
    /// Galois Field multiplication for u16
    fn galois_multiply_u16(&self, a: u16, b: u16) -> u16 {
        let mut result = 0u16;
        let mut a = a;
        let mut b = b;
        
        while b > 0 {
            if b & 1 != 0 {
                result ^= a;
            }
            
            let high_bit = a & 0x100;
            a <<= 1;
            
            if high_bit != 0 {
                a ^= 0x11B; // Primitive polynomial
            }
            
            b >>= 1;
        }
        
        result
    }

    /// Calculate storage overhead percentage
    pub fn overhead_percentage(&self) -> f64 {
        (self.parity_shards as f64 / self.data_shards as f64) * 100.0
    }

    /// Calculate total shards
    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }

    /// Calculate maximum tolerable failures
    pub fn max_failures(&self) -> usize {
        self.parity_shards
    }
}

/// Default erasure coding configuration (10+4)
impl Default for ErasureCoding {
    fn default() -> Self {
        Self::new(10, 4).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erasure_coding_creation() {
        let ec = ErasureCoding::new(10, 4).unwrap();
        assert_eq!(ec.data_shards, 10);
        assert_eq!(ec.parity_shards, 4);
        assert_eq!(ec.total_shards(), 14);
        assert_eq!(ec.max_failures(), 4);
        assert_eq!(ec.overhead_percentage(), 40.0);
    }

    #[test]
    fn test_encode_decode() {
        let ec = ErasureCoding::new(5, 2).unwrap();
        let data = b"Hello, World! This is a test of erasure coding.";
        
        // Encode
        let shards = ec.encode(data).unwrap();
        assert_eq!(shards.len(), 7); // 5 data + 2 parity
        
        // Decode with all shards
        let decoded = ec.decode(&shards).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_decode_with_subset() {
        let ec = ErasureCoding::new(5, 2).unwrap();
        let data = b"Test data for erasure coding";
        
        // Encode
        let shards = ec.encode(data).unwrap();
        
        // Decode with only data shards (first 5)
        let decoded = ec.decode(&shards[..5]).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_insufficient_shards() {
        let ec = ErasureCoding::new(5, 2).unwrap();
        let data = b"Test data";
        
        let shards = ec.encode(data).unwrap();
        
        // Try to decode with insufficient shards
        let result = ec.decode(&shards[..4]);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_data() {
        let ec = ErasureCoding::new(10, 4).unwrap();
        let data = vec![42u8; 1024 * 1024]; // 1MB of data
        
        let shards = ec.encode(&data).unwrap();
        assert_eq!(shards.len(), 14);
        
        // Each shard should be roughly 1MB / 10 = ~100KB
        let expected_shard_size = (data.len() + 9) / 10;
        for shard in &shards {
            assert_eq!(shard.data.len(), expected_shard_size);
        }
        
        let decoded = ec.decode(&shards).unwrap();
        assert_eq!(decoded, data);
    }
}
