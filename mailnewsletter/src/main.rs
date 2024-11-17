use std::net::TcpListener;
use env_logger::Env;
use sqlx::PgPool;
use mailnewsletter::configuration::get_configuration;
use mailnewsletter::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 配置logger，默认是输入所有info及以上的日志
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    // 如果读取配置失败，发生panic
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await.expect("Failed to connect to Postgres.");
    // 如果绑定地址失败，则发生错误io::Error，否则，调用.await
    // 获取TcpListener对象，获取绑定的实际端口
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;

    run(listener, connection_pool)?.await
}
