use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub prompt: String,
    pub completion: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub request: Option<String>,
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

    pub async fn analyze_lead(
        &self, 
        lead: &mut crate::scraper::EnrichedLead, 
        api_key: &str,
        model: &str
    ) -> Result<(), String> {
        println!("AI: Analyzing lead '{}' using model '{}'", lead.name, model);
        
        let prompt = format!(
            "Analyze this business lead and verify if it matches the user request. 
Lead: Name: {}, Description: {}, City: {}. 
Return a JSON with 'is_relevant' (bool) and 'category' (string).",
            lead.name, lead.address, lead.city
        );

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let resp = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("AI Request failed: {}", e))?;

        if !resp.status().is_success() {
             let error_text = resp.text().await.unwrap_or_default();
             return Err(format!("AI API Error: {}", error_text));
        }

        // For now, we just log success. In future we parse the JSON and update the lead.
        // We'll mark status as 'Contacted' to show it was processed, or keep 'New'
        // For demonstration of 'change', let's say we verified it
        // lead.status = crate::scraper::LeadStatus::Contacted; 

        println!("AI Analysis Success for {}", lead.name);
        Ok(())
    }

    /// Generate leads using AI when scraping fails
    pub async fn generate_leads(
        &self,
        city: &str,
        query: &str,
        api_key: &str,
        model: &str
    ) -> Result<Vec<crate::scraper::EnrichedLead>, String> {
        println!("AI: Generating leads for '{}' in '{}' using model '{}'", query, city, model);
        
        let prompt = format!(
            r#"Generate a list of 5 real businesses matching this search:
City: {}
Category/Query: {}

For each business, provide JSON in this exact format:
[
  {{"name": "Business Name", "address": "Street Address", "phone": "+380XXXXXXXXX", "website": "https://...", "category": "category"}}
]

Important: 
- Use realistic Ukrainian phone numbers starting with +380
- Include real street names for the city
- Only output valid JSON array, no other text"#,
            city, query
        );

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.7
        });

        let resp = self.client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("AI Request failed: {}", e))?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(format!("AI API Error: {}", error_text));
        }

        #[derive(Deserialize)]
        struct ChatChoice {
            message: ChatMessage,
        }
        #[derive(Deserialize)]
        struct ChatMessage {
            content: String,
        }
        #[derive(Deserialize)]
        struct ChatResponse {
            choices: Vec<ChatChoice>,
        }
        #[derive(Deserialize)]
        struct GeneratedLead {
            name: String,
            address: String,
            phone: String,
            website: Option<String>,
            #[serde(default)]
            category: Option<String>,
        }

        let chat_resp: ChatResponse = resp.json().await
            .map_err(|e| format!("Failed to parse AI response: {}", e))?;

        let content = chat_resp.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        println!("AI Response: {}", content);

        // Try to parse JSON from the response
        let json_start = content.find('[').unwrap_or(0);
        let json_end = content.rfind(']').map(|i| i + 1).unwrap_or(content.len());
        let json_str = &content[json_start..json_end];

        let generated: Vec<GeneratedLead> = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse generated leads: {} - Content: {}", e, json_str))?;

        let leads: Vec<crate::scraper::EnrichedLead> = generated
            .into_iter()
            .map(|g| crate::scraper::EnrichedLead {
                name: g.name,
                address: g.address.clone(),
                phone: g.phone.clone(),
                normalized_phone: g.phone,
                phone_type: crate::data_normalizer::PhoneType::Mobile,
                website: g.website,
                instagram: None,
                facebook: None,
                city: city.to_string(),
                status: crate::scraper::LeadStatus::New,
                source: "ai-generated".to_string(),
            })
            .collect();

        println!("AI: Generated {} leads", leads.len());
        Ok(leads)
    }
}
