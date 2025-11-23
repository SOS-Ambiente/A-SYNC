// Input validation module for Tauri commands
// Add this file to src-tauri/src/validation.rs

use uuid::Uuid;

/// Validates file paths to prevent security vulnerabilities
/// 
/// # Security Checks
/// - Path traversal prevention (..)
/// - Null byte injection prevention
/// - Path length validation
/// - Empty path check
pub fn validate_file_path(path: &str) -> Result<(), String> {
    // Prevent path traversal attacks
    if path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }
    
    // Check for null bytes (can bypass security checks)
    if path.contains('\0') {
        return Err("Invalid path: null byte detected".to_string());
    }
    
    // Validate path length (prevent buffer overflow)
    if path.len() > 4096 {
        return Err("Path too long (max 4096 characters)".to_string());
    }
    
    // Check for empty path
    if path.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    
    Ok(())
}

/// Validates UUID format
pub fn validate_uuid(uuid_str: &str) -> Result<Uuid, String> {
    Uuid::parse_str(uuid_str)
        .map_err(|_| "Invalid UUID format".to_string())
}

/// Validates storage limit configuration
/// 
/// # Constraints
/// - Minimum: 100 MB
/// - Maximum: 1 TB (1,000,000 MB)
pub fn validate_storage_limit(limit_mb: u64) -> Result<(), String> {
    if limit_mb < 100 {
        return Err("Storage limit must be at least 100 MB".to_string());
    }
    
    if limit_mb > 1_000_000 {
        return Err("Storage limit cannot exceed 1 TB".to_string());
    }
    
    Ok(())
}

/// Validates peer ID format
pub fn validate_peer_id(peer_id: &str) -> Result<(), String> {
    // Check length (typical peer IDs are 46-52 characters)
    if peer_id.len() < 10 || peer_id.len() > 100 {
        return Err("Invalid peer ID length".to_string());
    }
    
    // Check for valid characters (alphanumeric + some special chars)
    if !peer_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("Invalid peer ID format".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_path() {
        // Valid paths
        assert!(validate_file_path("test.txt").is_ok());
        assert!(validate_file_path("folder/test.txt").is_ok());
        assert!(validate_file_path("/absolute/path/test.txt").is_ok());
        
        // Invalid paths
        assert!(validate_file_path("../etc/passwd").is_err());
        assert!(validate_file_path("test/../../../etc/passwd").is_err());
        assert!(validate_file_path("test\0.txt").is_err());
        assert!(validate_file_path("").is_err());
        assert!(validate_file_path("   ").is_err());
        
        // Path too long
        let long_path = "a".repeat(5000);
        assert!(validate_file_path(&long_path).is_err());
    }

    #[test]
    fn test_validate_uuid() {
        // Valid UUID
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        
        // Invalid UUIDs
        assert!(validate_uuid("not-a-uuid").is_err());
        assert!(validate_uuid("").is_err());
        assert!(validate_uuid("550e8400-e29b-41d4-a716").is_err());
    }

    #[test]
    fn test_validate_storage_limit() {
        // Valid limits
        assert!(validate_storage_limit(100).is_ok());
        assert!(validate_storage_limit(10240).is_ok());
        assert!(validate_storage_limit(1_000_000).is_ok());
        
        // Invalid limits
        assert!(validate_storage_limit(50).is_err());
        assert!(validate_storage_limit(0).is_err());
        assert!(validate_storage_limit(1_000_001).is_err());
    }

    #[test]
    fn test_validate_peer_id() {
        // Valid peer IDs
        assert!(validate_peer_id("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N").is_ok());
        assert!(validate_peer_id("12D3KooWD3bfmNbuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuu").is_ok());
        
        // Invalid peer IDs
        assert!(validate_peer_id("short").is_err());
        assert!(validate_peer_id("").is_err());
        assert!(validate_peer_id("invalid@peer#id").is_err());
    }
}
