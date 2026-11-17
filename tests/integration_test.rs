// Integration tests for SigmaOS-iso-tests

use sigmaos_iso_tests::IsoTester;

#[test]
fn test_iso_tester_creation() {
    // Arrange
    let tester = IsoTester::new();

    // Act
    let result = tester.test_iso(&vec![0u8; 100]);

    // Assert
    assert!(!result.valid);
    assert!(!result.errors.is_empty());
}
