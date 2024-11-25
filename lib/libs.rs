mod adapter;
mod domain;
pub mod port;
mod routes;

pub mod api {
    use crate::adapter::{configuration::DatabaseConfiguration, repository::Repository};
    use crate::{adapter, routes};
    use axum::extract::{MatchedPath, Request};
    use axum::response::Response;
    use axum::{
        routing::{get, post},
        Router,
    };
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tower_http::trace::TraceLayer;
    use tracing::{error, info, info_span, warn, Span};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    pub fn app() -> Router {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .init();

        let cfg = DatabaseConfiguration::new();
        let repo = Repository::new(&cfg);
        if repo.is_err() {
            panic!("failed to instantiate repo")
        }
        let repo = repo.unwrap();
        let repo: Arc<Mutex<dyn adapter::repository::SubscriptionRepository + Send + Sync>> =
            Arc::new(Mutex::new(repo));
        let application = routes::app::Application::new(repo);
        let application = Arc::new(application);
        Router::new()
            .route("/echo", get(routes::echo::handler))
            .route("/health_check", get(routes::health_check::handler))
            .route(
                "/subscribe",
                post(routes::subscriptions::create_subscription_handler)
                    .delete(routes::subscriptions::remove_subscription_handler),
            )
            .route(
                "/subscriptions",
                get(routes::subscriptions::get_subscription_handler),
            )
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<_>| {
                        let matched_path = request
                            .extensions()
                            .get::<MatchedPath>()
                            .map(MatchedPath::as_str);

                        info_span!(
                            "http_request",
                            method = ?request.method(),
                            matched_path,
                            uri = ?request.uri(),
                            some_other_field = tracing::field::Empty,
                        )
                    })
                    .on_request(|request: &Request<_>, span: &Span| {
                        info!(
                            parent: span,
                                "Started {} request to {}",
                                request.method(),
                                request.uri(),
                        )
                    })
                    .on_response(|response: &Response, latency: Duration, span: &Span| {
                        let status = response.status();
                        let latency_ms = latency.as_millis();

                        if status.is_success() {
                            info!(
                                parent: span,
                                    "completed request (status={}, latency={}ms)",
                                    status,
                                    latency_ms,
                            );
                        } else if status.is_server_error() {
                            error!(
                                parent: span,
                                    "completed request (status={}, latency={}ms)",
                                    status,
                                    latency_ms,
                            )
                        } else {
                            warn!(
                                parent: span,
                                "request completed with non-200 status (status={}, latency={})",
                                status,
                                latency_ms
                            );

                            if let Some(content_type) = response.headers().get("content-type") {
                                info!(
                                    parent: span,
                                        "response content-type: {:?}",
                                        content_type,
                                );
                            }
                        }
                    }),
            )
            .layer(axum::Extension(application))
    }
}
