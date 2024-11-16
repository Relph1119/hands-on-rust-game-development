use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // HttpServer处理所有传输层的问题
    HttpServer::new(|| {
        // App使用建造者模式，添加两个端点
        App::new()
            // web::get()实际上是Route::new().guard(guard::Get())的简写
            .route("/health_check", web::get().to(health_check))
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
