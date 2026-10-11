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
        transport: Box<AsyncSmtpTransport<Tokio1Executor>>,
        from: String,
    },
}

impl Mailer {
    pub fn new(config: &MailConfig) -> Result<Self, MailError> {
        Ok(match config {
            MailConfig::Log => Mailer::Log,
            // `smtp://host:port` is plain SMTP (Mailpit in compose); a real mail service is
            // `smtps://user:password@host:465` or `smtp://user:password@host:587?tls=required`.
            MailConfig::Smtp { url, from } => {
                let transport =
                    Box::new(AsyncSmtpTransport::<Tokio1Executor>::from_url(url)?.build());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn smtp(url: &str) -> Result<Mailer, MailError> {
        Mailer::new(&MailConfig::Smtp {
            url: url.to_owned(),
            from: "ФСП Таланты <no-reply@example.com>".to_owned(),
        })
    }

    #[test]
    fn accepts_plain_starttls_and_implicit_tls_urls() {
        for url in [
            "smtp://mailpit:1025",
            "smtp://user:p%40ss@smtp.example.com:587?tls=required",
            "smtps://user:secret@smtp.example.com:465",
        ] {
            assert!(smtp(url).is_ok(), "{url}");
        }
    }

    #[test]
    fn rejects_other_schemes() {
        assert!(smtp("http://mailpit:1025").is_err());
        assert!(smtp("mailpit:1025").is_err());
    }
}
