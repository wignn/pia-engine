use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcMarketPricesRequest {
    pub symbols: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcMarketPricesResponse {
    pub ok: bool,
    pub raw_json: String,
}
