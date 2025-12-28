use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PhoneType {
    Mobile,
    Landline,
    Unknown,
}

lazy_static! {
    // Украинские мобильные коды (операторы)
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

    // Регулярные выражения для очистки телефонов
    static ref PHONE_REGEX: Regex = Regex::new(r"[^\d]").unwrap();
    static ref UA_PHONE_REGEX: Regex = Regex::new(r"^(\d{3})(\d{2})(\d{2})(\d{2})$").unwrap();
}

/// Нормализует украинский телефонный номер
/// Примеры входных данных:
/// - "(067) 123-45-67"
/// - "+380671234567"
/// - "067.123.45.67"
/// - "380671234567"
/// 
/// Возвращает кортеж: (нормализованный_номер, тип_телефона)
pub fn normalize_phone(phone: &str) -> Result<(String, PhoneType), String> {
    // Удаляем все нецифровые символы
    let cleaned: String = PHONE_REGEX.replace_all(phone, "").to_string();
    
    // Если номер начинается с +38 или 38, удаляем префикс
    let cleaned = if cleaned.starts_with("38") && cleaned.len() > 10 {
        cleaned[2..].to_string()
    } else if cleaned.starts_with("8") && cleaned.len() > 10 {
        cleaned[1..].to_string()
    } else if cleaned.len() == 9 {
        // Если 9 цифр, добавляем ведущий ноль (код оператора)
        format!("0{}", cleaned)
    } else {
        cleaned
    };
    
    // Проверяем, что это украинский номер (10 цифр: код оператора + номер)
    if cleaned.len() != 10 {
        return Err(format!("Неверная длина номера: {}", cleaned.len()));
    }
    
    // Извлекаем код оператора (первые 3 цифры)
    let operator_code = &cleaned[0..3];
    
    if operator_code == "000" {
        return Err("Код оператора не может быть 000".to_string());
    }
    
    // Определяем тип телефона
    let phone_type = if MOBILE_CODES.contains(operator_code) {
        PhoneType::Mobile
    } else {
        PhoneType::Landline
    };
    
    // Форматируем в международный формат: 380XXYYYYZZZZ
    let normalized = format!("38{}", cleaned);
    
    Ok((normalized, phone_type))
}

/// Проверяет, является ли номер мобильным
pub fn is_mobile_phone(phone: &str) -> bool {
    match normalize_phone(phone) {
        Ok((_, PhoneType::Mobile)) => true,
        _ => false,
    }
}

/// Форматирует телефон для отображения пользователю
pub fn format_phone_for_display(phone: &str) -> String {
    // Удаляем все нецифровые символы
    let cleaned = PHONE_REGEX.replace_all(phone, "");
    
    if cleaned.len() == 10 {
        format!("({}) {}-{}-{}", 
            &cleaned[0..3], 
            &cleaned[3..5], 
            &cleaned[5..7], 
            &cleaned[7..9]
        )
    } else if cleaned.len() == 12 {
        format!("+{} ({}) {}-{}-{}", 
            &cleaned[0..2],
            &cleaned[2..5], 
            &cleaned[5..7], 
            &cleaned[7..9], 
            &cleaned[9..11]
        )
    } else {
        phone.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_phone() {
        let test_cases = vec![
            ("(067) 123-45-67", "380671234567", PhoneType::Mobile),
            ("+380671234567", "380671234567", PhoneType::Mobile),
            ("067.123.45.67", "380671234567", PhoneType::Mobile),
            ("380671234567", "380671234567", PhoneType::Mobile),
            ("(044) 123-45-67", "380441234567", PhoneType::Landline), // Киев
            ("(056) 123-45-67", "380561234567", PhoneType::Landline), // Днепр
        ];
        
        for (input, expected_num, expected_type) in test_cases {
            let result = normalize_phone(input).unwrap();
            assert_eq!(result.0, expected_num);
            assert_eq!(result.1, expected_type);
        }
    }

    #[test]
    fn test_mobile_codes() {
        let mobile_codes = vec!["050", "066", "067", "068", "063", "073", "093"];
        for code in mobile_codes {
            let phone = format!("{}1234567", code);
            let result = normalize_phone(&phone).unwrap();
            assert_eq!(result.1, PhoneType::Mobile, "Code {} should be mobile", code);
        }
    }
}