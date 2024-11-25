use serde_aux::field_attributes::deserialize_number_from_string;
use secrecy::{ExposeSecret, SecretString};
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize)]
#[derive(Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
#[derive(Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    // 授权token配置
    pub authorization_token: SecretString,
    // 配置超时时间
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize)]
#[derive(Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

// 使用SecretString对数据库密码进行保护
#[derive(serde::Deserialize)]
#[derive(Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    // 确定是否要求加密连接
    pub require_ssl: bool,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_directory = base_path.join("configuration");

    // 检查运行环境，如果没有指定，则默认是local
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());

    // 初始化配置读取器
    let settings = config::Config::builder()
        // 从一个名为configuration.yaml的文件中读取配置
        .add_source(config::File::from(configuration_directory.join("base.yaml")))
        .add_source(config::File::from(configuration_directory.join(&environment_filename)))
        // 从环境变量中添加设置，前缀为APP，将__作为分隔
        .add_source(config::Environment::with_prefix("APP").prefix_separator("_").separator("__"))
        .build()?;
    // 尝试将配置转换为Settings类型
    settings.try_deserialize::<Settings>()
}

// 应用程序可能的运行环境
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either local or production.",
                other)),
        }
    }
}

impl DatabaseSettings {
    // 配置Postgres的数据库连接
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);
        // 将sqlx的日志级别设置为trace
        options.clone().log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}