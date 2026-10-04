// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertUserParams<T1: crate::BytesSql, T2: crate::BytesSql, T3: crate::StringSql> {
    pub id: uuid::Uuid,
    pub role: crate::types::UserRole,
    pub email_enc: T1,
    pub email_bidx: T2,
    pub password_hash: T3,
}
#[derive(Debug)]
pub struct UpdatePasswordHashParams<T1: crate::StringSql> {
    pub password_hash: T1,
    pub id: uuid::Uuid,
}
#[derive(Debug)]
pub struct AnonymizeUserParams<T1: crate::BytesSql> {
    pub tombstone_bidx: T1,
    pub id: uuid::Uuid,
}
#[derive(Debug)]
pub struct InsertEmailVerificationParams<T1: crate::BytesSql> {
    pub token_hash: T1,
    pub user_id: uuid::Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug)]
pub struct InsertRefreshTokenParams<T1: crate::BytesSql> {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub family_id: uuid::Uuid,
    pub token_hash: T1,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct UserByEmailBidx {
    pub id: uuid::Uuid,
    pub role: crate::types::UserRole,
    pub email_enc: Vec<u8>,
    pub password_hash: String,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub struct UserByEmailBidxBorrowed<'a> {
    pub id: uuid::Uuid,
    pub role: crate::types::UserRole,
    pub email_enc: &'a [u8],
    pub password_hash: &'a str,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl<'a> From<UserByEmailBidxBorrowed<'a>> for UserByEmailBidx {
    fn from(
        UserByEmailBidxBorrowed {
            id,
            role,
            email_enc,
            password_hash,
            email_verified_at,
            deleted_at,
        }: UserByEmailBidxBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            role,
            email_enc: email_enc.into(),
            password_hash: password_hash.into(),
            email_verified_at,
            deleted_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct UserById {
    pub id: uuid::Uuid,
    pub role: crate::types::UserRole,
    pub email_enc: Vec<u8>,
    pub password_hash: String,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub struct UserByIdBorrowed<'a> {
    pub id: uuid::Uuid,
    pub role: crate::types::UserRole,
    pub email_enc: &'a [u8],
    pub password_hash: &'a str,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl<'a> From<UserByIdBorrowed<'a>> for UserById {
    fn from(
        UserByIdBorrowed {
            id,
            role,
            email_enc,
            password_hash,
            email_verified_at,
            deleted_at,
        }: UserByIdBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            role,
            email_enc: email_enc.into(),
            password_hash: password_hash.into(),
            email_verified_at,
            deleted_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct RefreshTokenByHash {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub family_id: uuid::Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub rotated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct UserByEmailBidxQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<UserByEmailBidxBorrowed, tokio_postgres::Error>,
    mapper: fn(UserByEmailBidxBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UserByEmailBidxQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(UserByEmailBidxBorrowed) -> R,
    ) -> UserByEmailBidxQuery<'c, 'a, 's, C, R, N> {
        UserByEmailBidxQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct UserByIdQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<UserByIdBorrowed, tokio_postgres::Error>,
    mapper: fn(UserByIdBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UserByIdQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(UserByIdBorrowed) -> R) -> UserByIdQuery<'c, 'a, 's, C, R, N> {
        UserByIdQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct UuidUuidQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<uuid::Uuid, tokio_postgres::Error>,
    mapper: fn(uuid::Uuid) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UuidUuidQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(uuid::Uuid) -> R) -> UuidUuidQuery<'c, 'a, 's, C, R, N> {
        UuidUuidQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct RefreshTokenByHashQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<RefreshTokenByHash, tokio_postgres::Error>,
    mapper: fn(RefreshTokenByHash) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> RefreshTokenByHashQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(RefreshTokenByHash) -> R,
    ) -> RefreshTokenByHashQuery<'c, 'a, 's, C, R, N> {
        RefreshTokenByHashQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct InsertUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_user() -> InsertUserStmt {
    InsertUserStmt(
        "INSERT INTO users (id, role, email_enc, email_bidx, password_hash) VALUES ($1, $2, $3, $4, $5)",
        None,
    )
}
impl InsertUserStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::BytesSql,
        T2: crate::BytesSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        role: &'a crate::types::UserRole,
        email_enc: &'a T1,
        email_bidx: &'a T2,
        password_hash: &'a T3,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, role, email_enc, email_bidx, password_hash])
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::BytesSql,
    T2: crate::BytesSql,
    T3: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertUserParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertUserStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertUserParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.role,
            &params.email_enc,
            &params.email_bidx,
            &params.password_hash,
        ))
    }
}
pub struct UserByEmailBidxStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn user_by_email_bidx() -> UserByEmailBidxStmt {
    UserByEmailBidxStmt(
        "SELECT id, role, email_enc, password_hash, email_verified_at, deleted_at FROM users WHERE email_bidx = $1",
        None,
    )
}
impl UserByEmailBidxStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        email_bidx: &'a T1,
    ) -> UserByEmailBidxQuery<'c, 'a, 's, C, UserByEmailBidx, 1> {
        UserByEmailBidxQuery {
            client,
            params: [email_bidx],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<UserByEmailBidxBorrowed, tokio_postgres::Error> {
                Ok(UserByEmailBidxBorrowed {
                    id: row.try_get(0)?,
                    role: row.try_get(1)?,
                    email_enc: row.try_get(2)?,
                    password_hash: row.try_get(3)?,
                    email_verified_at: row.try_get(4)?,
                    deleted_at: row.try_get(5)?,
                })
            },
            mapper: |it| UserByEmailBidx::from(it),
        }
    }
}
pub struct UserByIdStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn user_by_id() -> UserByIdStmt {
    UserByIdStmt(
        "SELECT id, role, email_enc, password_hash, email_verified_at, deleted_at FROM users WHERE id = $1",
        None,
    )
}
impl UserByIdStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
    ) -> UserByIdQuery<'c, 'a, 's, C, UserById, 1> {
        UserByIdQuery {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<UserByIdBorrowed, tokio_postgres::Error> {
                    Ok(UserByIdBorrowed {
                        id: row.try_get(0)?,
                        role: row.try_get(1)?,
                        email_enc: row.try_get(2)?,
                        password_hash: row.try_get(3)?,
                        email_verified_at: row.try_get(4)?,
                        deleted_at: row.try_get(5)?,
                    })
                },
            mapper: |it| UserById::from(it),
        }
    }
}
pub struct TouchLastLoginStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn touch_last_login() -> TouchLastLoginStmt {
    TouchLastLoginStmt("UPDATE users SET last_login_at = now() WHERE id = $1", None)
}
impl TouchLastLoginStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id]).await
    }
}
pub struct MarkEmailVerifiedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn mark_email_verified() -> MarkEmailVerifiedStmt {
    MarkEmailVerifiedStmt(
        "UPDATE users SET email_verified_at = now() WHERE id = $1 AND email_verified_at IS NULL",
        None,
    )
}
impl MarkEmailVerifiedStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id]).await
    }
}
pub struct UpdatePasswordHashStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_password_hash() -> UpdatePasswordHashStmt {
    UpdatePasswordHashStmt("UPDATE users SET password_hash = $1 WHERE id = $2", None)
}
impl UpdatePasswordHashStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        password_hash: &'a T1,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[password_hash, id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdatePasswordHashParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdatePasswordHashStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdatePasswordHashParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.password_hash, &params.id))
    }
}
pub struct AnonymizeUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn anonymize_user() -> AnonymizeUserStmt {
    AnonymizeUserStmt(
        "UPDATE users SET email_enc = '\\x'::bytea, email_bidx = $1, password_hash = '!', deleted_at = now() WHERE id = $2",
        None,
    )
}
impl AnonymizeUserStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        tombstone_bidx: &'a T1,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[tombstone_bidx, id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        AnonymizeUserParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for AnonymizeUserStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a AnonymizeUserParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.tombstone_bidx, &params.id))
    }
}
pub struct InsertEmailVerificationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_email_verification() -> InsertEmailVerificationStmt {
    InsertEmailVerificationStmt(
        "INSERT INTO email_verifications (token_hash, user_id, expires_at) VALUES ($1, $2, $3)",
        None,
    )
}
impl InsertEmailVerificationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
        user_id: &'a uuid::Uuid,
        expires_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[token_hash, user_id, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertEmailVerificationParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertEmailVerificationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertEmailVerificationParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.token_hash,
            &params.user_id,
            &params.expires_at,
        ))
    }
}
pub struct ConsumeEmailVerificationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn consume_email_verification() -> ConsumeEmailVerificationStmt {
    ConsumeEmailVerificationStmt(
        "UPDATE email_verifications SET used_at = now() WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now() RETURNING user_id",
        None,
    )
}
impl ConsumeEmailVerificationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 1> {
        UuidUuidQuery {
            client,
            params: [token_hash],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct InsertRefreshTokenStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_refresh_token() -> InsertRefreshTokenStmt {
    InsertRefreshTokenStmt(
        "INSERT INTO refresh_tokens (id, user_id, family_id, token_hash, expires_at) VALUES ($1, $2, $3, $4, $5)",
        None,
    )
}
impl InsertRefreshTokenStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        user_id: &'a uuid::Uuid,
        family_id: &'a uuid::Uuid,
        token_hash: &'a T1,
        expires_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, user_id, family_id, token_hash, expires_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::BytesSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertRefreshTokenParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertRefreshTokenStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertRefreshTokenParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.user_id,
            &params.family_id,
            &params.token_hash,
            &params.expires_at,
        ))
    }
}
pub struct RefreshTokenByHashStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn refresh_token_by_hash() -> RefreshTokenByHashStmt {
    RefreshTokenByHashStmt(
        "SELECT id, user_id, family_id, expires_at, rotated_at, revoked_at FROM refresh_tokens WHERE token_hash = $1",
        None,
    )
}
impl RefreshTokenByHashStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::BytesSql>(
        &'s self,
        client: &'c C,
        token_hash: &'a T1,
    ) -> RefreshTokenByHashQuery<'c, 'a, 's, C, RefreshTokenByHash, 1> {
        RefreshTokenByHashQuery {
            client,
            params: [token_hash],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<RefreshTokenByHash, tokio_postgres::Error> {
                    Ok(RefreshTokenByHash {
                        id: row.try_get(0)?,
                        user_id: row.try_get(1)?,
                        family_id: row.try_get(2)?,
                        expires_at: row.try_get(3)?,
                        rotated_at: row.try_get(4)?,
                        revoked_at: row.try_get(5)?,
                    })
                },
            mapper: |it| RefreshTokenByHash::from(it),
        }
    }
}
pub struct RotateRefreshTokenStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn rotate_refresh_token() -> RotateRefreshTokenStmt {
    RotateRefreshTokenStmt(
        "UPDATE refresh_tokens SET rotated_at = now() WHERE id = $1 AND rotated_at IS NULL AND revoked_at IS NULL",
        None,
    )
}
impl RotateRefreshTokenStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id]).await
    }
}
pub struct RevokeRefreshFamilyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_refresh_family() -> RevokeRefreshFamilyStmt {
    RevokeRefreshFamilyStmt(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE family_id = $1 AND revoked_at IS NULL",
        None,
    )
}
impl RevokeRefreshFamilyStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        family_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[family_id]).await
    }
}
pub struct RevokeUserRefreshTokensStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_user_refresh_tokens() -> RevokeUserRefreshTokensStmt {
    RevokeUserRefreshTokensStmt(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
        None,
    )
}
impl RevokeUserRefreshTokensStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        user_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[user_id]).await
    }
}
