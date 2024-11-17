use std::net::TcpListener;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use mailnewsletter::configuration::get_configuration;
use mailnewsletter::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 将log中的记录导入trace中
    LogTracer::init().expect("Failed to set logger");

    // 配置logger，默认是输入所有info及以上的日志
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    // 将格式化跨度输出到stdout中
    let formatting_layer = BunyanFormattingLayer::new("mailnewsletter".into(), std::io::stdout);
    // 使用Registry构建subscriber
    let subscriber = Registry::default().with(env_filter).with(JsonStorageLayer).with(formatting_layer);
    // 使用set_global_default方法，用于指定处理跨度的订阅器
    set_global_default(subscriber).expect("Failed to set subscriber");

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
