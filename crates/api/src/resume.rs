use std::panic::AssertUnwindSafe;

pub const MAX_PDF_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ResumeError {
    #[error("not a PDF file")]
    NotPdf,
    #[error("PDF is too large")]
    TooLarge,
    #[error("text could not be extracted")]
    Unreadable,
    #[error("PDF contains no text layer (scanned image?)")]
    NoText,
}

/// pdf-extract panics on some malformed files, so extraction runs isolated and any panic
/// becomes `Unreadable` instead of taking the worker down.
pub async fn extract_text(bytes: Vec<u8>) -> Result<String, ResumeError> {
    if bytes.len() > MAX_PDF_BYTES {
        return Err(ResumeError::TooLarge);
    }
    if !bytes.starts_with(b"%PDF") {
        return Err(ResumeError::NotPdf);
    }
    let text = tokio::task::spawn_blocking(move || {
        std::panic::catch_unwind(AssertUnwindSafe(|| pdf_extract::extract_text_from_mem(&bytes)))
    })
    .await
    .map_err(|_| ResumeError::Unreadable)?
    .map_err(|_| ResumeError::Unreadable)?
    .map_err(|_| ResumeError::Unreadable)?;
    if text.trim().chars().count() < 40 {
        return Err(ResumeError::NoText);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_non_pdf_and_garbage() {
        assert!(matches!(extract_text(b"hello".to_vec()).await, Err(ResumeError::NotPdf)));
        assert!(matches!(extract_text(b"%PDF-1.7 garbage".to_vec()).await, Err(ResumeError::Unreadable)));
    }

    #[tokio::test]
    async fn extracts_cyrillic_from_generated_pdf() {
        use crate::pdf::{Line, ProfileDoc, Section, render};
        let pdf = render(&ProfileDoc {
            title: "Иван Петров".into(),
            subtitle: "Бэкенд-разработчик, Go, PostgreSQL, Kafka".into(),
            badge: None,
            sections: vec![Section {
                heading: "Контакты".into(),
                lines: vec![Line::KeyValue("Телефон".into(), "+7 999 123-45-67".into())],
            }],
            footer: "Опыт работы 4 года".into(),
        })
        .unwrap();
        let text = extract_text(pdf).await.unwrap();
        assert!(text.contains("Петров"), "{text}");
        assert!(text.contains("PostgreSQL"), "{text}");
    }
}
