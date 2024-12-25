#[cfg(test)]
mod test {

    use std::str::FromStr;
    use std::time;

    use crate::adapter::repository::{connection_pool, SubscriptionRepository};
    use crate::adapter::schema::subscriptions;
    use crate::adapter::{configuration, repository::Repository};
    use crate::domain::errors::DomainError;
    use diesel::r2d2::{ConnectionManager, Pool};
    use diesel::{PgConnection, RunQueryDsl};
    use dotenvy::dotenv;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use uuid::Uuid;

    pub struct TestContext {
        repo: Repository,
        pool: Pool<ConnectionManager<PgConnection>>,
    }

    impl TestContext {
        pub async fn new(cfg: configuration::DatabaseConfiguration) -> Self {
            let repo = Repository::new(&cfg);
            assert!(repo.is_ok());
            let repo = repo.unwrap();
            Self {
                repo,
                pool: connection_pool(&cfg),
            }
        }

        pub fn clear(&mut self) {
            let mut conn = self.pool.clone().get().unwrap();
            let _ = diesel::delete(subscriptions::table).execute(&mut conn);
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
