use std::net::TcpListener;
use mailnewsletter::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 如果绑定地址失败，则发生错误io::Error，否则，调用.await
    // 获取TcpListener对象，获取绑定的实际端口
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");
    run(listener)?.await
}
