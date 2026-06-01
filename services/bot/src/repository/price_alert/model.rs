pub type PriceAlertRow = (
    i64,
    i64,
    i64,
    String,
    f64,
    String,
    bool,
    String,
    Option<String>,
);

#[derive(Debug, Clone)]
pub struct PriceAlert {
    pub id: i64,
    pub user_id: i64,
    pub guild_id: i64,
    pub symbol: String,
    pub target_price: f64,
    pub direction: String,
    pub is_triggered: bool,
    pub created_at: String,
    pub triggered_at: Option<String>,
}

impl From<PriceAlertRow> for PriceAlert {
    fn from(row: PriceAlertRow) -> Self {
        Self {
            id: row.0,
            user_id: row.1,
            guild_id: row.2,
            symbol: row.3,
            target_price: row.4,
            direction: row.5,
            is_triggered: row.6,
            created_at: row.7,
            triggered_at: row.8,
        }
    }
}
