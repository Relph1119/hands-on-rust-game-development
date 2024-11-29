use dotenv::dotenv;
use mailnewsletter::configuration::{get_configuration, DatabaseSettings};
use mailnewsletter::startup::{get_connection_pool, Application};
use mailnewsletter::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use sha3::Digest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use wiremock::MockServer;

/*
 * 使用once_cell，确保在测试期间最多只被初始化一次
 */
static TRACING: Lazy<()> = Lazy::new(|| {
    dotenv().ok();
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // 通过条件语句，将sink和stdout分开
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(default_filter_level, subscriber_name, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(default_filter_level, subscriber_name, std::io::sink);
        init_subscriber(subscriber);
    }
});

// 在发送给邮件API的请求中所包含的确认链接
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    // 添加端口
    pub port: u16,
    test_user: TestUser,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // 从发送给邮件API的请求中提取确认链接
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        // 从指定的字段中提取链接
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            // 解析确认的链接
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // 确保调用的API是本地的
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            // 设置URL中的端口
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            // 配置随机凭证
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // 删除数据库
    pub async fn cleanup(&self) {
        let connection_options = self.db_pool.connect_options();
        // 获取数据库名称，关闭数据库
        let database_name = connection_options.get_database().unwrap();
        // 关闭数据库连接池
        self.db_pool.close().await;

        // 等待所有连接关闭，设置超时时间为5秒
        let close_future = self.db_pool.close_event();
        match timeout(Duration::from_secs(5), close_future).await {
            Ok(_) => {
                if self.db_pool.is_closed() {
                    // 删除数据库
                    let configuration = get_configuration().expect("Failed to read configuration.");
                    let connection = PgPool::connect_with(configuration.database.without_db())
                        .await
                        .expect("Failed to connect to Postgres.");
                    connection
                        .execute(format!(r#"DROP DATABASE "{}";"#, database_name).as_str())
                        .await
                        .expect("Failed to create database.");
                }
            }
            Err(_) => tracing::error!("Timeout waiting for all connections to close."),
        }
    }
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let password_hash = sha3::Sha3_256::digest(self.password.as_bytes());
        let password_hash = format!("{:x}", password_hash);
        sqlx::query!(
            r#"INSERT INTO users (user_id, username, password_hash)
        VALUES ($1, $2, $3)"#,
            self.user_id,
            self.username,
            password_hash,
        )
        .execute(pool)
        .await
        .expect("Failed to store test user.");
    }
}

/*
 * 使用随机端口启动应用程序的一个实例，并返回地址
 */
pub async fn spawn_app() -> TestApp {
    // 只在第一次运行测试的时候调用
    Lazy::force(&TRACING);
    // 模拟一个服务器
    let email_server = MockServer::start().await;

    let configuration = {
        // 连接数据库
        let mut c = get_configuration().expect("Failed to read configuration.");
        // 创建随机名称的数据库
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        // 使用模拟服务器作为邮件API
        c.email_client.base_url = email_server.uri();
        c
    };

    configuration_database(&configuration.database).await;

    // 启动应用程序作为后台服务
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    // 在启动应用程序之前获取端口
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    // 将应用程序地址返回给调用者
    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
        test_user: TestUser::generate(),
    };

    // 添加一个随机的用户名和密码
    test_app.test_user.store(&test_app.db_pool).await;

    test_app
}

async fn configuration_database(config: &DatabaseSettings) -> PgPool {
    // 创建数据库
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // 迁移数据库
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
