// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertSessionParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::JsonSql,
    T5: crate::JsonSql,
> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub specialization: T1,
    pub claimed_grade: T2,
    pub attempt_kind: T3,
    pub state: T4,
    pub current_item: T5,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub shown_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Clone, Copy, Debug)]
pub struct SessionForUpdateParams {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug)]
pub struct UpdateSessionProgressParams<T1: crate::JsonSql, T2: crate::JsonSql> {
    pub state: T1,
    pub current_item: T2,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub shown_at: chrono::DateTime<chrono::Utc>,
    pub items_count: i32,
    pub id: uuid::Uuid,
}
#[derive(Debug)]
pub struct FinishSessionParams<T1: crate::JsonSql, T2: crate::StringSql> {
    pub status: crate::types::AssessmentStatus,
    pub state: T1,
    pub theta: f64,
    pub se: f64,
    pub measured_grade: Option<T2>,
    pub confident_above_next: bool,
    pub person_fit_lz: f64,
    pub aberrant: bool,
    pub items_count: i32,
    pub finished_at: chrono::DateTime<chrono::Utc>,
    pub id: uuid::Uuid,
}
#[derive(Debug)]
pub struct InsertResponseParams<T1: crate::StringSql, T2: crate::JsonSql> {
    pub session_id: uuid::Uuid,
    pub seq: i32,
    pub template_id: T1,
    pub level: i16,
    pub seed: i64,
    pub answer: Option<T2>,
    pub correct: bool,
    pub timed_out: bool,
    pub elapsed_ms: i32,
    pub theta_after: f64,
    pub se_after: f64,
    pub answered_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Clone, Copy, Debug)]
