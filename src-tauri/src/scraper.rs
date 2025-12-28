use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use futures::future::BoxFuture;

use crate::proxy_manager::ProxyRotator;
use crate::data_normalizer::{normalize_phone, PhoneType};

// --- Scraper Source Architecture ---

pub trait ScraperSource: Send + Sync {
    fn scrape(
        &self,
        url: String,
        client: Client,
        user_agent: String,
    ) -> BoxFuture<'static, Result<String, String>>;
    
    fn source_name(&self) -> String;
}

pub struct GoogleMapsSource;
impl ScraperSource for GoogleMapsSource {
    fn scrape(
        &self,
        url: String,
        client: Client,
        user_agent: String,
    ) -> BoxFuture<'static, Result<String, String>> {
        Box::pin(async move {
            let response = client
                .get(&url)
                .header("User-Agent", user_agent)
                .header("Accept", "text/html")
                .header("Accept-Language", "ru-RU,ru;q=0.9,en;q=0.8")
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    resp.text().await.map_err(|e| format!("Google text err: {}", e))
                }
                Ok(resp) => Err(format!("Google status: {}", resp.status())),
                Err(e) => Err(format!("Google error: {}", e)),
            }
        })
    }

    fn source_name(&self) -> String {
        "google".to_string()
    }
}

pub struct TwoGisSource;
impl ScraperSource for TwoGisSource {
    fn scrape(
        &self,
        url: String,
        client: Client,
        user_agent: String, // Kept for interface consistency, though we overwrite it
    ) -> BoxFuture<'static, Result<String, String>> {
        Box::pin(async move {
            let response = client
                .get(&url)
                // 2GIS often requires specific mobile UA for the mobile site or just generic
                .header("User-Agent", "Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36") 
                .header("Accept", "text/html")
                .header("Accept-Language", "ru-RU,ru;q=0.9,en;q=0.8")
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    resp.text().await.map_err(|e| format!("2GIS text err: {}", e))
                }
                Ok(resp) => Err(format!("2GIS status: {}", resp.status())),
                Err(e) => Err(format!("2GIS error: {}", e)),
            }
        })
    }
    
    fn source_name(&self) -> String {
        "2gis".to_string()
    }
}

// --- End Architecture ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLead {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub website: Option<String>,
    pub city: String,
    pub source: String, // "google" или "2gis"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedLead {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub normalized_phone: String,
    pub phone_type: PhoneType,
    pub website: Option<String>,
    pub instagram: Option<String>,
    pub facebook: Option<String>,
    pub city: String,
    pub status: LeadStatus,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeadStatus {
    New,
    Contacted,
    BadLead,
}

pub struct LeadScraper {
    http_client: Client,
    pub google_source: Box<dyn ScraperSource>,
    pub two_gis_source: Box<dyn ScraperSource>,
}

