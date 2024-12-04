use dotenv::dotenv;
use mailnewsletter::configuration::get_configuration;
use mailnewsletter::issue_delivery_worker::run_worker_until_stopped;
use mailnewsletter::startup::Application;
use mailnewsletter::telemetry::{get_subscriber, init_subscriber};
use std::fmt::{Debug, Display};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let subscriber = get_subscriber("mailnewsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 如果读取配置失败，发生panic
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    /*
     * 通过在当前任务上运行所有异步表达式，这些表达式可以并发运行，但是不能并行。意味着所有表达式都在同一个线程上运行，
     * 如果一个分支阻塞了线程，那么所有其他表达式都将无法继续运行。
     * 如果需要并行，使用tokio::spawn生成每个异步表达式。
     */
    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Background worker", o),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