pub struct SessionStateParams {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SessionForUpdate {
    pub id: uuid::Uuid,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: crate::types::AssessmentStatus,
    pub state: serde_json::Value,
    pub current_item: Option<serde_json::Value>,
    pub current_item_deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub current_item_shown_at: Option<chrono::DateTime<chrono::Utc>>,
    pub items_count: i32,
}
pub struct SessionForUpdateBorrowed<'a> {
    pub id: uuid::Uuid,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub attempt_kind: &'a str,
    pub status: crate::types::AssessmentStatus,
    pub state: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub current_item: Option<postgres_types::Json<&'a serde_json::value::RawValue>>,
    pub current_item_deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub current_item_shown_at: Option<chrono::DateTime<chrono::Utc>>,
    pub items_count: i32,
}
impl<'a> From<SessionForUpdateBorrowed<'a>> for SessionForUpdate {
    fn from(
        SessionForUpdateBorrowed {
            id,
            specialization,
            claimed_grade,
            attempt_kind,
            status,
            state,
            current_item,
            current_item_deadline,
            current_item_shown_at,
            items_count,
        }: SessionForUpdateBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            specialization: specialization.into(),
            claimed_grade: claimed_grade.into(),
            attempt_kind: attempt_kind.into(),
            status,
            state: serde_json::from_str(state.0.get()).unwrap(),
            current_item: current_item.map(|v| serde_json::from_str(v.0.get()).unwrap()),
            current_item_deadline,
            current_item_shown_at,
            items_count,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateSessions {
    pub id: uuid::Uuid,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: crate::types::AssessmentStatus,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<String>,
    pub confident_above_next: Option<bool>,
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub items_count: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub assigned: bool,
}
pub struct CandidateSessionsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub attempt_kind: &'a str,
    pub status: crate::types::AssessmentStatus,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<&'a str>,
    pub confident_above_next: Option<bool>,
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub items_count: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub assigned: bool,
}
impl<'a> From<CandidateSessionsBorrowed<'a>> for CandidateSessions {
    fn from(
        CandidateSessionsBorrowed {
            id,
            specialization,
            claimed_grade,
            attempt_kind,
            status,
            theta,
            se,
            measured_grade,
            confident_above_next,
            person_fit_lz,
            aberrant,
            items_count,
            started_at,
            finished_at,
            assigned,
        }: CandidateSessionsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            specialization: specialization.into(),
            claimed_grade: claimed_grade.into(),
            attempt_kind: attempt_kind.into(),
            status,
            theta,
            se,
            measured_grade: measured_grade.map(|v| v.into()),
            confident_above_next,
            person_fit_lz,
            aberrant,
            items_count,
            started_at,
            finished_at,
            assigned,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SessionState {
    pub id: uuid::Uuid,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: crate::types::AssessmentStatus,
    pub state: serde_json::Value,
    pub current_item: Option<serde_json::Value>,
    pub current_item_deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<String>,
    pub confident_above_next: Option<bool>,
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub items_count: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub assigned: bool,
}
pub struct SessionStateBorrowed<'a> {
    pub id: uuid::Uuid,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub attempt_kind: &'a str,
    pub status: crate::types::AssessmentStatus,
    pub state: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub current_item: Option<postgres_types::Json<&'a serde_json::value::RawValue>>,
    pub current_item_deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<&'a str>,
    pub confident_above_next: Option<bool>,
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub items_count: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub assigned: bool,
}
impl<'a> From<SessionStateBorrowed<'a>> for SessionState {
    fn from(
        SessionStateBorrowed {
            id,
            specialization,
            claimed_grade,
            attempt_kind,
            status,
            state,
            current_item,
            current_item_deadline,
            theta,
            se,
            measured_grade,
            confident_above_next,
            person_fit_lz,
            aberrant,
            items_count,
            started_at,
            finished_at,
            assigned,
        }: SessionStateBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            specialization: specialization.into(),
            claimed_grade: claimed_grade.into(),
            attempt_kind: attempt_kind.into(),
            status,
            state: serde_json::from_str(state.0.get()).unwrap(),
            current_item: current_item.map(|v| serde_json::from_str(v.0.get()).unwrap()),
            current_item_deadline,
            theta,
            se,
            measured_grade: measured_grade.map(|v| v.into()),
            confident_above_next,
            person_fit_lz,
            aberrant,
            items_count,
            started_at,
            finished_at,
            assigned,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
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
pub struct SessionForUpdateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<SessionForUpdateBorrowed, tokio_postgres::Error>,
    mapper: fn(SessionForUpdateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> SessionForUpdateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(SessionForUpdateBorrowed) -> R,
    ) -> SessionForUpdateQuery<'c, 'a, 's, C, R, N> {
        SessionForUpdateQuery {
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
pub struct CandidateSessionsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CandidateSessionsBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateSessionsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateSessionsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateSessionsBorrowed) -> R,
    ) -> CandidateSessionsQuery<'c, 'a, 's, C, R, N> {
        CandidateSessionsQuery {
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
pub struct SessionStateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<SessionStateBorrowed, tokio_postgres::Error>,
    mapper: fn(SessionStateBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> SessionStateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(SessionStateBorrowed) -> R,
    ) -> SessionStateQuery<'c, 'a, 's, C, R, N> {
        SessionStateQuery {
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
pub struct ExpireAbandonedSessionsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn expire_abandoned_sessions() -> ExpireAbandonedSessionsStmt {
    ExpireAbandonedSessionsStmt(
        "UPDATE assessment_sessions SET status = 'expired', finished_at = now() WHERE candidate_id = $1 AND status = 'in_progress' AND coalesce(current_item_deadline, started_at) < now() - interval '30 minutes'",
        None,
    )
}
impl ExpireAbandonedSessionsStmt {
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
pub struct ActiveSessionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn active_session() -> ActiveSessionStmt {
    ActiveSessionStmt(
        "SELECT id FROM assessment_sessions WHERE candidate_id = $1 AND status = 'in_progress'",
        None,
    )
}
impl ActiveSessionStmt {
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
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 1> {
        UuidUuidQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct InsertSessionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_session() -> InsertSessionStmt {
    InsertSessionStmt(
        "INSERT INTO assessment_sessions (id, candidate_id, specialization, claimed_grade, attempt_kind, state, current_item, current_item_deadline, current_item_shown_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        None,
    )
}
impl InsertSessionStmt {
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
        T4: crate::JsonSql,
        T5: crate::JsonSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        specialization: &'a T1,
        claimed_grade: &'a T2,
        attempt_kind: &'a T3,
        state: &'a T4,
        current_item: &'a T5,
        deadline: &'a chrono::DateTime<chrono::Utc>,
        shown_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    candidate_id,
                    specialization,
                    claimed_grade,
                    attempt_kind,
                    state,
                    current_item,
                    deadline,
                    shown_at,
                ],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::JsonSql,
    T5: crate::JsonSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertSessionParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertSessionStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertSessionParams<T1, T2, T3, T4, T5>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.candidate_id,
            &params.specialization,
            &params.claimed_grade,
            &params.attempt_kind,
            &params.state,
            &params.current_item,
            &params.deadline,
            &params.shown_at,
        ))
    }
}
pub struct SessionForUpdateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn session_for_update() -> SessionForUpdateStmt {
    SessionForUpdateStmt(
        "SELECT id, specialization, claimed_grade, attempt_kind, status, state, current_item, current_item_deadline, current_item_shown_at, items_count FROM assessment_sessions WHERE id = $1 AND candidate_id = $2 FOR UPDATE",
        None,
    )
}
impl SessionForUpdateStmt {
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
        candidate_id: &'a uuid::Uuid,
    ) -> SessionForUpdateQuery<'c, 'a, 's, C, SessionForUpdate, 2> {
        SessionForUpdateQuery {
            client,
            params: [id, candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<SessionForUpdateBorrowed, tokio_postgres::Error> {
                Ok(SessionForUpdateBorrowed {
                    id: row.try_get(0)?,
                    specialization: row.try_get(1)?,
                    claimed_grade: row.try_get(2)?,
                    attempt_kind: row.try_get(3)?,
                    status: row.try_get(4)?,
                    state: row.try_get(5)?,
                    current_item: row.try_get(6)?,
                    current_item_deadline: row.try_get(7)?,
                    current_item_shown_at: row.try_get(8)?,
                    items_count: row.try_get(9)?,
                })
            },
            mapper: |it| SessionForUpdate::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        SessionForUpdateParams,
        SessionForUpdateQuery<'c, 'a, 's, C, SessionForUpdate, 2>,
        C,
    > for SessionForUpdateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a SessionForUpdateParams,
    ) -> SessionForUpdateQuery<'c, 'a, 's, C, SessionForUpdate, 2> {
        self.bind(client, &params.id, &params.candidate_id)
    }
}
pub struct UpdateSessionProgressStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_session_progress() -> UpdateSessionProgressStmt {
    UpdateSessionProgressStmt(
        "UPDATE assessment_sessions SET state = $1, current_item = $2, current_item_deadline = $3, current_item_shown_at = $4, items_count = $5 WHERE id = $6",
        None,
    )
}
impl UpdateSessionProgressStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::JsonSql, T2: crate::JsonSql>(
        &'s self,
        client: &'c C,
        state: &'a T1,
        current_item: &'a T2,
        deadline: &'a chrono::DateTime<chrono::Utc>,
        shown_at: &'a chrono::DateTime<chrono::Utc>,
        items_count: &'a i32,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[state, current_item, deadline, shown_at, items_count, id],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::JsonSql, T2: crate::JsonSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdateSessionProgressParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdateSessionProgressStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdateSessionProgressParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.state,
            &params.current_item,
            &params.deadline,
            &params.shown_at,
            &params.items_count,
            &params.id,
        ))
    }
}
pub struct FinishSessionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn finish_session() -> FinishSessionStmt {
    FinishSessionStmt(
        "UPDATE assessment_sessions SET status = $1, state = $2, current_item = NULL, current_item_deadline = NULL, theta = $3, se = $4, measured_grade = $5, confident_above_next = $6, person_fit_lz = $7, aberrant = $8, items_count = $9, finished_at = $10 WHERE id = $11",
        None,
    )
}
impl FinishSessionStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::JsonSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        status: &'a crate::types::AssessmentStatus,
        state: &'a T1,
        theta: &'a f64,
        se: &'a f64,
        measured_grade: &'a Option<T2>,
        confident_above_next: &'a bool,
        person_fit_lz: &'a f64,
        aberrant: &'a bool,
        items_count: &'a i32,
        finished_at: &'a chrono::DateTime<chrono::Utc>,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    status,
                    state,
                    theta,
                    se,
                    measured_grade,
                    confident_above_next,
                    person_fit_lz,
                    aberrant,
                    items_count,
                    finished_at,
                    id,
                ],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::JsonSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        FinishSessionParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for FinishSessionStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a FinishSessionParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.status,
            &params.state,
            &params.theta,
            &params.se,
            &params.measured_grade,
            &params.confident_above_next,
            &params.person_fit_lz,
            &params.aberrant,
            &params.items_count,
            &params.finished_at,
            &params.id,
        ))
    }
}
pub struct InsertResponseStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_response() -> InsertResponseStmt {
    InsertResponseStmt(
        "INSERT INTO assessment_responses (session_id, seq, template_id, level, seed, answer, correct, timed_out, elapsed_ms, theta_after, se_after, answered_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        None,
    )
}
impl InsertResponseStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::JsonSql>(
        &'s self,
        client: &'c C,
        session_id: &'a uuid::Uuid,
        seq: &'a i32,
        template_id: &'a T1,
        level: &'a i16,
        seed: &'a i64,
        answer: &'a Option<T2>,
        correct: &'a bool,
        timed_out: &'a bool,
        elapsed_ms: &'a i32,
        theta_after: &'a f64,
        se_after: &'a f64,
        answered_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    session_id,
                    seq,
                    template_id,
                    level,
                    seed,
                    answer,
                    correct,
                    timed_out,
                    elapsed_ms,
                    theta_after,
                    se_after,
                    answered_at,
                ],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::JsonSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertResponseParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertResponseStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertResponseParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.session_id,
            &params.seq,
            &params.template_id,
            &params.level,
            &params.seed,
            &params.answer,
            &params.correct,
            &params.timed_out,
            &params.elapsed_ms,
            &params.theta_after,
            &params.se_after,
            &params.answered_at,
        ))
    }
}
pub struct CandidateSessionsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_sessions() -> CandidateSessionsStmt {
    CandidateSessionsStmt(
        "SELECT s.id, s.specialization, s.claimed_grade, s.attempt_kind, s.status, s.theta, s.se, s.measured_grade, s.confident_above_next, s.person_fit_lz, s.aberrant, s.items_count, s.started_at, s.finished_at, EXISTS (SELECT 1 FROM grade_assignments g WHERE g.session_id = s.id) AS assigned FROM assessment_sessions s WHERE s.candidate_id = $1 ORDER BY s.started_at DESC",
        None,
    )
}
impl CandidateSessionsStmt {
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
    ) -> CandidateSessionsQuery<'c, 'a, 's, C, CandidateSessions, 1> {
        CandidateSessionsQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateSessionsBorrowed, tokio_postgres::Error> {
                Ok(CandidateSessionsBorrowed {
                    id: row.try_get(0)?,
                    specialization: row.try_get(1)?,
                    claimed_grade: row.try_get(2)?,
                    attempt_kind: row.try_get(3)?,
                    status: row.try_get(4)?,
                    theta: row.try_get(5)?,
                    se: row.try_get(6)?,
                    measured_grade: row.try_get(7)?,
                    confident_above_next: row.try_get(8)?,
                    person_fit_lz: row.try_get(9)?,
                    aberrant: row.try_get(10)?,
                    items_count: row.try_get(11)?,
                    started_at: row.try_get(12)?,
                    finished_at: row.try_get(13)?,
                    assigned: row.try_get(14)?,
                })
            },
            mapper: |it| CandidateSessions::from(it),
        }
    }
}
pub struct SessionStateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn session_state() -> SessionStateStmt {
    SessionStateStmt(
        "SELECT s.id, s.specialization, s.claimed_grade, s.attempt_kind, s.status, s.state, s.current_item, s.current_item_deadline, s.theta, s.se, s.measured_grade, s.confident_above_next, s.person_fit_lz, s.aberrant, s.items_count, s.started_at, s.finished_at, EXISTS (SELECT 1 FROM grade_assignments g WHERE g.session_id = s.id) AS assigned FROM assessment_sessions s WHERE s.id = $1 AND s.candidate_id = $2",
        None,
    )
}
impl SessionStateStmt {
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
        candidate_id: &'a uuid::Uuid,
    ) -> SessionStateQuery<'c, 'a, 's, C, SessionState, 2> {
        SessionStateQuery {
            client,
            params: [id, candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<SessionStateBorrowed, tokio_postgres::Error> {
                    Ok(SessionStateBorrowed {
                        id: row.try_get(0)?,
                        specialization: row.try_get(1)?,
                        claimed_grade: row.try_get(2)?,
                        attempt_kind: row.try_get(3)?,
                        status: row.try_get(4)?,
                        state: row.try_get(5)?,
                        current_item: row.try_get(6)?,
                        current_item_deadline: row.try_get(7)?,
                        theta: row.try_get(8)?,
                        se: row.try_get(9)?,
                        measured_grade: row.try_get(10)?,
                        confident_above_next: row.try_get(11)?,
                        person_fit_lz: row.try_get(12)?,
                        aberrant: row.try_get(13)?,
                        items_count: row.try_get(14)?,
                        started_at: row.try_get(15)?,
                        finished_at: row.try_get(16)?,
                        assigned: row.try_get(17)?,
                    })
                },
            mapper: |it| SessionState::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        SessionStateParams,
        SessionStateQuery<'c, 'a, 's, C, SessionState, 2>,
        C,
    > for SessionStateStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a SessionStateParams,
    ) -> SessionStateQuery<'c, 'a, 's, C, SessionState, 2> {
        self.bind(client, &params.id, &params.candidate_id)
    }
}
