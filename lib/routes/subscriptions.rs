use crate::domain::errors::{self as domain_errors, DomainError};
use crate::model::models::{self as api_models};
use axum::http::header::CONTENT_TYPE;
use axum::http::Response;
use axum::response::IntoResponse;
use axum::{http::StatusCode, Extension, Json};
use std::str::FromStr;
use std::{sync::Arc, time::SystemTime};
use uuid::Uuid;

fn get_subscriptions(
    req: api_models::GetSubscriptionRequest,
    repo: &mut (dyn crate::adapter::repository::SubscriptionRepository + Send + Sync),
) -> Result<api_models::GetSubscriptionsResponse, domain_errors::DomainError> {
    let resp = repo.get_subscriptions(req.email.clone());
    if resp.is_empty() {
        Err(DomainError::NotFound(format!(
            "no subscriptions found for email: {}",
            req.email
        )))
    } else {
        Ok(api_models::GetSubscriptionsResponse { resp })
    }
}

fn remove_subscription(
    req: api_models::RemoveSubscriptionRequest,
    repo: &mut (dyn crate::adapter::repository::SubscriptionRepository + Send + Sync),
) -> Result<api_models::RemoveSubscriptionResponse, DomainError> {
    let id = Uuid::from_str(req.subscription_id.as_str());
    if id.is_err() {
        return Err(DomainError::Validation {
            field: "subscription_id".to_string(),
            message: "Id must be a uuid".to_string(),
        });
    }
    let res = repo.remove_subscription(id.unwrap());
    match res {
        Err(err) => Err(err),
        Ok(res) => Ok(api_models::RemoveSubscriptionResponse { subscription: res }),
    }
}

pub(crate) async fn create_subscription_handler(
    Extension(app): axum::Extension<Arc<super::app::Application>>,
    arg: Json<api_models::CreateSubscriptionRequest>,
) -> (StatusCode, Json<api_models::SubscriptionResponse>) {
    let repo = app.repo.clone();
    let repo = repo.lock().unwrap();
    let resp = repo.add_subscription(
        arg.name.as_str().to_string(),
        arg.email.as_str().to_string(),
        SystemTime::now(),
    );
    match resp {
        Ok(_) => (
            StatusCode::CREATED,
            Json(api_models::SubscriptionResponse {
                message: format!(
                    "Subscription created for user: {} with email: {}",
                    arg.name, arg.email
                ),
            }),
        ),
        Err(_) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(api_models::SubscriptionResponse {
                message: "Failed to add subscription".to_string(),
            }),
        ),
    }
}

pub(crate) async fn get_subscription_handler(
    Extension(app): axum::Extension<Arc<super::app::Application>>,
    arg: Json<api_models::GetSubscriptionRequest>,
) -> axum::response::Response {
    let repo = app.repo.clone();
    let mut repo = repo.lock().unwrap();
    let res = get_subscriptions(arg.0, &mut *repo);
    match res {
        Ok(t) => {
            let json_body = serde_json::to_string(&t).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(json_body))
                .unwrap()
        }
        Err(e) => domain_errors::error_to_response(e).into_response(),
    }
}

pub(crate) async fn remove_subscription_handler(
    Extension(app): axum::Extension<Arc<super::app::Application>>,
    arg: Json<api_models::RemoveSubscriptionRequest>,
) -> axum::response::Response {
    let repo = app.repo.clone();
    let mut repo = repo.lock().unwrap();
    let res = remove_subscription(arg.0, &mut *repo);
    match res {
        Ok(t) => {
            let json_body = serde_json::to_string(&t).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(StatusCode::NO_CONTENT)
                .header(CONTENT_TYPE, "application/json")
                .body(axum::body::Body::from(json_body))
                .unwrap()
        }
        Err(e) => domain_errors::error_to_response(e).into_response(),
    }
}
