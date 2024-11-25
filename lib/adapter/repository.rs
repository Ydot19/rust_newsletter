use std::time;

use crate::domain::errors::DomainError;
use crate::port::models as api_models;

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

impl Repository {
    pub fn new(cfg: &DatabaseConfiguration) -> Result<Self, diesel::r2d2::PoolError> {
        let manager = ConnectionManager::<PgConnection>::new(cfg.url());
        let pool = Pool::builder()
            .max_size(10)
            .test_on_check_out(true)
            .build(manager)
            .expect("failed to get connection pool");
        Ok(Self { pool })
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::adapter::configuration;
    use crate::adapter::schema::subscriptions;
    use dotenvy::dotenv;
    use fake::{faker::internet::en::SafeEmail, Fake};
    pub struct TestContext {
        repo: Repository,
    }

    impl TestContext {
        pub async fn new(cfg: configuration::DatabaseConfiguration) -> Self {
            let repo = Repository::new(&cfg);
            assert!(repo.is_ok());
            let repo = repo.unwrap();
            Self { repo }
        }

        pub fn clear(&mut self) {
            let pool = &mut self.repo.pool.get().unwrap();
            let _ = diesel::delete(subscriptions::table).execute(pool);
        }
    }

    fn get_db_configuration() -> configuration::DatabaseConfiguration {
        dotenv().ok();
        configuration::DatabaseConfiguration::new()
    }

    #[tokio::test]
    async fn add_subscription() {
        // arrange
        let cfg = get_db_configuration();
        let mut ctx = TestContext::new(cfg).await;
        ctx.clear();
        const EMAIL: &str = "ydot19@github.com";
        // act
        let result = ctx.repo.add_subscription(
            "Ydot19".to_string(),
            EMAIL.to_string(),
            time::SystemTime::now(),
        );
        // assert
        assert!(result.is_ok());
        let res = result.unwrap();
        assert_eq!("Ydot19".to_string(), res.subscription_name);
    }

    #[tokio::test]
    async fn get_subscriptions() {
        // arrange
        let cfg = get_db_configuration();
        let mut ctx = TestContext::new(cfg).await;
        ctx.clear();
        let fake_email: String = SafeEmail().fake();
        let repo = ctx.repo.clone();
        let first = repo.clone().add_subscription(
            "a".to_string(),
            fake_email.clone(),
            time::SystemTime::now(),
        );
        assert!(first.is_ok());
        let first_subscription = first.unwrap();

        let second = repo.clone().add_subscription(
            "b".to_string(),
            fake_email.clone(),
            time::SystemTime::now(),
        );
        assert!(second.is_ok());
        let second_subscription = second.unwrap();
        // ACT - 1
        let res = ctx.repo.get_subscriptions(fake_email.clone());

        // assert
        assert_eq!(2, res.len());
        assert!(res
            .clone()
            .into_iter()
            .any(|el| el.subscription_id == first_subscription.subscription_id));

        assert!(res
            .clone()
            .into_iter()
            .any(|el| el.subscription_id == second_subscription.subscription_id));
        // ACT - 2
        let second_id = Uuid::from_str(second_subscription.subscription_id.as_str());
        let result = ctx.repo.remove_subscription(second_id.unwrap());

        // assert
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(second_subscription.subscription_id, result.subscription_id)
    }

    #[tokio::test]
    async fn remove_subscriptions() {
        // arrange
        let cfg = get_db_configuration();
        let mut ctx = TestContext::new(cfg).await;
        ctx.clear();
        let subscription_id = Uuid::new_v4();
        // act
        let result = ctx.repo.remove_subscription(subscription_id);
        // assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::NotFound(_)))
    }
}
