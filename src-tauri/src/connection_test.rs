use reqwest::Client;
use std::time::Duration;

/// Тестер соединения для диагностики проблем
pub struct ConnectionTester {
    client: Client,
}

impl ConnectionTester {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// Проверяет интернет соединение
    pub async fn check_internet(&self) -> Result<String, String> {
        match self.client.get("https://8.8.8.8").send().await {
            Ok(_) => Ok("✅ Интернет работает".to_string()),
            Err(e) => Err(format!("❌ Нет интернета: {}", e)),
        }
    }

    /// Проверяет доступность Google
    pub async fn check_google(&self) -> Result<String, String> {
        match self.client.get("https://www.google.com").send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok("✅ Google доступен".to_string())
                } else {
                    Err(format!("❌ Google вернул статус: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("❌ Ошибка Google: {}", e)),
        }
    }

    /// Проверяет доступность 2GIS
    pub async fn check_2gis(&self) -> Result<String, String> {
        match self.client.get("https://2gis.ua").send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok("✅ 2GIS доступен".to_string())
                } else {
                    Err(format!("❌ 2GIS вернул статус: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("❌ Ошибка 2GIS: {}", e)),
        }
    }

    /// Комплексная проверка
    pub async fn run_diagnostics(&self) -> String {
        let mut report = String::new();
        report.push_str("=== LeadMiner Diagnostics ===\n\n");

        // Проверка интернета
        match self.check_internet().await {
            Ok(msg) => report.push_str(&format!("{}\n", msg)),
            Err(msg) => report.push_str(&format!("{}\n", msg)),
        }

        // Проверка Google
        match self.check_google().await {
            Ok(msg) => report.push_str(&format!("{}\n", msg)),
            Err(msg) => report.push_str(&format!("{}\n", msg)),
        }

        // Проверка 2GIS
        match self.check_2gis().await {
            Ok(msg) => report.push_str(&format!("{}\n", msg)),
            Err(msg) => report.push_str(&format!("{}\n", msg)),
        }

        report.push_str("\n=== Рекомендации ===\n");
        report.push_str("1. Проверьте интернет соединение\n");
        report.push_str("2. Убедитесь, что Google и 2GIS доступны\n");
        report.push_str("3. Настройте прокси если нужно\n");
        report.push_str("4. Попробуйте перезапустить приложение\n");

        report
    }
}