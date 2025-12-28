use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub prompt: String,
    pub completion: String,
    pub image: String,
    pub request: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub pricing: Pricing,
    pub context_length: u32,
    pub architecture: Architecture,
    pub top_provider: Option<Provider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Architecture {
    pub modality: String,
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub context_length: Option<u32>,
    pub max_completion_tokens: Option<u32>,
    pub is_moderated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

pub struct AiService {
    client: Client,
}

impl AiService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_models(&self, api_key: &str) -> Result<Vec<OpenRouterModel>, String> {
        let url = "https://openrouter.ai/api/v1/models";
        let resp = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
             return Err(format!("API Error: {}", resp.status()));
        }

        let body = resp.json::<ModelsResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(body.data)
    }
}
