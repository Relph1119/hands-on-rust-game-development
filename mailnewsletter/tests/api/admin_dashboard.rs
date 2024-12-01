use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // 准备
    let app = spawn_app().await;

    // 执行
    let response = app.get_admin_dashboard().await;

    // 断言
    assert_is_redirect_to(&response, "/login");
}