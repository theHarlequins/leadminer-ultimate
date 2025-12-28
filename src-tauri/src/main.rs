#
![cfg_attr(not(debug_assertions)
, windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;

// Импортируем модули из lib.rs
use leadminer_ultimate::{
    proxy_manager::ProxyRotator,
    data_normalizer::{normalize_phone, PhoneType},
    scraper::{EnrichedLead, LeadScraper},
    connection_test::ConnectionTester,
    ai::{AiService, OpenRouterModel},
    settings::{Settings, SettingsStore},
};

// Глобальное состояние приложения
struct AppState {
    proxy_rotator: Arc<Mutex<ProxyRotator>>,
    scraper: Arc<LeadScraper>,
    ai_service: Arc<AiService>,
    settings_store: Arc<SettingsStore>,
}

#[tauri::command]
async fn fetch_models(
    api_key: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<OpenRouterModel>, String> {
    state.ai_service.fetch_models(&api_key).await
}

#[tauri::command]
async fn save_settings(
    settings: Settings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.settings_store.save(settings)?;
    Ok(())
}

#[tauri::command]
async fn get_settings(
    state: tauri::State<'_, AppState>,
) -> Result<Settings, String> {
    Ok(state.settings_store.get())
}

#[tauri::command]
async fn start_scraping(
    city: String,
    query: String,
    api_key: Option<String>,
    model_id: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<EnrichedLead>, String> {
    // ... code ...
    println!("=== LeadMiner: Запрос на скрапинг ===");
    println!("Город: {}", city);
    println!("Запрос: {}", query);
    println!("Время: {:?}", std::time::SystemTime::now());
    
    let scraper = state.scraper.clone();
    let proxy_rotator = state.proxy_rotator.clone();
    let ai_service = state.ai_service.clone();
    
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
        Ok(mut leads) => {
            println!("=== Успешно! Найдено {} лидов ===", leads.len());
            
            // Если есть API ключ, запускаем AI анализ
            if let Some(key) = api_key {
                if !key.is_empty() {
                    let model = model_id.unwrap_or_else(|| "google/gemini-2.0-flash-exp:free".to_string());
                    println!("=== Запуск AI анализа (Model: {}) ===", model);
                    
                    for lead in &mut leads {
                        match ai_service.analyze_lead(lead, &key, &model).await {
                            Ok(_) => println!("AI: Анализ для {} завершен", lead.name),
                            Err(e) => println!("AI: Ошибка анализа для {}: {}", lead.name, e),
                        }
                    }
                } else {
                     println!("=== AI анализ пропущен (нет ключа) ===");
                }
            } else {
                println!("=== AI анализ пропущен (нет ключа) ===");
            }
            
            // Авто-сохранение в CSV
            if let Err(e) = save_leads_to_csv(&leads, &city, &app_handle) {
                println!("Ошибка сохранения CSV: {}", e);
            }

            Ok(leads)
        }
        Err(e) => {
            println!("=== Ошибка скрапинга: {} ===", e);
            Err(format!("Не удалось выполнить скрапинг: {}", e))
        }
    }
}

fn save_leads_to_csv(leads: &[EnrichedLead], city: &str, app: &tauri::AppHandle) -> std::io::Result<()> {
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    let dir = app.path().app_data_dir().unwrap().join("exports");
    std::fs::create_dir_all(&dir)?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let filename = format!("leads_{}_{}.csv", city, timestamp);
    let path = dir.join(filename);

    let mut file = std::fs::File::create(&path)?;
    writeln!(file, "Name,Phone,Type,City,Address,Website,Status,Source")?;

    for lead in leads {
        writeln!(file, "{},{},{:?},{},{},{},{:?},{}", 
            escape_csv(&lead.name),
            escape_csv(&lead.normalized_phone),
            lead.phone_type,
            escape_csv(&lead.city),
            escape_csv(&lead.address),
            escape_csv(&lead.website.clone().unwrap_or_default()),
            lead.status,
            lead.source
        )?;
    }
    
    println!("=== Результаты сохранены в: {:?} ===", path);
    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace("\"", "\"\""))
    } else {
        s.to_string()
    }
}

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
        .setup(|app| {
             let settings_store = SettingsStore::new(app.handle());
             app.manage(AppState {
                proxy_rotator: Arc::new(Mutex::new(ProxyRotator::new())),
                scraper: Arc::new(LeadScraper::new()),
                ai_service: Arc::new(AiService::new()),
                settings_store: Arc::new(settings_store),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_scraping,
            update_proxies,
            normalize_phone_command,
            test_connection,
            fetch_models,
            save_settings,
            get_settings
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска LeadMiner Ultimate");
}