use crate::common::*;

#[async_trait]
pub trait SmtpService {
    async fn send_email(&self, subject: &str, html_body: &str) -> anyhow::Result<()>;
    async fn send_email_to(
        &self,
        address: &str,
        subject: &str,
        html_body: &str,
    ) -> anyhow::Result<()>;
}
