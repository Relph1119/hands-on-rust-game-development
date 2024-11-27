use actix_web::{HttpRequest, HttpResponse};

// 使用HttpResponse代替impl Responder类型
pub async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}
