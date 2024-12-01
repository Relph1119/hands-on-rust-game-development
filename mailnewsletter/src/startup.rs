/*
 * 需要将run函数标记为公共的，不再是二进制文件入口，可以将其标记为异步的
 * 无须使用任何过程宏
 * 不需要使用async，正常返回Server，无须等待
 * 删除.await的目的是由于HttpServer::run会返回一个Server实例，当调用.await时，不断循环监听地址，处理到达的请求。
 * 采用随机端口运行后台程序：加入应用地址address作为参数
 */
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{confirm, health_check, home, login, login_form, publish_newsletter, subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use secrecy::SecretString;
use tracing_actix_web::TracingLogger;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        // 设置超时响应时间为5秒
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: HmacSecret,
) -> Result<Server, std::io::Error> {
    // 将连接包装到一个智能指针中
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    // HttpServer处理所有传输层的问题
    let server = HttpServer::new(move || {
        // App使用建造者模式，添加两个端点
        App::new()
            // 通过wrap将TraceLogger中间件加入到App中
            .wrap(TracingLogger::default())
            // web::get()实际上是Route::new().guard(guard::Get())的简写
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            // 向应用程序状态（与单个请求生命周期无关的数据）添加信息
            .app_data(db_pool.clone())
            // 将EmailClient注册到应用程序的上下文中
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(hmac_secret.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[derive(Clone, Debug)]
pub struct HmacSecret (pub SecretString);

pub struct ApplicationBaseUrl(pub String);

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        // 构建一个EmailClient
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        // 如果绑定地址失败，则发生错误io::Error，否则，调用.await
        // 获取TcpListener对象，获取绑定的实际端口
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
            HmacSecret(configuration.application.hmac_secret),
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // 程序仅在应用停止时返回
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
