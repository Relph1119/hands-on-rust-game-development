/*
 * 需要将run函数标记为公共的，不再是二进制文件入口，可以将其标记为异步的
 * 无须使用任何过程宏
 * 不需要使用async，正常返回Server，无须等待
 * 删除.await的目的是由于HttpServer::run会返回一个Server实例，当调用.await时，不断循环监听地址，处理到达的请求。
 * 采用随机端口运行后台程序：加入应用地址address作为参数
 */
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{admin_dashboard, change_password, change_password_form, confirm, health_check, home, log_out, login, login_form, publish_newsletter, subscribe};
use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::{ExposeSecret, SecretString};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        // 设置超时响应时间为5秒
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy_with(configuration.with_db())
}

pub async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: HmacSecret,
    redis_uri: SecretString,
) -> Result<Server, anyhow::Error> {
    // 将连接包装到一个智能指针中
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(hmac_secret.0.expose_secret().as_bytes());
    // 存储闪现消息
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    // 创建Redis会话存储
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    // HttpServer处理所有传输层的问题
    let server = HttpServer::new(move || {
        // App使用建造者模式，添加两个端点
        App::new()
            // 注册消息组件
            .wrap(message_framework.clone())
            // 注册Session组件
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
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
            .route("/admin/dashboard", web::get().to(admin_dashboard))
            .route("/admin/password", web::get().to(change_password_form))
            .route("/admin/password", web::post().to(change_password))
            .route("/admin/logout", web::post().to(log_out))
            // 向应用程序状态（与单个请求生命周期无关的数据）添加信息
            .app_data(db_pool.clone())
            // 将EmailClient注册到应用程序的上下文中
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            // 如果是自定义封装的类，依然需要用Data封装一次，否则无法获取
            .app_data(web::Data::new(hmac_secret.clone()))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

#[derive(Clone, Debug)]
pub struct HmacSecret(pub SecretString);

pub struct ApplicationBaseUrl(pub String);

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
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
            configuration.redis_uri,
        )
        .await?;

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
