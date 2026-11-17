//! ISO image validation and boot testing framework for SigmaOS.
//!
//! Provides ISO integrity checks, boot sector validation, and test reporting.
//! Dependency-free.

use std::fmt;

/// Errors for ISO testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsoTestError {
    InvalidIso(String),
    BootSectorMissing,
    FileTooSmall,
    ChecksumMismatch,
}

impl fmt::Display for IsoTestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIso(m) => write!(f, "invalid ISO: {m}"),
            Self::BootSectorMissing => write!(f, "boot sector missing"),
            Self::FileTooSmall => write!(f, "file too small for ISO"),
            Self::ChecksumMismatch => write!(f, "checksum mismatch"),
        }
    }
}

impl std::error::Error for IsoTestError {}

/// ISO test result.
#[derive(Debug, Clone, PartialEq)]
pub struct IsoTestResult {
    pub valid: bool,
    pub bootable: bool,
    pub size_mb: u64,
    pub checksum: String,
    pub errors: Vec<String>,
}

impl IsoTestResult {
    pub fn ok(size_mb: u64, checksum: String) -> Self {
        Self {
            valid: true,
            bootable: true,
            size_mb,
            checksum,
            errors: Vec::new(),
        }
    }

    pub fn with_error(mut self, err: impl Into<String>) -> Self {
        self.valid = false;
        self.bootable = false;
        self.errors.push(err.into());
        self
    }
}

/// ISO validator and tester.
#[derive(Debug, Clone, Default)]
pub struct IsoTester {
    min_size_bytes: u64,
}

impl IsoTester {
    pub fn new() -> Self {
        Self { min_size_bytes: 1024 * 1024 } // 1 MB minimum
    }

    pub fn with_min_size(mut self, bytes: u64) -> Self {
        self.min_size_bytes = bytes;
        self
    }

    pub fn validate_size(&self, size_bytes: u64) -> Result<(), IsoTestError> {
        if size_bytes < self.min_size_bytes {
            return Err(IsoTestError::FileTooSmall);
        }
        Ok(())
    }

    pub fn check_boot_signature(&self, data: &[u8]) -> Result<(), IsoTestError> {
        if data.len() < 2048 {
            return Err(IsoTestError::FileTooSmall);
        }
        // Check for ISO 9660 magic at offset 32769 (sector 16, offset 2048)
        let offset = 32769;
        if offset + 6 > data.len() {
            return Err(IsoTestError::InvalidIso("ISO9660 magic missing".into()));
        }
        let magic = &data[offset..offset + 6];
        if magic != b"CD001\x01" {
            return Err(IsoTestError::InvalidIso("invalid ISO9660 signature".into()));
        }
        Ok(())
    }

    pub fn compute_checksum(&self, data: &[u8]) -> String {
        // Simple FNV-1a hash for demo purposes
        let mut hash: u64 = 0xcbf29ce484222325;
        for &byte in data.iter().take(4096) { // First 4KB only for speed
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{:016x}", hash)
    }

    pub fn test_iso(&self, data: &[u8]) -> IsoTestResult {
        let mut result = IsoTestResult::ok(
            (data.len() / (1024 * 1024)) as u64,
            self.compute_checksum(data),
        );

        if let Err(e) = self.validate_size(data.len() as u64) {
            result = result.with_error(e.to_string());
        }
        if let Err(e) = self.check_boot_signature(data) {
            result = result.with_error(e.to_string());
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_validation_aaa() {
        // Arrange
        let tester = IsoTester::new().with_min_size(1024);

        // Act
        let res = tester.validate_size(512);

        // Assert
        assert!(res.is_err());
        assert!(tester.validate_size(2048).is_ok());
    }

    #[test]
    fn test_checksum_deterministic_aaa() {
        // Arrange
        let tester = IsoTester::new();
        let data = vec![0u8; 4096];

        // Act
        let h1 = tester.compute_checksum(&data);
        let h2 = tester.compute_checksum(&data);

        // Assert
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
    }

    #[test]
    fn test_iso_test_small_file_aaa() {
        // Arrange
        let tester = IsoTester::new();
        let small_data = vec![0u8; 100]; // Too small

        // Act
        let result = tester.test_iso(&small_data);

        // Assert
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }
}
