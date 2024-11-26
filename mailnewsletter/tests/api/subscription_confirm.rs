use reqwest::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // 准备
    let app = spawn_app().await;

    // 执行
    let response = reqwest::get(&format!("{}/subscriptions/confirm", &app.address))
        .await
        .unwrap();

    // 断言
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // 准备
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // 先调用POST /subscriptions方法
    app.post_subscriptions(body.into()).await;

    // 获取第一个被截取的请求
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    // 将正文从二进制数据转成JSON格式
    let confirmation_links = app.get_confirmation_links(email_request);

    // 执行
    let response = reqwest::get(confirmation_links.html)
        .await
        .unwrap();

    // 断言
    assert_eq!(response.status().as_u16(), 200);
}