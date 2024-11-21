use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use mailnewsletter::configuration::get_configuration;
use mailnewsletter::startup::run;
use mailnewsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("mailnewsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 如果读取配置失败，发生panic
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new()
        // 设置超时响应时间为5秒
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy_with(configuration.database.with_db());
    // 如果绑定地址失败，则发生错误io::Error，否则，调用.await
    // 获取TcpListener对象，获取绑定的实际端口
    let address = format!("{}:{}",  configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await?;
    Ok(())
}
