#[cfg(test)]
mod tests {
    use crate::ai::{AiService, OpenRouterModel};
    use crate::scraper::{EnrichedLead, LeadStatus};
    use crate::data_normalizer::PhoneType;

    #[tokio::test]
    async fn test_ai_connection_and_analysis() {
        // Credentials provided by user for testing
        let api_key = "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";
        let model_id = "google/gemini-2.0-flash-exp:free";

        let service = AiService::new();

        // 1. Test Fetch Models
        println!("Testing fetch_models...");
        match service.fetch_models(api_key).await {
            Ok(models) => {
                assert!(!models.is_empty(), "Models list should not be empty");
                println!("Success! Fetched {} models.", models.len());
                
                // Verify our specific model exists (optional, as free models might rotate, but good to check)
                let model_exists = models.iter().any(|m| m.id == model_id);
                println!("Model '{}' availability in list: {}", model_id, model_exists);
            },
            Err(e) => panic!("Failed to fetch models: {}", e),
        }

        // 2. Test Analyze Lead
        println!("Testing analyze_lead...");
        let mut lead = EnrichedLead {
            name: "Coffee House Test".to_string(),
            address: "123 Test St, Test City".to_string(),
            phone: "1234567890".to_string(),
            normalized_phone: "1234567890".to_string(),
            phone_type: PhoneType::Mobile,
            website: Some("https://example.com".to_string()),
            instagram: None,
            facebook: None,
            city: "Test City".to_string(),
            status: LeadStatus::New,
            source: "test".to_string(),
        };

        match service.analyze_lead(&mut lead, api_key, model_id).await {
            Ok(_) => println!("Success! Lead analyzed."),
            Err(e) => panic!("Failed to analyze lead: {}", e),
        }
    }
}
