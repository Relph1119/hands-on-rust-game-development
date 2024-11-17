use std::net::TcpListener;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use mailnewsletter::configuration::{DatabaseSettings, get_configuration};
use mailnewsletter::startup::run;
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
    // 删除数据库
    pub async fn cleanup(&self) {
        // 获取数据库名称，关闭数据库
        let database_name = self.db_pool.connect_options().get_database().unwrap();
        self.db_pool.close().await;
        // 删除数据库
        let configuration = get_configuration().expect("Failed to read configuration.");
        let mut connection = PgConnection::connect(&configuration.database.connection_string_without_db())
            .await.expect("Failed to connect to Postgres.");
        connection.execute(format!(r#"DROP DATABASE "{}";"#, database_name).as_str())
            .await.expect("Failed to create database.");
    }
}

/*
 * 使用随机端口启动应用程序的一个实例，并返回地址
 */
async fn spawn_app() -> TestApp {
    // 只在第一次运行测试的时候调用
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    // 检索操作系统分配的端口
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // 连接数据库
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // 创建随机名称的数据库
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connect_pool = configuration_database(&configuration.database).await;

    let server = run(listener, connect_pool.clone())
        .expect("Failed to bind address");
    // 启动服务器作为后台任务
    let _ = tokio::spawn(server);
    // 将应用程序地址返回给调用者
    TestApp {
        address,
        db_pool: connect_pool,
    }
}

pub async fn configuration_database(config: &DatabaseSettings) -> PgPool {
    // 创建数据库
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await.expect("Failed to connect to Postgres.");
    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await.expect("Failed to create database.");

    // 迁移数据库
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    // 准备
    let app = spawn_app().await;
    // 需要引入reqwest对应用程序执行HTTP请求
    let client = reqwest::Client::new();

    // 执行
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // 断言：返回状态码200
    assert!(response.status().is_success());
    // 没有响应体
    assert_eq!(Some(0), response.content_length());
    // 清理数据库
    app.cleanup().await;
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // 准备
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // 执行
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // 断言
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    // 清理数据库
    app.cleanup().await;
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // 准备
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // 执行
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // 断言
        assert_eq!(400,
                   response.status().as_u16(),
                   "The API did not fail with 400 Bad Request when the payload was {}",
                   error_message);
    }

    // 清理数据库
    app.cleanup().await;
}