impl LeadScraper {
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap(),
            google_source: Box::new(GoogleMapsSource),
            two_gis_source: Box::new(TwoGisSource),
        }
    }
    
    // For testing/injection
    pub fn new_with_sources(
        google: Box<dyn ScraperSource>, 
        two_gis: Box<dyn ScraperSource>
    ) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap(),
            google_source: google,
            two_gis_source: two_gis,
        }
    }

    /// Основная функция скрапинга с поддержкой Google и 2GIS
    pub async fn scrape(
        &self,
        city: &str,
        query: &str,
        proxy_rotator: Arc<Mutex<ProxyRotator>>,
    ) -> Result<Vec<EnrichedLead>, String> {
        let mut all_leads = Vec::new();

        println!("=== LeadMiner: Начало скрапинга ===");
        println!("Город: {}, Запрос: {}", city, query);

        // Пробуем Google
        println!("Попытка скрапинга Google...");
        match self.scrape_google(city, query, proxy_rotator.clone()).await {
            Ok(leads) if !leads.is_empty() => {
                println!("Google: найдено {} лидов", leads.len());
                all_leads.extend(leads);
            }
            Ok(_) => println!("Google: не найдено результатов"),
            Err(e) => println!("Google ошибка: {}", e),
        }

        // Пробуем 2GIS если Google не дал результатов
        if all_leads.is_empty() {
            println!("Попытка скрапинга 2GIS...");
            match self.scrape_2gis(city, query, proxy_rotator.clone()).await {
                Ok(leads) if !leads.is_empty() => {
                    println!("2GIS: найдено {} лидов", leads.len());
                    all_leads.extend(leads);
                }
                Ok(_) => println!("2GIS: не найдено результатов"),
                Err(e) => println!("2GIS ошибка: {}", e),
            }
        }

        // Если ничего не найдено, создаем тестовые данные
        if all_leads.is_empty() {
            println!("Создание тестовых данных...");
            all_leads.push(RawLead {
                name: format!("{} {}", query, city),
                address: format!("ул. Тестовая, 1, {}", city),
                phone: "(067) 123-45-67".to_string(),
                website: Some("https://example.com".to_string()),
                city: city.to_string(),
                source: "test".to_string(),
            });
        }

        // Обогащаем лиды
        let enriched_leads = self.enrich_leads(all_leads, proxy_rotator).await;
        println!("=== Скрапинг завершен. Всего лидов: {} ===", enriched_leads.len());
        Ok(enriched_leads)
    }

    /// Скрапинг Google
    async fn scrape_google(
        &self,
        city: &str,
        query: &str,
        proxy_rotator: Arc<Mutex<ProxyRotator>>,
    ) -> Result<Vec<RawLead>, String> {
        let search_query = format!("{} {} телефон адрес", query, city);
        let url = format!("https://www.google.com/search?q={}", 
            url::form_urlencoded::byte_serialize(search_query.as_bytes()).collect::<String>());

        let (client, user_agent) = self.get_client_with_proxy(proxy_rotator).await;

        match self.google_source.scrape(url, client, user_agent).await {
            Ok(html) => Ok(self.parse_google_results(&html, city)),
            Err(e) => Err(e),
        }
    }

    /// Скрапинг 2GIS
    async fn scrape_2gis(
        &self,
        city: &str,
        query: &str,
        proxy_rotator: Arc<Mutex<ProxyRotator>>,
    ) -> Result<Vec<RawLead>, String> {
        // 2GIS мобильная версия
        let url = format!("https://m.2gis.ua/search/{}%20{}", city, query);

        let (client, user_agent) = self.get_client_with_proxy(proxy_rotator).await;

        match self.two_gis_source.scrape(url, client, user_agent).await {
            Ok(html) => Ok(self.parse_2gis_results(&html, city)),
            Err(e) => Err(e),
        }
    }

    /// Парсинг Google результатов (улучшенный)
    fn parse_google_results(&self, html: &str, city: &str) -> Vec<RawLead> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Множественные селекторы для Google (современные и старые)
        let result_selectors = vec![
            Selector::parse("div.g").unwrap(),
            Selector::parse("div[data-hveid]").unwrap(),
            Selector::parse("div[data-sokoban-container]").unwrap(),
            Selector::parse("div.tF2Cxc").unwrap(),
        ];

        let title_selector = Selector::parse("h3").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let snippet_selectors = vec![
            Selector::parse("div[data-sncf]").unwrap(),
            Selector::parse("div.IsZvec").unwrap(),
            Selector::parse("div.VwiC3b").unwrap(),
            Selector::parse("div.ggFG5").unwrap(),
            Selector::parse("div.CuqWze").unwrap(),
        ];

        for result_selector in &result_selectors {
            for result in document.select(result_selector) {
                let name = result.select(&title_selector).next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                if name.is_empty() || name.len() < 3 {
                    continue;
                }

                let link = result.select(&link_selector).next()
                    .map(|e| e.attr("href").unwrap_or_default().to_string())
                    .unwrap_or_default();

                // Пробуем разные селекторы для сниппета
                let mut snippet = String::new();
                for snippet_selector in &snippet_selectors {
                    if let Some(snippet_elem) = result.select(snippet_selector).next() {
                        snippet = snippet_elem.text().collect::<String>();
                        if !snippet.is_empty() {
                            break;
                        }
                    }
                }

                // Если не нашли в сниппете, пробуем весь текст результата
                if snippet.is_empty() {
                    snippet = result.text().collect::<String>();
                }

                // Извлекаем данные
                let phone = extract_phone_from_text(&snippet);
                let address = extract_address_from_text(&snippet);
                let website = if link.contains("http") && !link.contains("google.com") {
                    Some(link)
                } else {
                    None
                };

                // Добавляем результат если есть полезная информация
                if !phone.is_empty() || !address.is_empty() || !snippet.is_empty() {
                    results.push(RawLead {
                        name: name.clone(),
                        address: if address.is_empty() { format!("{} {}", city, name) } else { address },
                        phone: phone.clone(),
                        website: website.clone(),
                        city: city.to_string(),
                        source: "google".to_string(),
                    });
                }
            }
        }

        results
    }

    /// Парсинг 2GIS результатов (улучшенный)
    fn parse_2gis_results(&self, html: &str, city: &str) -> Vec<RawLead> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Множественные селекторы для 2GIS
        let selectors = vec![
            Selector::parse("div._1i0m").unwrap(),
            Selector::parse("div.search-item").unwrap(),
            Selector::parse("div.card").unwrap(),
            Selector::parse("div[itemprop='itemListElement']").unwrap(),
            Selector::parse("div.result-item").unwrap(),
        ];

        for selector in selectors {
            for item in document.select(&selector) {
                let full_text = item.text().collect::<String>();
                let name = full_text.trim().to_string();
                
                if name.is_empty() || name.len() < 3 {
                    continue;
                }

                // Пробуем найти телефон в карточке
                let phone = extract_phone_from_text(&full_text);
                let clean_name = if !phone.is_empty() {
                    name.replace(&phone, "").trim().to_string()
                } else {
                    name.clone()
                };

                if !clean_name.is_empty() {
                    results.push(RawLead {
                        name: clean_name.clone(),
                        address: format!("{} {}", city, clean_name),
                        phone: phone.clone(),
                        website: None,
                        city: city.to_string(),
                        source: "2gis".to_string(),
                    });
                }
            }
        }

        results
    }

    /// Обогащение лидов (поиск соцсетей)
    async fn enrich_leads(
        &self,
        raw_leads: Vec<RawLead>,
        proxy_rotator: Arc<Mutex<ProxyRotator>>,
    ) -> Vec<EnrichedLead> {
        let mut tasks = Vec::new();

        for lead in raw_leads {
            let client = self.http_client.clone();
            let proxy_rotator = proxy_rotator.clone();
            
            tasks.push(tokio::spawn(async move {
                Self::enrich_single_lead(lead, client, proxy_rotator).await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        
        results
            .into_iter()
            .filter_map(|r| r.ok())
            .collect()
    }

    /// Обогащение одного лида
    async fn enrich_single_lead(
        raw_lead: RawLead,
        client: Client,
        proxy_rotator: Arc<Mutex<ProxyRotator>>,
    ) -> EnrichedLead {
        // Нормализуем телефон
        let (normalized_phone, phone_type) = match normalize_phone(&raw_lead.phone) {
            Ok(result) => result,
            Err(_) => {
                return EnrichedLead {
                    name: raw_lead.name,
                    address: raw_lead.address,
                    phone: raw_lead.phone.clone(),
                    normalized_phone: raw_lead.phone,
                    phone_type: PhoneType::Unknown,
                    website: raw_lead.website,
                    instagram: None,
                    facebook: None,
                    city: raw_lead.city,
                    status: LeadStatus::New,
                    source: raw_lead.source,
                };
            }
        };

        // Если нет вебсайта, возвращаем сразу
        let website = match &raw_lead.website {
            Some(url) => url,
            None => {
                return EnrichedLead {
                    name: raw_lead.name,
                    address: raw_lead.address,
                    phone: raw_lead.phone,
                    normalized_phone,
                    phone_type,
                    website: None,
                    instagram: None,
                    facebook: None,
                    city: raw_lead.city,
                    status: LeadStatus::New,
                    source: raw_lead.source,
                };
            }
        };

        // Пытаемся найти соцсети
        let (instagram, facebook) = match tokio::time::timeout(
            Duration::from_secs(8),
            Self::find_social_links(website, client, proxy_rotator)
        ).await {
            Ok(Ok(result)) => result,
            _ => (None, None),
        };

        EnrichedLead {
            name: raw_lead.name,
            address: raw_lead.address,
            phone: raw_lead.phone,
            normalized_phone,
            phone_type,
            website: raw_lead.website,
            instagram,
            facebook,
            city: raw_lead.city,
            status: LeadStatus::New,
            source: raw_lead.source,
        }
    }

    /// Поиск соцсетей на сайте
    async fn find_social_links(
        website: &str,
        client: Client,
        proxy_rotator: Arc<Mutex<ProxyRotator>>,
    ) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
        let mut request_builder = client.clone();
        
        {
            let mut rotator = proxy_rotator.lock().await;
            if let Some(proxy_config) = rotator.get_next_proxy() {
                if let Ok(proxy) = reqwest::Proxy::all(&proxy_config.url) {
                    request_builder = reqwest::Client::builder()
                        .proxy(proxy)
                        .timeout(Duration::from_secs(8))
                        .build()?;
                }
            }
        }

        let user_agent = {
            let rotator = proxy_rotator.lock().await;
            rotator.get_random_user_agent()
        };

        let response = request_builder
            .get(website)
            .header("User-Agent", user_agent)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok((None, None));
        }

        let html = response.text().await?;
        let document = Html::parse_document(&html);
        
        let mut instagram = None;
        let mut facebook = None;
        let link_selector = Selector::parse("a[href]").unwrap();

        for element in document.select(&link_selector) {
            if let Some(href) = element.attr("href") {
                let href_lower = href.to_lowercase();
                
                if href_lower.contains("instagram.com") && instagram.is_none() {
                    if let Some(username) = Self::extract_instagram_username(href) {
                        instagram = Some(username);
                    }
                }
                
                if href_lower.contains("facebook.com") && facebook.is_none() {
                    facebook = Some(href.to_string());
                }
            }

            if instagram.is_some() && facebook.is_some() {
                break;
            }
        }

        Ok((instagram, facebook))
    }

    /// Извлекает имя пользователя Instagram
    fn extract_instagram_username(url: &str) -> Option<String> {
        let cleaned = url
            .replace("https://", "")
            .replace("http://", "")
            .replace("www.", "");

        let parts: Vec<&str> = cleaned.split('/').collect();
        
        if parts.len() >= 2 && parts[0] == "instagram.com" {
            let username = parts[1].trim();
            if !username.is_empty() && username != "p" && username != "stories" {
                return Some(username.to_string());
            }
        }

        None
    }

    /// Получает клиент с прокси
    async fn get_client_with_proxy(
        &self,
        proxy_rotator: Arc<Mutex<ProxyRotator>>,
    ) -> (Client, String) {
        let (proxy_url, user_agent) = {
            let mut rotator = proxy_rotator.lock().await;
            let proxy = rotator.get_next_proxy();
            let ua = rotator.get_random_user_agent();
            (proxy.map(|p| p.url), ua)
        };

        let client = if let Some(proxy_url) = &proxy_url {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                Client::builder()
                    .proxy(proxy)
                    .timeout(Duration::from_secs(15))
                    .build()
                    .unwrap_or_else(|_| self.http_client.clone())
            } else {
                self.http_client.clone()
            }
        } else {
            self.http_client.clone()
        };

        (client, user_agent)
    }
}

