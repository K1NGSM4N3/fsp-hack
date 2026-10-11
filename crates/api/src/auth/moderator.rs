//! `api create-moderator <email>`: registration refuses the `admin` role, so the operator creates
//! moderator accounts from the command line, the way the demo seed creates `moderator@demo.fsp`:
//! a verified users row and nothing else, no candidate or company profile, no consents.

use std::io::{BufRead, IsTerminal, Write};

use sqlx::PgPool;
use uuid::Uuid;

use super::routes::{MAX_PASSWORD_BYTES, MIN_PASSWORD_CHARS, valid_email};
use crate::crypto::{Crypto, CryptoError, normalize_email};

#[derive(Debug, thiserror::Error)]
pub enum CreateModeratorError {
    #[error("использование: api create-moderator <email>, пароль - одной строкой в stdin")]
    Usage,
    #[error("некорректный email: {0}")]
    Email(String),
    #[error("пароль не получен: передайте его одной строкой в stdin")]
    NoPassword,
    #[error("пароль короче {} символов", MIN_PASSWORD_CHARS)]
    ShortPassword,
    #[error("пароль длиннее {} байт", MAX_PASSWORD_BYTES)]
    LongPassword,
    #[error("аккаунт {0} уже существует, роль существующего пользователя команда не меняет")]
    Taken(String),
    #[error("stdin: {0}")]
    Stdin(#[source] std::io::Error),
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

/// No `Debug`: the password must not reach a log by accident.
pub struct NewModerator {
    /// Normalized, as registration stores and login looks it up.
    pub email: String,
    pub password: String,
}

/// The password comes from stdin rather than argv: arguments end up in shell history and in the
/// process list.
pub fn from_cli(
    args: impl IntoIterator<Item = String>,
) -> Result<NewModerator, CreateModeratorError> {
    let email = email_arg(args)?;
    let stdin = std::io::stdin();
    if stdin.is_terminal() {
        eprint!("Пароль модератора: ");
        let _ = std::io::stderr().flush();
    }
    let password = read_password(stdin.lock())?;
    check(&email, password)
}

fn email_arg(args: impl IntoIterator<Item = String>) -> Result<String, CreateModeratorError> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(email), None) => Ok(email),
        _ => Err(CreateModeratorError::Usage),
    }
}

/// The first line without its terminator; leading and trailing spaces are part of the password,
/// as they are at registration and login.
fn read_password(mut input: impl BufRead) -> Result<String, CreateModeratorError> {
    let mut line = String::new();
    input
        .read_line(&mut line)
        .map_err(CreateModeratorError::Stdin)?;
    let password = line
        .strip_suffix('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .unwrap_or(&line);
    if password.is_empty() {
        return Err(CreateModeratorError::NoPassword);
    }
    Ok(password.to_owned())
}

fn check(email: &str, password: String) -> Result<NewModerator, CreateModeratorError> {
    let email = normalize_email(email);
    if !valid_email(&email) {
        return Err(CreateModeratorError::Email(email));
    }
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(CreateModeratorError::ShortPassword);
    }
    if password.len() > MAX_PASSWORD_BYTES {
        return Err(CreateModeratorError::LongPassword);
    }
    Ok(NewModerator { email, password })
}

/// Refuses an email that already has an account instead of changing that user's role. A deleted
/// account does not hold its address: its blind index is replaced by a tombstone.
pub async fn create(
    pool: &PgPool,
    crypto: &Crypto,
    m: &NewModerator,
) -> Result<Uuid, CreateModeratorError> {
    let bidx = crypto.blind_index(&m.email);
    if db::users::user_by_email_bidx(pool, &bidx).await?.is_some() {
        return Err(CreateModeratorError::Taken(m.email.clone()));
    }
    let hash = crypto.passwords.hash(&m.password)?;
    let id = Uuid::now_v7();
    let mut tx = pool.begin().await?;
    // The check above races with a sign-up of the same address; the unique index decides.
    db::users::insert_user(
        &mut *tx,
        id,
        db::types::UserRole::Admin,
        &crypto.encrypt_str(&m.email)?,
        &bidx,
        &hash,
    )
    .await
    .map_err(|e| {
        if db::is_unique_violation(&e) {
            CreateModeratorError::Taken(m.email.clone())
        } else {
            e.into()
        }
    })?;
    db::users::mark_email_verified(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn exactly_one_email_argument() {
        assert_eq!(
            email_arg(args(&["mod@example.ru"])).unwrap(),
            "mod@example.ru"
        );
        assert!(matches!(
            email_arg(args(&[])),
            Err(CreateModeratorError::Usage)
        ));
        assert!(matches!(
            email_arg(args(&["mod@example.ru", "secret-password"])),
            Err(CreateModeratorError::Usage)
        ));
    }

    #[test]
    fn password_is_the_first_line_without_terminator() {
        assert_eq!(
            read_password(&b"Sever-2026!\nnext\n"[..]).unwrap(),
            "Sever-2026!"
        );
        assert_eq!(
            read_password(&b"Sever-2026!\r\n"[..]).unwrap(),
            "Sever-2026!"
        );
        assert_eq!(read_password(&b"Sever-2026!"[..]).unwrap(), "Sever-2026!");
        assert_eq!(read_password(&b" pass word \n"[..]).unwrap(), " pass word ");
    }

    #[test]
    fn missing_password_is_refused() {
        for input in [&b""[..], b"\n", b"\r\n"] {
            assert!(matches!(
                read_password(input),
                Err(CreateModeratorError::NoPassword)
            ));
        }
    }

    #[test]
    fn registration_rules_apply() {
        let m = check(" Mod@Example.RU ", "Sever-2026".into()).unwrap();
        assert_eq!(m.email, "mod@example.ru");
        assert!(matches!(
            check("mod@example", "Sever-2026".into()),
            Err(CreateModeratorError::Email(_))
        ));
        assert!(matches!(
            check("mod@example.ru", "пароль7".into()),
            Err(CreateModeratorError::ShortPassword)
        ));
        assert!(check("mod@example.ru", "пароль78".into()).is_ok());
        assert!(check("mod@example.ru", "x".repeat(MAX_PASSWORD_BYTES)).is_ok());
        assert!(matches!(
            check("mod@example.ru", "x".repeat(MAX_PASSWORD_BYTES + 1)),
            Err(CreateModeratorError::LongPassword)
        ));
    }
}
