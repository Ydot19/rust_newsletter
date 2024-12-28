#[cfg(test)]
mod test {
    use crate::common::helper::helper_functions;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use std::env;

    fn get_base_url() -> String {
        let password = env::var("BACKEND_URL");
        assert!(password.is_ok());
        password.unwrap()
    }

    #[tokio::test]
    async fn subscription() {
        // arrange
        dotenvy::dotenv().ok();
        let email: String = SafeEmail().fake();
        let subscription = "new_york_times".to_string();
        let payload =
            helper_functions::new_create_subscription_request(subscription.clone(), email);
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        // act
        let response = client
            .post(format!("{}/subscribe", base_url))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .expect("Failed to send request");

        // assert
        let status = response.status();
        println!("Status: {}", status);
        let body = response.text();
        let body = body.await.unwrap();
        println!("Message: {}", body);
        assert!(status.is_success())
    }
}
