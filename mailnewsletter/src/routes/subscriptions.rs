use chrono::Utc;
use actix_web::{HttpResponse, web};
use uuid::Uuid;
use sqlx::PgPool;

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
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    // 从应用程序状态中取出连接
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_subscriber(&pool, &form).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

/**
 * 使用tracing::instrument宏过程，将跨度分别处理
 */
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, form)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now())
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        // 使用?操作符，可以在函数调用失败时提前结束当前函数
        })?;
    Ok(())
}