use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use mailnewsletter::configuration::{DatabaseSettings, get_configuration};
use mailnewsletter::startup::{Application, get_connection_pool};
use mailnewsletter::telemetry::{get_subscriber, init_subscriber};

/*
 * 使用once_cell，确保在测试期间最多只被初始化一次
 */
static TRACING: Lazy<()> = Lazy::new(|| {
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

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        return reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
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
                        .await.expect("Failed to connect to Postgres.");
                    connection.execute(format!(r#"DROP DATABASE "{}";"#, database_name).as_str())
                        .await.expect("Failed to create database.");
                }
            }
            Err(_) => tracing::error!("Timeout waiting for all connections to close."),
        }
    }
}

/*
 * 使用随机端口启动应用程序的一个实例，并返回地址
 */
pub async fn spawn_app() -> TestApp {
    // 只在第一次运行测试的时候调用
    Lazy::force(&TRACING);

    let configuration = {
        // 连接数据库
        let mut c = get_configuration().expect("Failed to read configuration.");
        // 创建随机名称的数据库
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configuration_database(&configuration.database).await;

    // 启动应用程序作为后台服务
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    // 在启动应用程序之前获取端口
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    // 将应用程序地址返回给调用者
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
    }
}

async fn configuration_database(config: &DatabaseSettings) -> PgPool {
    // 创建数据库
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await.expect("Failed to connect to Postgres.");
    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await.expect("Failed to create database.");

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