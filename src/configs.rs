use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ApiConfig {
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_host() -> String { "0.0.0.0:8080".to_string() }
fn default_max_connections() -> u32 { 8 }
