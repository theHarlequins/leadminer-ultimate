use sqlx::{sqlite::SqlitePool, Error};
use crate::scraper::EnrichedLead;

pub struct LeadRepository {
    pool: SqlitePool,
}

impl LeadRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn init_db(&self) -> Result<(), Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS leads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                phone TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                address TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_lead(&self, lead: &EnrichedLead) -> Result<bool, Error> {
        // First check if lead exists to avoid error on duplicate (or use ON CONFLICT IGNORE)
        // Using explicit check to return bool indicating if it was new
        if self.lead_exists(&lead.normalized_phone).await? {
            return Ok(false);
        }

        sqlx::query(
            "INSERT INTO leads (phone, name, address) VALUES (?, ?, ?)"
        )
        .bind(&lead.normalized_phone)
        .bind(&lead.name)
        .bind(&lead.address)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn lead_exists(&self, phone: &str) -> Result<bool, Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leads WHERE phone = ?")
            .bind(phone)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(count.0 > 0)
    }
}
