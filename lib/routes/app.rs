use std::sync::Arc;
use std::sync::Mutex;

use crate::adapter::repository;

#[derive(Clone)]
pub struct Application {
    pub repo: Arc<Mutex<dyn repository::SubscriptionRepository + Send + Sync>>,
}

impl Application {
    pub fn new(repo: Arc<Mutex<dyn repository::SubscriptionRepository + Send + Sync>>) -> Self {
        Self { repo }
    }
}
