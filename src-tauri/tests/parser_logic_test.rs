use mockall::predicate::*;
use mockall::mock;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;
use futures::future::BoxFuture;
use reqwest::Client;

use leadminer_ultimate::proxy_manager::ProxyRotator;
use leadminer_ultimate::scraper::{LeadScraper, ScraperSource};

// Define mock
mock! {
    pub Source {}
    
    impl ScraperSource for Source {
        fn scrape<'a>(
            &'a self,
            url: &'a str,
            client: &'a Client,
            user_agent: &'a str,
        ) -> BoxFuture<'a, Result<String, String>>;
        
        fn source_name(&self) -> String;
    }
}

// Ensure MockSource is Send + Sync
unsafe impl Send for MockSource {}
unsafe impl Sync for MockSource {}

fn load_fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    std::fs::read_to_string(path).expect("Failed to read fixture")
}

#[tokio::test]
async fn test_google_parser_logic() {
    // Arrange
    let html = load_fixture("google_maps_sample.html");
    
    let mut mock_google = MockSource::new();
    mock_google
        .expect_scrape()
        .returning(move |_, _, _| {
            let html = html.clone();
            Box::pin(async move { Ok(html) })
        });
    mock_google
        .expect_source_name()
        .returning(|| "google".to_string());
        
    let mut mock_2gis = MockSource::new();
    mock_2gis.expect_scrape().returning(|_, _, _| Box::pin(async { Ok("".to_string()) }));
    mock_2gis.expect_source_name().returning(|| "2gis".to_string());

    let scraper = LeadScraper::new_with_sources(
        Box::new(mock_google),
        Box::new(mock_2gis)
    );
    
    let rotator = Arc::new(Mutex::new(ProxyRotator::new()));

    // Act
    let results = scraper.scrape("Киев", "Pizza", rotator).await.unwrap();

    // Assert
    assert!(!results.is_empty(), "Should find results");
    
    let pizza = results.iter().find(|r| r.name.contains("Pizza Bella"));
    assert!(pizza.is_some());
    assert!(pizza.unwrap().phone.contains("044"));
    
    let sushi = results.iter().find(|r| r.name.contains("Sushi Master"));
    assert!(sushi.is_some());
    assert!(sushi.unwrap().phone.contains("067 999 88 77"));
}

#[tokio::test]
async fn test_2gis_parser_logic() {
    // Arrange
    let html = load_fixture("2gis_sample.html");
    
    let mut mock_google = MockSource::new();
    mock_google
        .expect_scrape()
        .returning(|_, _, _| Box::pin(async { Ok("".to_string()) })); // Google returns empty
    mock_google
        .expect_source_name()
        .returning(|| "google".to_string());

    let mut mock_2gis = MockSource::new();
    mock_2gis
        .expect_scrape()
        .returning(move |_, _, _| {
            let html = html.clone();
            Box::pin(async move { Ok(html) })
        });
     mock_2gis
        .expect_source_name()
        .returning(|| "2gis".to_string());

    let scraper = LeadScraper::new_with_sources(
        Box::new(mock_google),
        Box::new(mock_2gis)
    );
    
    let rotator = Arc::new(Mutex::new(ProxyRotator::new()));

    // Act
    // We scrape Google (returns empty) -> then 2GIS (returns results)
    let results = scraper.scrape("Львов", "Coffee", rotator).await.unwrap();

    // Assert
    assert!(!results.is_empty(), "Should find results from 2GIS");
    
    let coffee = results.iter().find(|r| r.name.contains("Coffee House"));
    assert!(coffee.is_some());
    assert!(coffee.unwrap().phone.contains("093"));
}
