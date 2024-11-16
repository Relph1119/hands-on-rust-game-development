use actix_web::{HttpResponse, web};
use chrono::Utc;
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
 */
pub async fn subscribe(
    form: web::Form<FormData>,
    // 从应用程序状态中取出连接
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now())
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}