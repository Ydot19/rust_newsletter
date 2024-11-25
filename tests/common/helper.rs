#[cfg(test)]
pub mod helper_functions {
    use axum::body::Body;
    use bytes::Bytes;
    use futures_util::stream::StreamExt;
    use serde::Deserialize;
    use service::port::models::{
        CreateSubscriptionRequest, GetSubscriptionRequest, RemoveSubscriptionRequest,
    };
    pub fn new_create_subscription_request(
        name: String,
        email: String,
    ) -> CreateSubscriptionRequest {
        CreateSubscriptionRequest { email, name }
    }

    pub fn new_get_subscription_request(email: String) -> GetSubscriptionRequest {
        GetSubscriptionRequest { email }
    }

    pub fn new_remove_subscription_request(subscription_id: String) -> RemoveSubscriptionRequest {
        RemoveSubscriptionRequest { subscription_id }
    }

    pub async fn body_to_bytes(body: Body) -> Result<bytes::Bytes, axum::Error> {
        let mut bytes = Vec::new();
        let mut stream = body.into_data_stream();
        while let Some(chunk) = stream.next().await {
            bytes.extend_from_slice(&chunk?);
        }
        Ok(Bytes::from(bytes))
    }

    pub async fn get_response<T>(body: Body) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Execute the async operation synchronously and handle the Result
        let body_bytes = tokio::task::spawn_blocking(|| body_to_bytes(body))
            .await
            .unwrap();
        let body_bytes = body_bytes.await.unwrap();

        // deserialize
        let res: T = serde_json::from_slice(&body_bytes)?;
        Ok(res)
    }
}
