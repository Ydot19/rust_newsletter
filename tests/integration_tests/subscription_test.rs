mod common;

#[cfg(test)]
mod subscription_integration_tests {
    use std::collections::HashMap;

    use crate::common::helper::helper_functions;
    use axum::body;
    use axum::http::StatusCode;
    use axum::http::{header, Method, Request};
    use dotenvy::dotenv;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use http_body_util::BodyExt;
    use service::api;
    use service::port::models::{
        GetSubscriptionsResponse, RemoveSubscriptionResponse, Subscription,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn subscription_test() {
        // arrange
        dotenv().ok();
        let app = api::app();
        let email: String = SafeEmail().fake();
        let subscription = "new_york_times".to_string();
        let payload =
            helper_functions::new_create_subscription_request(subscription.clone(), email);
        let req = Request::builder()
            .method(Method::POST)
            .uri("/subscribe")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::Body::from(serde_json::to_string(&payload).unwrap()));

        // act
        let response = app.oneshot(req.unwrap()).await.unwrap();

        // assert
        assert_eq!(StatusCode::CREATED, response.status());
        let response_body = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();

        assert!(response_body.contains(&subscription))
    }

    #[tokio::test]
    async fn get_subscriptions_not_found_test() {
        // arrange
        dotenv().ok();
        let fake_email: String = SafeEmail().fake();
        let app = api::app();
        let payload = helper_functions::new_get_subscription_request(fake_email.clone());
        let req = Request::builder()
            .method(Method::GET)
            .uri("/subscriptions")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::Body::from(serde_json::to_string(&payload).unwrap()));
        // act
        let response = app.clone().oneshot(req.unwrap()).await.unwrap();

        // assert
        assert_eq!(StatusCode::NOT_FOUND, response.status());
        let body = helper_functions::body_to_bytes(response.into_body())
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains(fake_email.as_str()));
    }

    #[tokio::test]
    async fn remove_subscription_invalid_argument() {
        // arrange
        let app = api::app();
        let remove_subscription_request =
            helper_functions::new_remove_subscription_request("not_uuid".to_string());
        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/subscribe")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::Body::from(
                serde_json::to_string(&remove_subscription_request).unwrap(),
            ));

        let response = app.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(StatusCode::BAD_REQUEST, response.status());
    }

    #[tokio::test]
    async fn remove_subscription_not_found() {
        // arrange
        let subscription_id = Uuid::new_v4();
        let app = api::app();
        let remove_subscription_request =
            helper_functions::new_remove_subscription_request(subscription_id.to_string());
        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/subscribe")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::Body::from(
                serde_json::to_string(&remove_subscription_request).unwrap(),
            ));

        let response = app.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, response.status())
    }

    #[tokio::test]
    async fn get_subscriptions_test() {
        // arrange
        dotenv().ok();
        let fake_email: String = SafeEmail().fake();
        let app = api::app();
        let subscriptions: Vec<String> = vec![
            "new_york_times".to_string(),
            "columbus dispatch".to_string(),
        ];
        for sub in subscriptions.clone() {
            let req_body =
                helper_functions::new_create_subscription_request(sub, fake_email.clone());
            let req = Request::builder()
                .method(Method::POST)
                .uri("/subscribe")
                .header(header::CONTENT_TYPE, "application/json")
                .body(body::Body::from(serde_json::to_string(&req_body).unwrap()));

            let response = app.clone().oneshot(req.unwrap()).await.unwrap();
            assert_eq!(StatusCode::CREATED, response.status());
        }

        // prepare request
        let payload = helper_functions::new_get_subscription_request(fake_email.clone());
        let req = Request::builder()
            .method(Method::GET)
            .uri("/subscriptions")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::Body::from(serde_json::to_string(&payload).unwrap()));
        // act
        let response = app.clone().oneshot(req.unwrap()).await.unwrap();

        // assert
        assert_eq!(StatusCode::OK, response.status());
        let get_subscriptions_resp: GetSubscriptionsResponse =
            helper_functions::get_response(response.into_body())
                .await
                .unwrap();

        let subs: HashMap<String, Subscription> = get_subscriptions_resp
            .resp
            .into_iter()
            .map(|sub| (sub.clone().subscription_id, sub))
            .collect();
        assert_eq!(subs.len(), 2);

        // REMOVE subscription
        let sub_id = subs.keys().next().unwrap().clone();
        let remove_subscription_request =
            helper_functions::new_remove_subscription_request(sub_id.clone());
        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/subscribe")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::Body::from(
                serde_json::to_string(&remove_subscription_request).unwrap(),
            ));

        let response = app.clone().oneshot(req.unwrap()).await.unwrap();
        assert_eq!(StatusCode::NO_CONTENT, response.status());

        let remove_subscription_resp: RemoveSubscriptionResponse =
            helper_functions::get_response(response.into_body())
                .await
                .unwrap();

        assert_eq!(
            sub_id.clone(),
            remove_subscription_resp.subscription.subscription_id
        );

        // GET subscriptions
        let payload = helper_functions::new_get_subscription_request(fake_email.clone());
        let req = Request::builder()
            .method(Method::GET)
            .uri("/subscriptions")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body::Body::from(serde_json::to_string(&payload).unwrap()));
        // act
        let response = app.clone().oneshot(req.unwrap()).await.unwrap();

        // assert
        assert_eq!(StatusCode::OK, response.status());
        let get_subscriptions_resp: GetSubscriptionsResponse =
            helper_functions::get_response(response.into_body())
                .await
                .unwrap();

        let subs: HashMap<String, Subscription> = get_subscriptions_resp
            .resp
            .into_iter()
            .map(|sub| (sub.clone().subscription_id, sub))
            .collect();
        assert_eq!(subs.len(), 1);
    }
}
