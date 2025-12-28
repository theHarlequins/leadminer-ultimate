use leadminer_ultimate::data_normalizer::{normalize_phone, PhoneType};

#[test]
fn test_normalize_ukrainian_mobiles() {
    // Arrange
    let inputs = vec![
        ("067-123-45-67", "380671234567"),
        ("+38 (044) 123 45 67", "380441234567"), // Kyiv Landline
        ("099 111 22 33", "380991112233"),
        ("80631234567", "380631234567"),
        ("380971234567", "380971234567"),
    ];

    for (input, expected) in inputs {
        // Act
        let result = normalize_phone(input);

        // Assert
        assert!(result.is_ok(), "Failed to normalize {}", input);
        let (normalized, _) = result.unwrap();
        assert_eq!(normalized, expected, "Input: {}", input);
    }
}

#[test]
fn test_detect_phone_type() {
    // Arrange
    let mobile = "067 123 45 67";
    let landline = "044 123 45 67";

    // Act
    let (_, type_mobile) = normalize_phone(mobile).unwrap();
    let (_, type_landline) = normalize_phone(landline).unwrap();

    // Assert
    assert_eq!(type_mobile, PhoneType::Mobile);
    assert_eq!(type_landline, PhoneType::Landline);
}

#[test]
fn test_reject_invalid_phone_format() {
    // Arrange
    let invalid_inputs = vec![
        "Gw. 555-123",
        "123",
        "0000000000", // Invalid operator code
        "abcdefghij",
    ];

    for input in invalid_inputs {
        // Act
        let result = normalize_phone(input);

        // Assert
        assert!(result.is_err(), "Should have failed for input: {}", input);
    }
}
