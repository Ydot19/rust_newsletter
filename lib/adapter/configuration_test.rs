#[cfg(test)]
mod tests {
    use crate::adapter::configuration::DatabaseConfiguration;

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