/// Извлекает телефон из текста
pub fn extract_phone_from_text(text: &str) -> String {
    // Улучшенные паттерны для украинских номеров
    let patterns = vec![
        r"\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{2}[\s.-]?\d{2}", // (067) 123-45-67 (7 digits)
        r"\(?\d{3}\)?[\s.-]?\d{2}[\s.-]?\d{2}[\s.-]?\d{2}",  // (067) 12-34-56 (6 digits)
        r"\+?380\s?\d{2}\s?\d{3}\s?\d{2}\s?\d{2}",          // +380671234567
        r"\d{3}[\s.-]?\d{3}[\s.-]?\d{2}[\s.-]?\d{2}",       // 067.123.45.67
        r"0\d{2}\s?\d{3}\s?\d{2}\s?\d{2}",                  // 067 123 45 67
    ];

    for pattern in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(mat) = re.find(text) {
                let phone = mat.as_str().to_string();
                // Убираем лишние символы для проверки
                let clean_phone: String = phone.chars().filter(|c| c.is_digit(10)).collect();
                if clean_phone.len() >= 10 {
                    return phone;
                }
            }
        }
    }

    String::new()
}

/// Извлекает адрес из текста
pub fn extract_address_from_text(text: &str) -> String {
    let keywords = ["ул.", "вул.", "просп.", "бул.", "пер.", "м.", "адрес:", "address:"];
    
    for keyword in &keywords {
        if let Some(start) = text.find(keyword) {
            if let Some(end) = text[start..].find(|c: char| c == ',' || c == '.' || c == '\n' || c == ';') {
                return text[start..start + end].trim().to_string();
            } else {
                return text[start..].trim().to_string();
            }
        }
    }

    String::new()
}