use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // 将log中的记录导入trace中
    LogTracer::init().expect("Failed to set logger");
    // 使用set_global_default方法，用于指定处理跨度的订阅器
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // 配置logger，默认是输入所有info及以上的日志
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // 将格式化跨度输出到sink中
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    // 使用Registry构建subscriber
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}
