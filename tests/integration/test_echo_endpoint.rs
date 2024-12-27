use axum::http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use service::api;
use tower::ServiceExt;

#[tokio::test]
async fn echo_endpoint() {
    // arrange
    let app = api::app();

    // act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/echo?name=ydot19")
                .method(Method::GET)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // assert
    assert_eq!(StatusCode::OK, response.status());
    let b = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    let bs = String::from_utf8(b).unwrap();
    assert!(bs.contains("ydot19"));
}
