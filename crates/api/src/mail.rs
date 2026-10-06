use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{
        MultiPart,
        header::{ContentType, HeaderName, HeaderValue},
    },
};

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

/// A letter with a plain-text and an HTML part.
pub struct Email<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    pub text: &'a str,
    pub html: &'a str,
    /// `<id@host>`; the same on every retry of one letter.
    pub message_id: &'a str,
    /// One-click unsubscribe URL (RFC 8058): `List-Unsubscribe` and `List-Unsubscribe-Post`.
    pub list_unsubscribe: Option<&'a str>,
}

pub enum Mailer {
    Log,
    Smtp {
        transport: AsyncSmtpTransport<Tokio1Executor>,
        from: String,
    },
}

impl Mailer {
    pub fn new(config: &MailConfig) -> Result<Self, MailError> {
        Ok(match config {
            MailConfig::Log => Mailer::Log,
            MailConfig::Smtp { url, from } => {
                let rest = url
                    .strip_prefix("smtp://")
                    .ok_or(MailError::InvalidUrl)?
                    .trim_end_matches('/');
                let (host, port) = rest.split_once(':').unwrap_or((rest, "25"));
                let port = port.parse().map_err(|_| MailError::InvalidUrl)?;
                let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                    .port(port)
                    .build();
                Mailer::Smtp {
                    transport,
                    from: from.clone(),
                }
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

    pub async fn send_email(&self, email: &Email<'_>) -> Result<(), MailError> {
        match self {
            Mailer::Log => {
                tracing::info!(
                    to = %email.to,
                    subject = %email.subject,
                    message_id = %email.message_id,
                    list_unsubscribe = ?email.list_unsubscribe,
                    "email (MAIL_MODE=log):\n{}",
                    email.text
                );
                Ok(())
            }
            Mailer::Smtp { transport, from } => {
                let mut builder = Message::builder()
                    .from(from.parse()?)
                    .to(email.to.parse()?)
                    .subject(email.subject)
                    .message_id(Some(email.message_id.to_owned()));
                if let Some(url) = email.list_unsubscribe {
                    builder = builder
                        .raw_header(HeaderValue::new(
                            HeaderName::new_from_ascii_str("List-Unsubscribe"),
                            format!("<{url}>"),
                        ))
                        .raw_header(HeaderValue::new(
                            HeaderName::new_from_ascii_str("List-Unsubscribe-Post"),
                            "List-Unsubscribe=One-Click".to_owned(),
                        ));
                }
                let message = builder.multipart(MultiPart::alternative_plain_html(
                    email.text.to_owned(),
                    email.html.to_owned(),
                ))?;
                transport.send(message).await?;
                Ok(())
            }
        }
    }
}
