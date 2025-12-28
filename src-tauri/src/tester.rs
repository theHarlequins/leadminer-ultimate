use reqwest::Client;
use std::time::Duration;

/// Тестер соединения и API
pub struct ApiTester {
    client: Client,
}

impl ApiTester {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    /// Тест соединения с интернетом
    pub async fn test_internet(&self) -> Result<bool, String> {
        match self.client.get("https://8.8.8.8").send().await {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("Нет интернета: {}", e)),
        }
    }

    /// Тест доступности Google
    pub async fn test_google(&self) -> Result<bool, String> {
        match self.client.get("https://www.google.com").send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(true)
                } else {
                    Err(format!("Google вернул статус: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Не удалось подключиться к Google: {}", e)),
        }
    }

    /// Тест доступности 2GIS
    pub async fn test_2gis(&self) -> Result<bool, String> {
        match self.client.get("https://2gis.ua").send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(true)
                } else {
                    Err(format!("2GIS вернул статус: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Не удалось подключиться к 2GIS: {}", e)),
        }
    }

    /// Тест прокси
    pub async fn test_proxy(&self, proxy_url: &str) -> Result<bool, String> {
        let proxy = match reqwest::Proxy::all(proxy_url) {
            Ok(p) => p,
            Err(e) => return Err(format!("Неверный формат прокси: {}", e)),
        };

        let client = match Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(10))
            .build() {
                Ok(c) => c,
                Err(e) => return Err(format!("Не удалось создать клиент: {}", e)),
            };

        match client.get("https://httpbin.org/ip").send().await {
            Ok(resp) if resp.status().is_success() => {
                let ip: serde_json::Value = resp.json().await.unwrap_or_default();
                Ok(true)
            }
            Ok(resp) => Err(format!("Прокси вернул статус: {}", resp.status())),
            Err(e) => Err(format!("Прокси не работает: {}", e)),
        }
    }

    /// Поиск в Google (тестовый)
    pub async fn search_google(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://www.google.com/search?q={}", 
            url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>());

        let response = self.client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let html = resp.text().await.unwrap_or_default();
                Ok(self.parse_google_results(&html))
            }
            Ok(resp) => Err(format!("Google вернул: {}", resp.status())),
            Err(e) => Err(format!("Ошибка поиска: {}", e)),
        }
    }

    /// Поиск в 2GIS (тестовый)
    pub async fn search_2gis(&self, city: &str, query: &str) -> Result<Vec<SearchResult>, String> {
        // 2GIS требует API ключ, но мы попробуем мобильную версию
        let url = format!("https://m.2gis.ua/search/{}%20{}", city, query);

        let response = self.client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Linux; Android 10; SM-G973F) AppleWebKit/537.36")
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let html = resp.text().await.unwrap_or_default();
                Ok(self.parse_2gis_results(&html))
            }
            Ok(resp) => Err(format!("2GIS вернул: {}", resp.status())),
            Err(e) => Err(format!("Ошибка 2GIS: {}", e)),
        }
    }

    /// Парсинг результатов Google
    fn parse_google_results(&self, html: &str) -> Vec<SearchResult> {
        use scraper::{Html, Selector};
        
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        let result_selector = Selector::parse("div.g").unwrap();
        let title_selector = Selector::parse("h3").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let snippet_selector = Selector::parse("div[data-sncf], div.IsZvec").unwrap();

        for result in document.select(&result_selector) {
            let title = result.select(&title_selector).next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let link = result.select(&link_selector).next()
                .map(|e| e.attr("href").unwrap_or_default().to_string())
                .unwrap_or_default();

            let snippet = result.select(&snippet_selector).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();

            if !title.is_empty() {
                results.push(SearchResult {
                    title,
                    link,
                    snippet,
                });
            }
        }

        results
    }

    /// Парсинг результатов 2GIS
    fn parse_2gis_results(&self, html: &str) -> Vec<SearchResult> {
        use scraper::{Html, Selector};
        
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // 2GIS мобильная версия имеет другую структуру
        let item_selector = Selector::parse("div._1i0m, div.search-item").unwrap();
        let title_selector = Selector::parse("div._1i0m, div.name").unwrap();

        for item in document.select(&item_selector) {
            let title = item.select(&title_selector).next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if !title.is_empty() {
                results.push(SearchResult {
                    title: title.clone(),
                    link: String::new(),
                    snippet: String::new(),
                });
            }
        }

        results
    }

    /// Комплексная проверка соединения
    pub async fn run_full_test(&self) -> Result<String, String> {
        let mut results = String::new();
        
        // Проверка интернета
        match self.test_internet().await {
            Ok(_) => {
                results.push_str("✅ Интернет: работает\n");
            }
            Err(e) => {
                results.push_str(&format!("❌ Интернет: {}\n", e));
                return Err(results);
            }
        }

        // Проверка Google
        match self.test_google().await {
            Ok(_) => {
                results.push_str("✅ Google: доступен\n");
            }
            Err(e) => {
                results.push_str(&format!("❌ Google: {}\n", e));
            }
        }

        // Проверка 2GIS
        match self.test_2gis().await {
            Ok(_) => {
                results.push_str("✅ 2GIS: доступен\n");
            }
            Err(e) => {
                results.push_str(&format!("❌ 2GIS: {}\n", e));
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_internet_connection() {
        let tester = ApiTester::new();
        let result = tester.test_internet().await;
        assert!(result.is_ok(), "Интернет должен работать");
    }

    #[tokio::test]
    async fn test_google_access() {
        let tester = ApiTester::new();
        let result = tester.test_google().await;
        assert!(result.is_ok(), "Google должен быть доступен");
    }

    #[tokio::test]
    async fn test_search_google() {
        let tester = ApiTester::new();
        let result = tester.search_google("обувь Киев").await;
        
        match result {
            Ok(results) => {
                assert!(!results.is_empty(), "Должны быть результаты");
                println!("Найдено {} результатов", results.len());
                for r in &results {
                    println!("- {}", r.title);
                }
            }
            Err(e) => {
                println!("Ошибка поиска: {}", e);
                // Не фейлим тест, т.к. Google может блокировать
            }
        }
    }
}