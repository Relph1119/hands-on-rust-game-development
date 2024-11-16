use std::net::TcpListener;
use sqlx::{Connection, PgConnection};
use mailnewsletter::configuration::get_configuration;

/*
 * 使用随机端口启动应用程序的一个实例，并返回地址
 */
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    // 检索操作系统分配的端口
    let port = listener.local_addr().unwrap().port();
    let server = mailnewsletter::startup::run(listener).expect("Failed to bind address");
    // 启动服务器作为后台任务
    let _ = tokio::spawn(server);
    // 将应用程序地址返回给调用者
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    // 准备
    let address = spawn_app();
    // 需要引入reqwest对应用程序执行HTTP请求
    let client = reqwest::Client::new();

    // 执行
    let response = client
        .get(&format!("{}/health_check", &address))
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
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();
    // 为了调用PgConnect::connect方法，Connection必须位于作用域内
    // 必须标记为可变的
    let mut connection = PgConnection::connect(&connection_string)
        .await.expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    // 执行
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // 断言
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // 准备
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // 执行
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
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