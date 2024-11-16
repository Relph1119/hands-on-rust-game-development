use std::net::TcpListener;
use sqlx::PgPool;
use mailnewsletter::configuration::get_configuration;
use mailnewsletter::startup::run;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool
}

/*
 * 使用随机端口启动应用程序的一个实例，并返回地址
 */
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    // 检索操作系统分配的端口
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // 连接数据库
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connect_pool = PgPool::connect(&configuration.database.connection_string())
        .await.expect("Failed to connect to Postgres.");

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
}