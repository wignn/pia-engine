use serde::Serialize;
use uuid::Uuid;

/// Payment initiation response returned by the billing adapter.
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub status: String,
    pub message: String,
    pub redirect_url: Option<String>,
    pub order_id: Option<String>,
}

/// Starts a plan-upgrade payment flow.
///
/// The current adapter reports billing as unavailable until a payment provider is configured.
pub async fn create_payment(_user_id: Uuid, _plan_id: &str) -> PaymentResponse {
    PaymentResponse {
        status: "not_available".into(),
        message: "Payment integration (Midtrans) coming soon.".into(),
        redirect_url: None,
        order_id: None,
    }
}

/// Handles asynchronous payment-provider notifications.
pub async fn handle_notification(_body: &serde_json::Value) -> Result<(), String> {
    Err("Midtrans webhook handler not implemented yet".into())
}
