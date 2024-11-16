use std::net::TcpListener;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use actix_web::dev::Server;

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().finish()
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

/*
 * 需要将run函数标记为公共的，不再是二进制文件入口，可以将其标记为异步的
 * 无须使用任何过程宏
 * 不需要使用async，正常返回Server，无须等待
 * 删除.await的目的是由于HttpServer::run会返回一个Server实例，当调用.await时，不断循环监听地址，处理到达的请求。
 * 采用随机端口运行后台程序：加入应用地址address作为参数
 */
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // HttpServer处理所有传输层的问题
    let server = HttpServer::new(|| {
        // App使用建造者模式，添加两个端点
        App::new()
            // web::get()实际上是Route::new().guard(guard::Get())的简写
            .route("/health_check", web::get().to(health_check))
    })
        .listen(listener)?
        .run();

    Ok(server)
}