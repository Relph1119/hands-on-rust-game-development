/*
 * 需要将run函数标记为公共的，不再是二进制文件入口，可以将其标记为异步的
 * 无须使用任何过程宏
 * 不需要使用async，正常返回Server，无须等待
 * 删除.await的目的是由于HttpServer::run会返回一个Server实例，当调用.await时，不断循环监听地址，处理到达的请求。
 * 采用随机端口运行后台程序：加入应用地址address作为参数
 */
use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use actix_web::middleware::Logger;
use sqlx::PgPool;
use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // 将连接包装到一个智能指针中
    let db_pool = web::Data::new(db_pool);
    // HttpServer处理所有传输层的问题
    let server = HttpServer::new(move || {
        // App使用建造者模式，添加两个端点
        App::new()
            // 通过wrap将Logger中间件加入到App中
            .wrap(Logger::default())
            // web::get()实际上是Route::new().guard(guard::Get())的简写
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // 向应用程序状态（与单个请求生命周期无关的数据）添加信息
            .app_data(db_pool.clone())
    })
        .listen(listener)?
        .run();

    Ok(server)
}