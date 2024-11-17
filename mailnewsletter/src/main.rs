use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use mailnewsletter::configuration::get_configuration;
use mailnewsletter::startup::run;
use mailnewsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("mailnewsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 如果读取配置失败，发生panic
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await.expect("Failed to connect to Postgres.");
    // 如果绑定地址失败，则发生错误io::Error，否则，调用.await
    // 获取TcpListener对象，获取绑定的实际端口
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await?;
    Ok(())
}
