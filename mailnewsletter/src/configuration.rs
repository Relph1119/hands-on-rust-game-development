use secrecy::{ExposeSecret, SecretString};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16
}

// 使用SecretString对数据库密码进行保护
#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // 初始化配置读取器
    let settings = config::Config::builder()
        // 从一个名为configuration.yaml的文件中读取配置
        .add_source(config::File::new("configuration.yaml", config::FileFormat::Yaml))
        .build()?;
    // 尝试将配置转换为Settings类型
    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    // 配置Postgres的数据库连接
    pub fn connection_string(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        ).into()
    }

    pub fn connection_string_without_db(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        ).into()
    }
}