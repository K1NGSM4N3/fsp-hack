// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertOidcStateParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub state: T1,
    pub user_id: uuid::Uuid,
    pub pkce_verifier: T2,
    pub nonce: T3,
}
#[derive(Debug)]
pub struct UpsertFspLinkParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub candidate_id: uuid::Uuid,
    pub fsp_id: T1,
    pub linked_via: T2,
}
#[derive(Debug)]
pub struct StoreFspAchievementsParams<T1: crate::JsonSql> {
    pub achievements: T1,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TakeOidcState {
    pub user_id: uuid::Uuid,
    pub pkce_verifier: String,
    pub nonce: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
pub struct TakeOidcStateBorrowed<'a> {
    pub user_id: uuid::Uuid,
    pub pkce_verifier: &'a str,
    pub nonce: &'a str,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<TakeOidcStateBorrowed<'a>> for TakeOidcState {
    fn from(
        TakeOidcStateBorrowed {
            user_id,
            pkce_verifier,
            nonce,
            created_at,
        }: TakeOidcStateBorrowed<'a>,
    ) -> Self {
        Self {
            user_id,
            pkce_verifier: pkce_verifier.into(),
            nonce: nonce.into(),
            created_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FspLink {
    pub candidate_id: uuid::Uuid,
    pub fsp_id: String,
    pub linked_via: String,
    pub linked_at: chrono::DateTime<chrono::Utc>,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub achievements: serde_json::Value,
    pub has_achievements: bool,
}
pub struct FspLinkBorrowed<'a> {
    pub candidate_id: uuid::Uuid,
    pub fsp_id: &'a str,
    pub linked_via: &'a str,
    pub linked_at: chrono::DateTime<chrono::Utc>,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub achievements: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub has_achievements: bool,
}
impl<'a> From<FspLinkBorrowed<'a>> for FspLink {
    fn from(
        FspLinkBorrowed {
            candidate_id,
            fsp_id,
            linked_via,
            linked_at,
            last_sync_at,
            achievements,
            has_achievements,
        }: FspLinkBorrowed<'a>,
    ) -> Self {
        Self {
            candidate_id,
            fsp_id: fsp_id.into(),
            linked_via: linked_via.into(),
            linked_at,
            last_sync_at,
            achievements: serde_json::from_str(achievements.0.get()).unwrap(),
            has_achievements,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct TakeOidcStateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<TakeOidcStateBorrowed, tokio_postgres::Error>,
    mapper: fn(TakeOidcStateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> TakeOidcStateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(TakeOidcStateBorrowed) -> R,
    ) -> TakeOidcStateQuery<'c, 'a, 's, C, R, N> {
        TakeOidcStateQuery {
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
pub struct FspLinkQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<FspLinkBorrowed, tokio_postgres::Error>,
    mapper: fn(FspLinkBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> FspLinkQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(FspLinkBorrowed) -> R) -> FspLinkQuery<'c, 'a, 's, C, R, N> {
        FspLinkQuery {
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
pub struct InsertOidcStateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_oidc_state() -> InsertOidcStateStmt {
    InsertOidcStateStmt(
        "INSERT INTO oidc_states (state, user_id, pkce_verifier, nonce) VALUES ($1, $2, $3, $4)",
        None,
    )
}
impl InsertOidcStateStmt {
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
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        state: &'a T1,
        user_id: &'a uuid::Uuid,
        pkce_verifier: &'a T2,
        nonce: &'a T3,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[state, user_id, pkce_verifier, nonce])
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertOidcStateParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertOidcStateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertOidcStateParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.state,
            &params.user_id,
            &params.pkce_verifier,
            &params.nonce,
        ))
    }
}
pub struct TakeOidcStateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn take_oidc_state() -> TakeOidcStateStmt {
    TakeOidcStateStmt(
        "DELETE FROM oidc_states WHERE state = $1 RETURNING user_id, pkce_verifier, nonce, created_at",
        None,
    )
}
impl TakeOidcStateStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        state: &'a T1,
    ) -> TakeOidcStateQuery<'c, 'a, 's, C, TakeOidcState, 1> {
        TakeOidcStateQuery {
            client,
            params: [state],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<TakeOidcStateBorrowed, tokio_postgres::Error> {
                    Ok(TakeOidcStateBorrowed {
                        user_id: row.try_get(0)?,
                        pkce_verifier: row.try_get(1)?,
                        nonce: row.try_get(2)?,
                        created_at: row.try_get(3)?,
                    })
                },
            mapper: |it| TakeOidcState::from(it),
        }
    }
}
pub struct PurgeOidcStatesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn purge_oidc_states() -> PurgeOidcStatesStmt {
    PurgeOidcStatesStmt(
        "DELETE FROM oidc_states WHERE created_at < now() - interval '15 minutes'",
        None,
    )
}
impl PurgeOidcStatesStmt {
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
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[]).await
    }
}
pub struct FspLinkStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn fsp_link() -> FspLinkStmt {
    FspLinkStmt(
        "SELECT candidate_id, fsp_id, linked_via, linked_at, last_sync_at, achievements, has_achievements FROM fsp_links WHERE candidate_id = $1",
        None,
    )
}
impl FspLinkStmt {
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
        candidate_id: &'a uuid::Uuid,
    ) -> FspLinkQuery<'c, 'a, 's, C, FspLink, 1> {
        FspLinkQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<FspLinkBorrowed, tokio_postgres::Error> {
                    Ok(FspLinkBorrowed {
                        candidate_id: row.try_get(0)?,
                        fsp_id: row.try_get(1)?,
                        linked_via: row.try_get(2)?,
                        linked_at: row.try_get(3)?,
                        last_sync_at: row.try_get(4)?,
                        achievements: row.try_get(5)?,
                        has_achievements: row.try_get(6)?,
                    })
                },
            mapper: |it| FspLink::from(it),
        }
    }
}
pub struct FspLinkOwnerStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn fsp_link_owner() -> FspLinkOwnerStmt {
    FspLinkOwnerStmt("SELECT candidate_id FROM fsp_links WHERE fsp_id = $1", None)
}
impl FspLinkOwnerStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        fsp_id: &'a T1,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 1> {
        UuidUuidQuery {
            client,
            params: [fsp_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct UpsertFspLinkStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn upsert_fsp_link() -> UpsertFspLinkStmt {
    UpsertFspLinkStmt(
        "INSERT INTO fsp_links (candidate_id, fsp_id, linked_via) VALUES ($1, $2, $3) ON CONFLICT (candidate_id) DO UPDATE SET fsp_id = EXCLUDED.fsp_id, linked_via = EXCLUDED.linked_via, linked_at = now(), achievements = '[]', last_sync_at = NULL",
        None,
    )
}
impl UpsertFspLinkStmt {
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
        candidate_id: &'a uuid::Uuid,
        fsp_id: &'a T1,
        linked_via: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[candidate_id, fsp_id, linked_via])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpsertFspLinkParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpsertFspLinkStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpsertFspLinkParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.candidate_id,
            &params.fsp_id,
            &params.linked_via,
        ))
    }
}
pub struct StoreFspAchievementsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn store_fsp_achievements() -> StoreFspAchievementsStmt {
    StoreFspAchievementsStmt(
        "UPDATE fsp_links SET achievements = $1, last_sync_at = now() WHERE candidate_id = $2",
        None,
    )
}
impl StoreFspAchievementsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::JsonSql>(
        &'s self,
        client: &'c C,
        achievements: &'a T1,
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[achievements, candidate_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::JsonSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        StoreFspAchievementsParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for StoreFspAchievementsStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a StoreFspAchievementsParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.achievements, &params.candidate_id))
    }
}
pub struct DeleteFspLinkStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_fsp_link() -> DeleteFspLinkStmt {
    DeleteFspLinkStmt("DELETE FROM fsp_links WHERE candidate_id = $1", None)
}
impl DeleteFspLinkStmt {
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
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[candidate_id]).await
    }
}
