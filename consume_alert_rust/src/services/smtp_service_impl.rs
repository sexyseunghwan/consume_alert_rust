use crate::common::*;
use crate::config::AppConfig;
use crate::repository::smtp_repository::*;
use crate::service_traits::smtp_service::*;

use futures::future::join_all;

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct SmtpServiceImpl<R: SmtpRepository> {
    smtp_repo: R,
    receivers: Vec<String>,
}

impl<R: SmtpRepository> SmtpServiceImpl<R> {
    /// Builds the service from a repository instance, reading the receiver list from `AppConfig`.
    pub fn new(smtp_repo: R) -> Self {
        let cfg: &AppConfig = AppConfig::get_global();
        let receivers: Vec<String> = cfg
            .smtp_receivers()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            smtp_repo,
            receivers,
        }
    }
}

#[async_trait]
impl<R: SmtpRepository + Send + Sync> SmtpService for SmtpServiceImpl<R> {
    /// Sends an HTML email to all recipients configured in `SMTP_RECEIVERS`.
    ///
    /// Delivery to each recipient is attempted concurrently. Per-recipient failures are
    /// logged as errors but do not propagate — the method always returns `Ok(())`.
    ///
    /// # Arguments
    ///
    /// * `subject` - Email subject line
    /// * `html_body` - HTML content of the email body
    ///
    /// # Errors
    ///
    /// Returns an error only if the receiver list is empty and no send is possible.
    async fn send_email(&self, subject: &str, html_body: &str) -> anyhow::Result<()> {
        let tasks = self
            .receivers
            .iter()
            .map(|addr| self.smtp_repo.send_html_email(addr, subject, html_body));

        let results: Vec<anyhow::Result<()>> = join_all(tasks).await;

        for result in results {
            match result {
                Ok(_) => info!("[SmtpServiceImpl::send_email] Email sent successfully"),
                Err(e) => error!(
                    "[SmtpServiceImpl::send_email] Failed to send email: {:#}",
                    e
                ),
            }
        }

        Ok(())
    }

    /// Sends an HTML email to a single specific recipient.
    ///
    /// # Arguments
    ///
    /// * `address` - Target recipient email address
    /// * `subject` - Email subject line
    /// * `html_body` - HTML content of the email body
    ///
    /// # Errors
    ///
    /// Returns an error if the SMTP delivery fails.
    async fn send_email_to(
        &self,
        address: &str,
        subject: &str,
        html_body: &str,
    ) -> anyhow::Result<()> {
        self.smtp_repo
            .send_html_email(address, subject, html_body)
            .await
    }
}
