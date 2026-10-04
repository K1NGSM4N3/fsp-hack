// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertUserConsentParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub ip: Option<T1>,
    pub user_agent: Option<T2>,
}
#[derive(Clone, Copy, Debug)]
pub struct RevokeUserConsentParams {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CurrentConsentDocuments {
    pub id: uuid::Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub body: String,
    pub required_for: Vec<crate::types::UserRole>,
    pub published_at: chrono::DateTime<chrono::Utc>,
}
pub struct CurrentConsentDocumentsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub kind: &'a str,
    pub version: i32,
    pub title: &'a str,
    pub body: &'a str,
    pub required_for: crate::ArrayIterator<'a, crate::types::UserRole>,
    pub published_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<CurrentConsentDocumentsBorrowed<'a>> for CurrentConsentDocuments {
    fn from(
        CurrentConsentDocumentsBorrowed {
            id,
            kind,
            version,
            title,
            body,
            required_for,
            published_at,
        }: CurrentConsentDocumentsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            kind: kind.into(),
            version,
            title: title.into(),
            body: body.into(),
            required_for: required_for.map(|v| v).collect(),
            published_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct UserConsents {
    pub id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub struct UserConsentsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub document_id: uuid::Uuid,
    pub kind: &'a str,
    pub version: i32,
    pub title: &'a str,
    pub granted_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl<'a> From<UserConsentsBorrowed<'a>> for UserConsents {
    fn from(
        UserConsentsBorrowed {
            id,
            document_id,
            kind,
            version,
            title,
            granted_at,
            revoked_at,
        }: UserConsentsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            document_id,
            kind: kind.into(),
            version,
            title: title.into(),
            granted_at,
            revoked_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CurrentConsentDocumentsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CurrentConsentDocumentsBorrowed, tokio_postgres::Error>,
    mapper: fn(CurrentConsentDocumentsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CurrentConsentDocumentsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CurrentConsentDocumentsBorrowed) -> R,
    ) -> CurrentConsentDocumentsQuery<'c, 'a, 's, C, R, N> {
        CurrentConsentDocumentsQuery {
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
pub struct UserConsentsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<UserConsentsBorrowed, tokio_postgres::Error>,
    mapper: fn(UserConsentsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> UserConsentsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(UserConsentsBorrowed) -> R,
    ) -> UserConsentsQuery<'c, 'a, 's, C, R, N> {
        UserConsentsQuery {
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
pub struct StringQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<&str, tokio_postgres::Error>,
    mapper: fn(&str) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> StringQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(&str) -> R) -> StringQuery<'c, 'a, 's, C, R, N> {
        StringQuery {
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
pub struct CurrentConsentDocumentsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn current_consent_documents() -> CurrentConsentDocumentsStmt {
    CurrentConsentDocumentsStmt(
        "SELECT DISTINCT ON (kind) id, kind, version, title, body, required_for, published_at FROM consent_documents ORDER BY kind, version DESC",
        None,
    )
}
impl CurrentConsentDocumentsStmt {
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
    ) -> CurrentConsentDocumentsQuery<'c, 'a, 's, C, CurrentConsentDocuments, 0> {
        CurrentConsentDocumentsQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CurrentConsentDocumentsBorrowed, tokio_postgres::Error> {
                Ok(CurrentConsentDocumentsBorrowed {
                    id: row.try_get(0)?,
                    kind: row.try_get(1)?,
                    version: row.try_get(2)?,
                    title: row.try_get(3)?,
                    body: row.try_get(4)?,
                    required_for: row.try_get(5)?,
                    published_at: row.try_get(6)?,
                })
            },
            mapper: |it| CurrentConsentDocuments::from(it),
        }
    }
}
pub struct InsertUserConsentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_user_consent() -> InsertUserConsentStmt {
    InsertUserConsentStmt(
        "INSERT INTO user_consents (id, user_id, document_id, ip, user_agent) VALUES ($1, $2, $3, $4, $5)",
        None,
    )
}
impl InsertUserConsentStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        user_id: &'a uuid::Uuid,
        document_id: &'a uuid::Uuid,
        ip: &'a Option<T1>,
        user_agent: &'a Option<T2>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, user_id, document_id, ip, user_agent])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertUserConsentParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertUserConsentStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertUserConsentParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.user_id,
            &params.document_id,
            &params.ip,
            &params.user_agent,
        ))
    }
}
pub struct UserConsentsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn user_consents() -> UserConsentsStmt {
    UserConsentsStmt(
        "SELECT uc.id, uc.document_id, d.kind, d.version, d.title, uc.granted_at, uc.revoked_at FROM user_consents uc JOIN consent_documents d ON d.id = uc.document_id WHERE uc.user_id = $1 ORDER BY uc.granted_at DESC",
        None,
    )
}
impl UserConsentsStmt {
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
        user_id: &'a uuid::Uuid,
    ) -> UserConsentsQuery<'c, 'a, 's, C, UserConsents, 1> {
        UserConsentsQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<UserConsentsBorrowed, tokio_postgres::Error> {
                    Ok(UserConsentsBorrowed {
                        id: row.try_get(0)?,
                        document_id: row.try_get(1)?,
                        kind: row.try_get(2)?,
                        version: row.try_get(3)?,
                        title: row.try_get(4)?,
                        granted_at: row.try_get(5)?,
                        revoked_at: row.try_get(6)?,
                    })
                },
            mapper: |it| UserConsents::from(it),
        }
    }
}
pub struct RevokeUserConsentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn revoke_user_consent() -> RevokeUserConsentStmt {
    RevokeUserConsentStmt(
        "UPDATE user_consents uc SET revoked_at = now() FROM consent_documents d WHERE uc.id = $1 AND uc.user_id = $2 AND uc.revoked_at IS NULL AND d.id = uc.document_id RETURNING d.kind",
        None,
    )
}
impl RevokeUserConsentStmt {
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
        user_id: &'a uuid::Uuid,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        StringQuery {
            client,
            params: [id, user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it.into(),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        RevokeUserConsentParams,
        StringQuery<'c, 'a, 's, C, String, 2>,
        C,
    > for RevokeUserConsentStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a RevokeUserConsentParams,
    ) -> StringQuery<'c, 'a, 's, C, String, 2> {
        self.bind(client, &params.id, &params.user_id)
    }
}
