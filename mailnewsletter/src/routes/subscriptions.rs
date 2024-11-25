use chrono::Utc;
use actix_web::{HttpResponse, web};
use uuid::Uuid;
use sqlx::PgPool;
use crate::domain::{NewSubscriber, SubscriberName};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/*
 * actix-web使用HashMap存储应用程序状态，
 * 当一个新的请求到来时，web::Data会获取函数签名中指定类型的TypeId，并检查类型映射中是否存在对应的记录。
 * 如果存在，检索到的值强制转换为指定的类型，并传递给处理器。
 * 使用tracing::instrument宏，创建请求ID，用于将日志和请求关联起来，并记录订阅者信息。
 */
#[tracing::instrument(
name = "Adding new subscriber",
skip(form, pool),
fields(
subscriber_email = % form.email,
subscriber_name = % form.name
)
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    // 从应用程序状态中取出连接
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // 校验输入的订阅者用户名
    let new_subscriber = NewSubscriber {
        email: form.0.email,
        name: SubscriberName::parse(form.0.name).expect("Name validation failed."),
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

/**
 * 使用tracing::instrument宏过程，将跨度分别处理
 */
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber ,pool)
)]
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email,
        // 使用SubscriberName的inner_ref方法，获取内部字符串的引用
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // 使用?操作符，可以在函数调用失败时提前结束当前函数
    })?;
    Ok(())
}