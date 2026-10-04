// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertGradeAssignmentParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub specialization: T1,
    pub grade: T2,
    pub theta: f64,
    pub se: f64,
    pub session_id: Option<uuid::Uuid>,
    pub reason: T3,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug)]
pub struct SetCandidateCategoryParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub specialization: T1,
    pub grade: T2,
    pub theta: f64,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct GradeHistory {
    pub id: uuid::Uuid,
    pub specialization: String,
    pub grade: String,
    pub theta: f64,
    pub se: f64,
    pub session_id: Option<uuid::Uuid>,
    pub reason: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}
pub struct GradeHistoryBorrowed<'a> {
    pub id: uuid::Uuid,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub theta: f64,
    pub se: f64,
    pub session_id: Option<uuid::Uuid>,
    pub reason: &'a str,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<GradeHistoryBorrowed<'a>> for GradeHistory {
    fn from(
        GradeHistoryBorrowed {
            id,
            specialization,
            grade,
            theta,
            se,
            session_id,
            reason,
            assigned_at,
        }: GradeHistoryBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            specialization: specialization.into(),
            grade: grade.into(),
            theta,
            se,
            session_id,
            reason: reason.into(),
            assigned_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct GradeHistoryQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<GradeHistoryBorrowed, tokio_postgres::Error>,
    mapper: fn(GradeHistoryBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> GradeHistoryQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(GradeHistoryBorrowed) -> R,
    ) -> GradeHistoryQuery<'c, 'a, 's, C, R, N> {
        GradeHistoryQuery {
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
pub struct InsertGradeAssignmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_grade_assignment() -> InsertGradeAssignmentStmt {
    InsertGradeAssignmentStmt(
        "INSERT INTO grade_assignments (id, candidate_id, specialization, grade, theta, se, session_id, reason, assigned_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        None,
    )
}
impl InsertGradeAssignmentStmt {
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
        id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        specialization: &'a T1,
        grade: &'a T2,
        theta: &'a f64,
        se: &'a f64,
        session_id: &'a Option<uuid::Uuid>,
        reason: &'a T3,
        assigned_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    candidate_id,
                    specialization,
                    grade,
                    theta,
                    se,
                    session_id,
                    reason,
                    assigned_at,
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
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertGradeAssignmentParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertGradeAssignmentStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertGradeAssignmentParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.candidate_id,
            &params.specialization,
            &params.grade,
            &params.theta,
            &params.se,
            &params.session_id,
            &params.reason,
            &params.assigned_at,
        ))
    }
}
pub struct SetCandidateCategoryStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_candidate_category() -> SetCandidateCategoryStmt {
    SetCandidateCategoryStmt(
        "UPDATE candidates SET specialization = $1, grade = $2, theta = $3, category_assigned_at = $4, last_activity_at = greatest(last_activity_at, $4), updated_at = now() WHERE user_id = $5",
        None,
    )
}
impl SetCandidateCategoryStmt {
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
        specialization: &'a T1,
        grade: &'a T2,
        theta: &'a f64,
        assigned_at: &'a chrono::DateTime<chrono::Utc>,
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[specialization, grade, theta, assigned_at, candidate_id],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetCandidateCategoryParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetCandidateCategoryStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetCandidateCategoryParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.specialization,
            &params.grade,
            &params.theta,
            &params.assigned_at,
            &params.candidate_id,
        ))
    }
}
pub struct GradeHistoryStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn grade_history() -> GradeHistoryStmt {
    GradeHistoryStmt(
        "SELECT id, specialization, grade, theta, se, session_id, reason, assigned_at FROM grade_assignments WHERE candidate_id = $1 ORDER BY assigned_at DESC",
        None,
    )
}
impl GradeHistoryStmt {
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
    ) -> GradeHistoryQuery<'c, 'a, 's, C, GradeHistory, 1> {
        GradeHistoryQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<GradeHistoryBorrowed, tokio_postgres::Error> {
                    Ok(GradeHistoryBorrowed {
                        id: row.try_get(0)?,
                        specialization: row.try_get(1)?,
                        grade: row.try_get(2)?,
                        theta: row.try_get(3)?,
                        se: row.try_get(4)?,
                        session_id: row.try_get(5)?,
                        reason: row.try_get(6)?,
                        assigned_at: row.try_get(7)?,
                    })
                },
            mapper: |it| GradeHistory::from(it),
        }
    }
}
