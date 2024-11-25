use std::env;

pub struct DatabaseConfiguration {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseConfiguration {
    pub fn new() -> Self {
        let username: String = env::var("DB_USER").ok().unwrap_or("postgres".to_string());

        let password: String = env::var("DB_PASSWORD")
            .ok()
            .unwrap_or("postgres".to_string());

        let port: u16 = env::var("DB_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5432);

        let host = env::var("DB_HOST").ok().unwrap_or("localhost".to_string());

        let db_name = env::var("DB_NAME").ok().unwrap_or("postgres".to_string());
        DatabaseConfiguration {
            username,
            password,
            port,
            host,
            database_name: db_name,
        }
    }

    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

impl Default for DatabaseConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg() {
        dotenvy::dotenv().ok();
        let cfg = DatabaseConfiguration::new();
        assert_eq!(cfg.username, "postgres");
        assert_eq!(cfg.password, "postgres");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.host, "localhost");
        assert_eq!(cfg.database_name, "newsletter");
    }
}
