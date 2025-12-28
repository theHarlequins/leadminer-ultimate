use sqlx::sqlite::SqlitePoolOptions;
use leadminer_ultimate::db::LeadRepository;
use leadminer_ultimate::scraper::{EnrichedLead, LeadStatus};
use leadminer_ultimate::data_normalizer::PhoneType;

#[tokio::test]
async fn test_save_and_retrieve_lead() {
    // Arrange
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    let repo = LeadRepository::new(pool);
    repo.init_db().await.expect("Failed to init DB");

    let lead = EnrichedLead {
        name: "Test Company".to_string(),
        address: "Test Address".to_string(),
        phone: "(067) 111-22-33".to_string(),
        normalized_phone: "380671112233".to_string(),
        phone_type: PhoneType::Mobile,
        website: None,
        instagram: None,
        facebook: None,
        city: "Kyiv".to_string(),
        status: LeadStatus::New,
        source: "google".to_string(),
    };

    // Act
    let saved = repo.save_lead(&lead).await.expect("Failed to save lead");

    // Assert
    assert!(saved, "Should have returned true for new lead");
    
    // Verify existence
    let exists = repo.lead_exists(&lead.normalized_phone).await.expect("Failed to check existence");
    assert!(exists, "Lead should exist in DB");
}

#[tokio::test]
async fn test_deduplication() {
    // Arrange
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    let repo = LeadRepository::new(pool);
    repo.init_db().await.expect("Failed to init DB");

    let lead = EnrichedLead {
        name: "Test Company".to_string(),
        address: "Test Address".to_string(),
        phone: "(067) 111-22-33".to_string(),
        normalized_phone: "380671112233".to_string(),
        phone_type: PhoneType::Mobile,
        website: None,
        instagram: None,
        facebook: None,
        city: "Kyiv".to_string(),
        status: LeadStatus::New,
        source: "google".to_string(),
    };

    // Act 1: Save first time
    let first_save = repo.save_lead(&lead).await.expect("Failed to save lead first time");
    assert!(first_save);

    // Act 2: Try save again (duplicate)
    let second_save = repo.save_lead(&lead).await.expect("Failed to save lead second time");

    // Assert
    assert!(!second_save, "Should have returned false for duplicate lead");
}
