use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::header::ContentType};

use crate::config::MailConfig;

#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("SMTP_URL must look like smtp://host:port")]
    InvalidUrl,
    #[error(transparent)]
    Address(#[from] lettre::address::AddressError),
    #[error(transparent)]
    Message(#[from] lettre::error::Error),
    #[error(transparent)]
    Smtp(#[from] lettre::transport::smtp::Error),
}

pub enum Mailer {
    Log,
    Smtp { transport: AsyncSmtpTransport<Tokio1Executor>, from: String },
}

impl Mailer {
    pub fn new(config: &MailConfig) -> Result<Self, MailError> {
        Ok(match config {
            MailConfig::Log => Mailer::Log,
            MailConfig::Smtp { url, from } => {
                let rest = url.strip_prefix("smtp://").ok_or(MailError::InvalidUrl)?.trim_end_matches('/');
                let (host, port) = rest.split_once(':').unwrap_or((rest, "25"));
                let port = port.parse().map_err(|_| MailError::InvalidUrl)?;
                let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host).port(port).build();
                Mailer::Smtp { transport, from: from.clone() }
            }
        })
    }

    pub async fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), MailError> {
        match self {
            Mailer::Log => {
                tracing::info!(%to, %subject, "email (MAIL_MODE=log):\n{body}");
                Ok(())
            }
            Mailer::Smtp { transport, from } => {
                let message = Message::builder()
                    .from(from.parse()?)
                    .to(to.parse()?)
                    .subject(subject)
                    .header(ContentType::TEXT_PLAIN)
                    .body(body.to_owned())?;
                transport.send(message).await?;
                Ok(())
            }
        }
    }
}
