use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub email: Option<String>,
    pub subscription_id: String,
    pub subscription_name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub subscribe_since: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CreateSubscriptionRequest {
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetSubscriptionRequest {
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetSubscriptionsResponse {
    pub resp: Vec<Subscription>,
}

#[derive(Serialize)]
pub struct SubscriptionResponse {
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct RemoveSubscriptionRequest {
    pub subscription_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct RemoveSubscriptionResponse {
    pub subscription: Subscription,
}
