use std::collections::HashSet;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq)]
pub enum PhoneType {
    Mobile,
    Landline,
    Unknown,
}

lazy_static! {
    static ref MOBILE_CODES: HashSet<&'static str> = {
        let mut codes = HashSet::new();
        codes.insert("050"); // Vodafone
        codes.insert("066"); // Vodafone
        codes.insert("095"); // Vodafone
        codes.insert("099"); // Vodafone
        codes.insert("067"); // Kyivstar
        codes.insert("068"); // Kyivstar
        codes.insert("096"); // Kyivstar
        codes.insert("097"); // Kyivstar
        codes.insert("098"); // Kyivstar
        codes.insert("063"); // Lifecell
        codes.insert("073"); // Lifecell
        codes.insert("093"); // Lifecell
        codes.insert("091"); // 3Mob
        codes.insert("092"); // 3Mob
        codes
    };

    static ref PHONE_REGEX: Regex = Regex::new(r"[^\d]").unwrap();
}

pub fn normalize_phone(phone: &str) -> Result<(String, PhoneType), String> {
    let cleaned: String = PHONE_REGEX.replace_all(phone, "").to_string();
    
    let cleaned = if cleaned.starts_with("38") && cleaned.len() > 10 {
        cleaned[2..].to_string()
    } else if cleaned.starts_with("8") && cleaned.len() > 10 {
        cleaned[1..].to_string()
    } else if cleaned.starts_with("0") && cleaned.len() == 10 {
        cleaned[1..].to_string()
    } else if cleaned.len() == 9 {
        format!("0{}", cleaned)
    } else {
        cleaned
    };
    
    if cleaned.len() != 10 {
        return Err(format!("Неверная длина номера: {}", cleaned.len()));
    }
    
    let operator_code = &cleaned[0..3];
    
    let phone_type = if MOBILE_CODES.contains(operator_code) {
        PhoneType::Mobile
    } else {
        PhoneType::Landline
    };
    
    let normalized = format!("380{}", cleaned);
    
    Ok((normalized, phone_type))
}

fn main() {
    let test_cases = vec![
        ("(067) 123-45-67", "380671234567", PhoneType::Mobile),
        ("+380671234567", "380671234567", PhoneType::Mobile),
        ("067.123.45.67", "380671234567", PhoneType::Mobile),
        ("380671234567", "380671234567", PhoneType::Mobile),
        ("(044) 123-45-67", "380441234567", PhoneType::Landline),
        ("(056) 123-45-67", "380561234567", PhoneType::Landline),
    ];
    
    for (input, expected_num, expected_type) in test_cases {
        match normalize_phone(input) {
            Ok((normalized, phone_type)) => {
                if normalized == expected_num && phone_type == expected_type {
                    println!("✓ PASS: {} -> {} ({:?})", input, normalized, phone_type);
                } else {
                    println!("✗ FAIL: {} -> {} ({:?}) (expected: {} ({:?}))", 
                            input, normalized, phone_type, expected_num, expected_type);
                }
            }
            Err(e) => {
                println!("✗ ERROR: {} -> {}", input, e);
            }
        }
    }
}