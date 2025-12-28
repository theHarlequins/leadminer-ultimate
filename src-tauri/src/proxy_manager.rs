use rand::Rng;
use reqwest::Proxy;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub requests_count: u32,
}

pub struct ProxyRotator {
    proxies: Vec<ProxyConfig>,
    current_index: usize,
    max_requests_per_proxy: u32,
}

impl ProxyRotator {
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
            current_index: 0,
            max_requests_per_proxy: 10, // Ротация после 10 запросов
        }
    }

    /// Обновляет список прокси
    pub fn update_proxies(&mut self, proxy_list: Vec<String>) {
        self.proxies = proxy_list
            .into_iter()
            .map(|url| ProxyConfig {
                url,
                username: None,
                password: None,
                requests_count: 0,
            })
            .collect();
        self.current_index = 0;
    }

    /// Добавляет прокси с аутентификацией
    pub fn add_proxy_with_auth(&mut self, url: String, username: String, password: String) {
        self.proxies.push(ProxyConfig {
            url,
            username: Some(username),
            password: Some(password),
            requests_count: 0,
        });
    }

    /// Получает следующий прокси для использования
    pub fn get_next_proxy(&mut self) -> Option<ProxyConfig> {
        if self.proxies.is_empty() {
            return None;
        }

        // Проверяем, нужно ли переключиться на следующий прокси
        if self.proxies[self.current_index].requests_count >= self.max_requests_per_proxy {
            self.proxies[self.current_index].requests_count = 0;
            self.current_index = (self.current_index + 1) % self.proxies.len();
        }

        let proxy = self.proxies[self.current_index].clone();
        self.proxies[self.current_index].requests_count += 1;

        Some(proxy)
    }

    /// Получает случайный прокси (для полной ротации)
    pub fn get_random_proxy(&mut self) -> Option<ProxyConfig> {
        if self.proxies.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.proxies.len());
        self.current_index = index;
        
        let proxy = self.proxies[index].clone();
        self.proxies[index].requests_count += 1;

        Some(proxy)
    }

    /// Создает reqwest Proxy из конфигурации
    pub fn create_reqwest_proxy(&self, config: &ProxyConfig) -> Result<Proxy, String> {
        let proxy = if let (Some(username), Some(password)) = (&config.username, &config.password) {
            Proxy::all(&config.url)
                .map_err(|e| format!("Invalid proxy URL: {}", e))?
                .basic_auth(username, password)
        } else {
            Proxy::all(&config.url)
                .map_err(|e| format!("Invalid proxy URL: {}", e))?
        };

        Ok(proxy)
    }

    /// Проверяет доступность прокси
    pub async fn test_proxy(&self, config: &ProxyConfig) -> bool {
        let client = match self.create_reqwest_proxy(config) {
            Ok(proxy) => reqwest::Client::builder()
                .proxy(proxy)
                .timeout(std::time::Duration::from_secs(10))
                .build(),
            Err(_) => return false,
        };

        match client {
            Ok(client) => {
                // Пробуем сделать запрос к простому сайту
                match client.get("https://httpbin.org/ip").send().await {
                    Ok(resp) => resp.status().is_success(),
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Получает количество доступных прокси
    pub fn get_proxy_count(&self) -> usize {
        self.proxies.len()
    }

    /// Генерирует случайный User-Agent
    pub fn get_random_user_agent(&self) -> String {
        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36 Edg/119.0.0.0",
        ];

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..user_agents.len());
        user_agents[index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_rotation() {
        let mut rotator = ProxyRotator::new();
        
        // Добавляем тестовые прокси
        rotator.update_proxies(vec![
            "http://proxy1:8080".to_string(),
            "http://proxy2:8080".to_string(),
            "http://proxy3:8080".to_string(),
        ]);

        assert_eq!(rotator.get_proxy_count(), 3);

        // Проверяем ротацию
        let first_proxy = rotator.get_next_proxy().unwrap();
        assert_eq!(first_proxy.url, "http://proxy1:8080");

        // Симулируем много запросов для проверки ротации
        for _ in 0..10 {
            rotator.get_next_proxy();
        }

        let second_proxy = rotator.get_next_proxy().unwrap();
        assert_eq!(second_proxy.url, "http://proxy2:8080");
    }

    #[test]
    fn test_user_agent_rotation() {
        let rotator = ProxyRotator::new();
        let ua1 = rotator.get_random_user_agent();
        let ua2 = rotator.get_random_user_agent();
        
        // User agents должны быть непустыми
        assert!(!ua1.is_empty());
        assert!(!ua2.is_empty());
        
        // Вероятность совпадения очень мала, но не проверяем строго
        // Главное что функция работает
    }
}