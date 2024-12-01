use dotenv::dotenv;
use mailnewsletter::configuration::get_configuration;
use mailnewsletter::startup::Application;
use mailnewsletter::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let subscriber = get_subscriber("mailnewsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 如果读取配置失败，发生panic
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
