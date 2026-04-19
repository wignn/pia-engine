use serde::Serialize;
use uuid::Uuid;

/// Stub billing response — ready for Midtrans integration later.
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub status: String,
    pub message: String,
    pub redirect_url: Option<String>,
    pub order_id: Option<String>,
}

/// Stub: create a payment for plan upgrade.
/// Will be replaced with Midtrans Snap API integration.
pub async fn create_payment(_user_id: Uuid, _plan_id: &str) -> PaymentResponse {
    PaymentResponse {
        status: "not_available".into(),
        message: "Payment integration (Midtrans) coming soon.".into(),
        redirect_url: None,
        order_id: None,
    }
}

/// Stub: handle payment notification webhook from Midtrans.
pub async fn handle_notification(_body: &serde_json::Value) -> Result<(), String> {
    Err("Midtrans webhook handler not implemented yet".into())
}
