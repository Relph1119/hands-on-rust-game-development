use crate::helpers::spawn_app;

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