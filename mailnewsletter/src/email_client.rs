use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

#[derive(Clone, Debug)]
pub struct EmailClient {
    // 存储Client实例
    http_client: Client,
    // 用于存储发出API请求的URL
    base_url: String,
    sender: SubscriberEmail,
    // 添加token授权
    authorization_token: SecretString,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: SecretString,
        time_out: std::time::Duration,
    ) -> Self {
        // 设置全局超时时间
        let http_client = Client::builder().timeout(time_out).build().unwrap();

        Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        // 发送HTTP请求
        let url = format!("{}/email", self.base_url);
        // 构建JSON请求体
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(&url)
            // 添加授权码
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            // 如果服务器返回错误，则将响应转换为错误
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::SecretString;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // 尝试将请求体解析为JSON
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                // 检查是否填充了所有必填字段
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                // 如果解析失败，则不匹配请求
                false
            }
        }
    }

    // 生成随机的邮件主题
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    // 生成随机的邮件内容
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    // 生成随机的订阅者电子邮件地址
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    // 获取EmailClient的测试实例
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            SecretString::from(Faker.fake::<String>()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // 准备
        let mock_server = MockServer::start().await;
        // mock_server.uri方法获取模拟服务器的地址
        let email_client = email_client(mock_server.uri());

        // 当收到任何请求时，返回状态码200
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            // 使用自定义匹配器，将请求体解析成JSON
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            // 仅接收到一个与mock设置的条件匹配的请求
            .expect(1)
            .mount(&mock_server)
            .await;

        // 执行
        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // 断言
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // 准备
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // 执行
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // 断言
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // 准备
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // 执行
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // 断言
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // 准备
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200)
            // 3分钟
            .set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // 执行
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        // 断言
        assert_err!(outcome);
    }
}
