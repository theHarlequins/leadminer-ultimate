#
![cfg_attr(not(debug_assertions)
, windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;

// Импортируем модули из lib.rs
use leadminer_ultimate::{
    proxy_manager::ProxyRotator,
    data_normalizer::{normalize_phone, PhoneType},
    scraper::{EnrichedLead, LeadScraper},
    connection_test::ConnectionTester,
    ai::{AiService, OpenRouterModel},
};

// Глобальное состояние приложения
struct AppState {
    proxy_rotator: Arc<Mutex<ProxyRotator>>,
    scraper: Arc<LeadScraper>,
    ai_service: Arc<AiService>,
}

#[tauri::command]
async fn fetch_models(
    api_key: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<OpenRouterModel>, String> {
    state.ai_service.fetch_models(&api_key).await
}

// ... existing commands ...

#[tauri::command]
async fn start_scraping(
    city: String,
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<EnrichedLead>, String> {
    // ... code ...
    println!("=== LeadMiner: Запрос на скрапинг ===");
    println!("Город: {}", city);
    println!("Запрос: {}", query);
    println!("Время: {:?}", std::time::SystemTime::now());
    
    let scraper = state.scraper.clone();
    let proxy_rotator = state.proxy_rotator.clone();
    
    // Проверяем количество прокси
    {
        let rotator = proxy_rotator.lock().await;
        let proxy_count = rotator.get_proxy_count();
        println!("Доступно прокси: {}", proxy_count);
        if proxy_count == 0 {
            println!("Внимание: Прокси не настроены, используется прямое соединение");
        }
    }
    
    match scraper.scrape(&city, &query, proxy_rotator).await {
        Ok(leads) => {
            println!("=== Успешно! Найдено {} лидов ===", leads.len());
            Ok(leads)
        }
        Err(e) => {
            println!("=== Ошибка скрапинга: {} ===", e);
            Err(format!("Не удалось выполнить скрапинг: {}", e))
        }
    }
}

// ... update_proxies, normalize_phone_command, test_connection ...
#[tauri::command]
async fn update_proxies(
    proxies: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut rotator = state.proxy_rotator.lock().await;
    rotator.update_proxies(proxies.clone());
    println!("Обновлено прокси: {} шт.", proxies.len());
    Ok(())
}

#[tauri::command]
async fn normalize_phone_command(phone: String) -> Result<(String, PhoneType), String> {
    match normalize_phone(&phone) {
        Ok((normalized, phone_type)) => {
            println!("Нормализован телефон: {} -> {}", phone, normalized);
            Ok((normalized, phone_type))
        }
        Err(e) => {
            println!("Ошибка нормализации телефона {}: {}", phone, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn test_connection() -> Result<String, String> {
    let tester = ConnectionTester::new();
    println!("=== Запуск диагностики соединения ===");
    
    let result = tester.run_diagnostics().await;
    println!("=== Диагностика завершена ===");
    Ok(result)
}

fn main() {
    println!("=== LeadMiner Ultimate запускается ===");
    
    tauri::Builder::default()
        .manage(AppState {
            proxy_rotator: Arc::new(Mutex::new(ProxyRotator::new())),
            scraper: Arc::new(LeadScraper::new()),
            ai_service: Arc::new(AiService::new()),
        })
        .invoke_handler(tauri::generate_handler![
            start_scraping,
            update_proxies,
            normalize_phone_command,
            test_connection,
            fetch_models,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска LeadMiner Ultimate");
}