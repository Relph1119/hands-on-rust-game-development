use std::net::TcpListener;

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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    // 检索操作系统分配的端口
    let port = listener.local_addr().unwrap().port();
    let server = mailnewsletter::run(listener).expect("Failed to bind address");
    // 启动服务器作为后台任务
    let _ = tokio::spawn(server);
    // 将应用程序地址返回给调用者
    format!("http://127.0.0.1:{}", port)
}