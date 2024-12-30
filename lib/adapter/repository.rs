use std::time;

use crate::domain::errors::DomainError;
use crate::model::models as api_models;

use super::schema::subscriptions;
use super::{configuration::DatabaseConfiguration, models::Subscription};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;

pub trait SubscriptionRepository {
    fn add_subscription(
        &self,
        name: String,
        email: String,
        subcribed_at: time::SystemTime,
    ) -> Result<api_models::Subscription, DomainError>;
    fn get_subscriptions(&mut self, email: String) -> Vec<api_models::Subscription>;
    fn remove_subscription(&mut self, id: Uuid) -> Result<api_models::Subscription, DomainError>;
}

#[derive(Clone)]
pub struct Repository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

pub(super) fn connection_pool(
    cfg: &DatabaseConfiguration,
) -> Pool<ConnectionManager<PgConnection>> {
    let mut retries = 3;
    let mut delay = std::time::Duration::from_secs(1);
    std::thread::sleep(delay);
    loop {
        let manager = ConnectionManager::<PgConnection>::new(cfg.connection_string());
        match Pool::builder()
            .max_size(10)
            .test_on_check_out(true)
            .build(manager)
        {
            Ok(pool) => return pool,
            Err(e) => {
                if retries == 0 {
                    panic!("Failed to create connection pool after retries: {}", e);
                }

                tracing::warn!(
                    "Failed to create connection pool: {}, retrying in {:?}",
                    e,
                    delay
                );

                std::thread::sleep(delay);
                retries -= 1;
                delay *= 2; // exponential backoff
            }
        }
    }
}

impl Repository {
    pub fn new(cfg: &DatabaseConfiguration) -> Result<Self, diesel::r2d2::PoolError> {
        Ok(Self {
            pool: connection_pool(cfg),
        })
    }
}

impl SubscriptionRepository for Repository {
    fn add_subscription(
        &self,
        name: String,
        email: String,
        subscribed_at: time::SystemTime,
    ) -> Result<api_models::Subscription, DomainError> {
        let id = Uuid::new_v4();
        let subscription = Subscription {
            id,
            email,
            name,
            subscribed_at: subscribed_at.into(),
        };

        let pool = &mut self.pool.get().map_err(|err| {
            DomainError::Internal(format!(
                "Database Error! Failed to get connection (Err={})",
                err
            ))
        })?;

        let res = diesel::insert_into(subscriptions::table)
            .values(&subscription)
            .execute(pool)
            .map(|_| subscription)
            .map_err(|err| {
                DomainError::Internal(format!(
                    "failed to store new subscription (Error = {})",
                    err
                ))
            });

        match res {
            Ok(sub) => Ok(api_models::Subscription {
                email: None,
                subscription_id: sub.id.to_string(),
                subscription_name: sub.name,
                subscribe_since: sub.subscribed_at,
            }),
            Err(err) => Err(err),
        }
    }

    fn get_subscriptions(&mut self, email: String) -> Vec<api_models::Subscription> {
        let pool = self.pool.get();
        if pool.is_err() {
            return Vec::new();
        }
        let mut pool = pool.unwrap();
        let subs: Vec<Subscription> = subscriptions::table
            .filter(subscriptions::email.eq(email))
            .select(Subscription::as_select())
            .load(&mut pool)
            .unwrap_or_else(|_| Vec::new());

        subs.into_iter()
            .map(|sub| api_models::Subscription {
                email: None,
                subscription_id: sub.id.to_string(),
                subscription_name: sub.name,
                subscribe_since: sub.subscribed_at,
            })
            .collect()
    }

    fn remove_subscription(&mut self, id: Uuid) -> Result<api_models::Subscription, DomainError> {
        let pool = self.pool.get();
        match pool {
            Ok(p) => {
                let mut pool = p;
                let subscription = Subscription {
                    id,
                    email: "".to_string(),
                    name: "()".to_string(),
                    subscribed_at: time::SystemTime::now().into(),
                };
                let res: Result<Subscription, DomainError> =
                    diesel::delete(subscriptions::table.find(id))
                        .execute(&mut pool)
                        .map(|rows_affected| {
                            if rows_affected == 0 {
                                Err(DomainError::NotFound(format!(
                                    "subscription not found for id = {}",
                                    id
                                )))
                            } else {
                                Ok(subscription.clone())
                            }
                        })
                        .map_err(|err| DomainError::Internal(format!("Database error: {}", err)))?;

                match res {
                    Err(err) => Err(err),
                    Ok(s) => Ok(api_models::Subscription {
                        email: s.email.clone().into(),
                        subscription_id: s.id.to_string(),
                        subscription_name: s.name,
                        subscribe_since: s.subscribed_at,
                    }),
                }
            }
            Err(err) => Err(DomainError::Internal(format!(
                "Database error. Failed to get connection `(Err={})`",
                err
            ))),
        }
    }
}
