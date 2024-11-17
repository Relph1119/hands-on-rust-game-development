use chrono::Utc;
use actix_web::{HttpResponse, web};
use uuid::Uuid;
use sqlx::PgPool;
use tracing::Instrument;

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
    // 创建请求ID，用于将日志和请求关联起来
    let request_id = Uuid::new_v4();
    // 创建一个info级别的跨度，记录订阅者信息
    let request_span = tracing::info_span!("Adding new subscriber",
        %request_id, subscriber_email = %form.email, subscriber_name = %form.name);

    // 守卫对象，在这个变量被析构前，所有的下游跨度都会被注册为当前跨度的子跨度
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database");
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
        // 绑定这个插桩，等待这个future完成
        .instrument(query_span)
        .await
    {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            // {:?}表示使用std::fmt::Debug格式捕获查询错误
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}