use leadminer_ultimate::{data_normalizer, proxy_manager, scraper};
use std::sync::Arc;
use tokio::sync::Mutex;

// Импортируем функции для тестов
use scraper::{extract_phone_from_text, extract_address_from_text};

#[tokio::test]
async fn test_normalize_phone() {
    // Тест нормализации украинских телефонов
    let test_cases = vec![
        ("(067) 123-45-67", "380671234567"),
        ("+380671234567", "380671234567"),
        ("067.123.45.67", "380671234567"),
        ("380671234567", "380671234567"),
    ];

    for (input, expected) in test_cases {
        let result = data_normalizer::normalize_phone(input);
        assert!(result.is_ok(), "Должен успешно нормализовать: {}", input);
        let (normalized, _) = result.unwrap();
        assert_eq!(normalized, expected, "Неверная нормализация для {}", input);
    }
}

#[tokio::test]
async fn test_proxy_rotator() {
    let mut rotator = proxy_manager::ProxyRotator::new();
    
    // Добавляем тестовые прокси
    rotator.update_proxies(vec![
        "http://proxy1:8080".to_string(),
        "http://proxy2:8080".to_string(),
    ]);

    assert_eq!(rotator.get_proxy_count(), 2);

    // Проверяем ротацию
    let first = rotator.get_next_proxy().unwrap();
    assert_eq!(first.url, "http://proxy1:8080");

    // Симулируем 10 запросов
    for _ in 0..10 {
        rotator.get_next_proxy();
    }

    let second = rotator.get_next_proxy().unwrap();
    assert_eq!(second.url, "http://proxy2:8080");
}

#[tokio::test]
async fn test_scraper_google() {
    let scraper = scraper::LeadScraper::new();
    let proxy_rotator = Arc::new(Mutex::new(proxy_manager::ProxyRotator::new()));

    // Тестовый запрос
    let result = scraper.scrape("Киев", "обувь", proxy_rotator).await;

    match result {
        Ok(leads) => {
            println!("Google: найдено {} лидов", leads.len());
            for lead in &leads {
                println!("- {} | {} | {} | {:?}", lead.name, lead.address, lead.phone, lead.source);
            }
            assert!(!leads.is_empty(), "Должны быть найдены лиды");
        }
        Err(e) => {
            println!("Ошибка Google: {}", e);
            // Не фейлим, т.к. Google может блокировать
        }
    }
}

#[tokio::test]
async fn test_scraper_2gis() {
    let scraper = scraper::LeadScraper::new();
    let proxy_rotator = Arc::new(Mutex::new(proxy_manager::ProxyRotator::new()));

    // Тестовый запрос
    let result = scraper.scrape("Львов", "кафе", proxy_rotator).await;

    match result {
        Ok(leads) => {
            println!("2GIS: найдено {} лидов", leads.len());
            for lead in &leads {
                println!("- {} | {} | {} | {:?}", lead.name, lead.address, lead.phone, lead.source);
            }
            assert!(!leads.is_empty(), "Должны быть найдены лиды");
        }
        Err(e) => {
            println!("Ошибка 2GIS: {}", e);
            // Не фейлим, т.к. 2GIS может блокировать
        }
    }
}

#[tokio::test]
async fn test_internet_connection() {
    let client = reqwest::Client::new();
    let result = client.get("https://8.8.8.8").timeout(std::time::Duration::from_secs(5)).send().await;
    assert!(result.is_ok(), "Должен быть доступ к интернету");
}

#[tokio::test]
async fn test_google_access() {
    let client = reqwest::Client::new();
    let result = client.get("https://www.google.com")
        .header("User-Agent", "Mozilla/5.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match result {
        Ok(resp) => {
            assert!(resp.status().is_success(), "Google должен отвечать");
            println!("Google доступен, статус: {}", resp.status());
        }
        Err(e) => {
            println!("Google недоступен: {}", e);
            // Не фейлим тест
        }
    }
}

#[tokio::test]
async fn test_2gis_access() {
    let client = reqwest::Client::new();
    let result = client.get("https://2gis.ua")
        .header("User-Agent", "Mozilla/5.0")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match result {
        Ok(resp) => {
            assert!(resp.status().is_success(), "2GIS должен отвечать");
            println!("2GIS доступен, статус: {}", resp.status());
        }
        Err(e) => {
            println!("2GIS недоступен: {}", e);
            // Не фейлим тест
        }
    }
}

#[test]
fn test_phone_extraction() {
    use leadminer_ultimate::scraper::extract_phone_from_text;
    
    let texts = vec![
        "Телефон: (067) 123-45-67",
        "Call us: +380671234567",
        "Номер: 067.123.45.67",
        "Тел: 067 123 45 67",
    ];

    for text in texts {
        let phone = extract_phone_from_text(text);
        assert!(!phone.is_empty(), "Должен найти телефон в: {}", text);
        println!("Найден телефон: {} в тексте: {}", phone, text);
    }
}

#[test]
fn test_address_extraction() {
    use leadminer_ultimate::scraper::extract_address_from_text;
    
    let texts = vec![
        "Адрес: ул. Хрещатик, 1, Киев",
        "вул. Леси Украинки, 5, Львов",
        "просп. Перемоги, 25, Днепр",
    ];

    for text in texts {
        let address = extract_address_from_text(text);
        assert!(!address.is_empty(), "Должен найти адрес в: {}", text);
        println!("Найден адрес: {} в тексте: {}", address, text);
    }
}