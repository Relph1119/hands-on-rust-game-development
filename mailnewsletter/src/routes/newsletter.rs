use actix_web::{web, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use crate::routes::error_chain_fmt;
use sqlx::PgPool;


#[derive(serde::Deserialize)]
pub struct BodyData {
    // 邮件主题
    title: String,
    // 邮件内容
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    // HTML文本
    html: String,
    // 纯文本
    text: String,
}

pub async fn publish_newsletter(
    _body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, PublishError> {
    let _subscribers = get_confirmed_subscribers(&pool).await?;
    Ok(HttpResponse::Ok().finish())
}

struct ConfirmedSubscriber {
    email: String,
}

// 获取已确认的订阅者
#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    // sqlx::query_as!将检索到的行映射到第一个参数ConfirmedSubscriber中指定的类型。
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(thiserror::Error)]
enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
