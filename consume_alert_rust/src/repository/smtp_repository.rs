use crate::common::*;
use crate::config::AppConfig;

use lettre::{
    message::{MultiPart, SinglePart},
    transport::smtp::authentication::Credentials as SmtpCredentials,
    AsyncSmtpTransport, AsyncTransport, Message as MailMessage,
};

#[async_trait]
pub trait SmtpRepository: Send + Sync {
    async fn send_html_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct SmtpRepositoryImpl {
    smtp_server: String,
    smtp_id: String,
    smtp_pw: String,
}

impl SmtpRepositoryImpl {
    /// Reads SMTP connection details from `AppConfig` and returns a new repository instance.
    ///
    /// # Errors
    ///
    /// Returns an error if `AppConfig` has not been initialized before this call.
    pub fn new() -> anyhow::Result<Self> {
        let cfg: &AppConfig = AppConfig::get_global();
        Ok(Self {
            smtp_server: cfg.smtp_server().clone(),
            smtp_id: cfg.smtp_id().clone(),
            smtp_pw: cfg.smtp_pw().clone(),
        })
    }
}

#[async_trait]
impl SmtpRepository for SmtpRepositoryImpl {
    /// Sends an HTML email to a single recipient via the configured SMTP server.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient email address
    /// * `subject` - Email subject line
    /// * `html_body` - HTML content of the email body
    ///
    /// # Errors
    ///
    /// Returns an error if address parsing, SMTP connection, or message delivery fails.
    async fn send_html_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
    ) -> anyhow::Result<()> {
        let email: MailMessage = MailMessage::builder()
            .from(self.smtp_id.parse().map_err(|e| {
                anyhow!(
                    "[SmtpRepositoryImpl::send_html_email] Invalid sender address '{}': {:?}",
                    self.smtp_id,
                    e
                )
            })?)
            .to(to.parse().map_err(|e| {
                anyhow!(
                    "[SmtpRepositoryImpl::send_html_email] Invalid recipient address '{}': {:?}",
                    to,
                    e
                )
            })?)
            .subject(subject)
            .multipart(
                MultiPart::alternative().singlepart(SinglePart::html(html_body.to_string())),
            )
            .map_err(|e| {
                anyhow!(
                    "[SmtpRepositoryImpl::send_html_email] Failed to build email message: {:?}",
                    e
                )
            })?;

        let creds: SmtpCredentials =
            SmtpCredentials::new(self.smtp_id.clone(), self.smtp_pw.clone());

        let mailer: AsyncSmtpTransport<lettre::Tokio1Executor> =
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&self.smtp_server)
                .map_err(|e| {
                    anyhow!(
                        "[SmtpRepositoryImpl::send_html_email] Failed to connect to SMTP server '{}': {:?}",
                        self.smtp_server,
                        e
                    )
                })?
                .credentials(creds)
                .build();

        mailer.send(email).await.map_err(|e| {
            anyhow!(
                "[SmtpRepositoryImpl::send_html_email] Failed to send email to '{}': {:?}",
                to,
                e
            )
        })?;

        Ok(())
    }
}
