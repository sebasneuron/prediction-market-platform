use std::sync::Arc;

use bloom::{ASMS, BloomFilter};
use db_service::schema::users::User;
use parking_lot::RwLock;
use utility_helpers::log_info;
use uuid::Uuid;

#[derive(Clone)]
pub struct BloomFilterWrapper {
    /// A thread-safe wrapper around a Bloom filter.
    ///
    /// **NOTE:** Direct DB update does not update the filter, so it is critical to prevent direct db updates for user
    filter: Arc<RwLock<BloomFilter>>,
}

impl BloomFilterWrapper {
    pub async fn new(db_pool: &sqlx::PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        let mut filter = BloomFilter::with_rate(0.01, 1_000_000_000); // 1% false positive rate, 1 billion items

        let user_ids = User::get_all_user_ids(db_pool)
            .await
            .map_err(|e| format!("Failed to fetch user IDs: {}", e))?;

        for id in &user_ids {
            filter.insert(id);
        }

        log_info!("Bloom filter initialized with {} user IDs", user_ids.len());

        Ok(BloomFilterWrapper {
            filter: Arc::new(RwLock::new(filter)),
        })
    }
    pub fn contains(&self, id: &Uuid) -> bool {
        self.filter.read().contains(id)
    }

    pub fn insert(&self, id: &Uuid) {
        self.filter.write().insert(id);
    }
}
