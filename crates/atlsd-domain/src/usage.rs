use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UsageEvent {
    pub user_id: Uuid,
    pub api_key_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub status_code: i32,
    pub response_ms: i32,
}
