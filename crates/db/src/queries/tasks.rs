// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertTaskParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::ArraySql<Item = T2>,
    T4: crate::StringSql,
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::StringSql,
    T8: crate::StringSql,
> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub specialization: T1,
    pub grades: T3,
    pub skills: T5,
    pub title: T6,
    pub body: T7,
    pub kind: T8,
}
#[derive(Clone, Copy, Debug)]
pub struct SetTaskActiveParams {
    pub active: bool,
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct TaskSubmissionsParams {
    pub task_id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Debug)]
pub struct ReviewSubmissionParams<T1: crate::StringSql> {
    pub rating: i16,
    pub review_comment: Option<T1>,
    pub id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct InsertAssignmentParams {
    pub id: uuid::Uuid,
    pub task_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub due_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug)]
pub struct SubmitAssignmentParams<T1: crate::StringSql> {
    pub answer: T1,
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct SkipAssignmentParams {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyTasks {
    pub id: uuid::Uuid,
    pub specialization: String,
    pub grades: Vec<String>,
    pub skills: Vec<String>,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub submissions: i64,
    pub reviewed: i64,
}
pub struct CompanyTasksBorrowed<'a> {
    pub id: uuid::Uuid,
    pub specialization: &'a str,
    pub grades: crate::ArrayIterator<'a, &'a str>,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub title: &'a str,
    pub body: &'a str,
    pub kind: &'a str,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub submissions: i64,
    pub reviewed: i64,
}
impl<'a> From<CompanyTasksBorrowed<'a>> for CompanyTasks {
    fn from(
        CompanyTasksBorrowed {
            id,
            specialization,
            grades,
            skills,
            title,
            body,
            kind,
            active,
            created_at,
            submissions,
            reviewed,
        }: CompanyTasksBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            specialization: specialization.into(),
            grades: grades.map(|v| v.into()).collect(),
            skills: skills.map(|v| v.into()).collect(),
            title: title.into(),
            body: body.into(),
            kind: kind.into(),
            active,
            created_at,
            submissions,
            reviewed,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TaskSubmissions {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub status: crate::types::TaskAssignmentStatus,
    pub answer: Option<String>,
    pub rating: Option<i16>,
    pub review_comment: Option<String>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub struct TaskSubmissionsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: &'a str,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub status: crate::types::TaskAssignmentStatus,
    pub answer: Option<&'a str>,
    pub rating: Option<i16>,
    pub review_comment: Option<&'a str>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl<'a> From<TaskSubmissionsBorrowed<'a>> for TaskSubmissions {
    fn from(
        TaskSubmissionsBorrowed {
            id,
            candidate_id,
            public_code,
            specialization,
            grade,
            status,
            answer,
            rating,
            review_comment,
            submitted_at,
        }: TaskSubmissionsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            candidate_id,
            public_code: public_code.into(),
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
            status,
            answer: answer.map(|v| v.into()),
            rating,
            review_comment: review_comment.map(|v| v.into()),
            submitted_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateAssignments {
    pub id: uuid::Uuid,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub skills: Vec<String>,
    pub company_name: Option<String>,
    pub status: crate::types::TaskAssignmentStatus,
    pub answer: Option<String>,
    pub rating: Option<i16>,
    pub review_comment: Option<String>,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub due_at: chrono::DateTime<chrono::Utc>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub struct CandidateAssignmentsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub title: &'a str,
    pub body: &'a str,
    pub kind: &'a str,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub company_name: Option<&'a str>,
    pub status: crate::types::TaskAssignmentStatus,
    pub answer: Option<&'a str>,
    pub rating: Option<i16>,
    pub review_comment: Option<&'a str>,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub due_at: chrono::DateTime<chrono::Utc>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl<'a> From<CandidateAssignmentsBorrowed<'a>> for CandidateAssignments {
    fn from(
        CandidateAssignmentsBorrowed {
            id,
            title,
            body,
            kind,
            skills,
            company_name,
            status,
            answer,
            rating,
            review_comment,
            assigned_at,
            due_at,
            submitted_at,
            reviewed_at,
        }: CandidateAssignmentsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            body: body.into(),
            kind: kind.into(),
            skills: skills.map(|v| v.into()).collect(),
            company_name: company_name.map(|v| v.into()),
            status,
            answer: answer.map(|v| v.into()),
            rating,
            review_comment: review_comment.map(|v| v.into()),
            assigned_at,
            due_at,
            submitted_at,
            reviewed_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CompanyTasksQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyTasksBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyTasksBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyTasksQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyTasksBorrowed) -> R,
    ) -> CompanyTasksQuery<'c, 'a, 's, C, R, N> {
        CompanyTasksQuery {
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
pub struct TaskSubmissionsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<TaskSubmissionsBorrowed, tokio_postgres::Error>,
    mapper: fn(TaskSubmissionsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> TaskSubmissionsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(TaskSubmissionsBorrowed) -> R,
    ) -> TaskSubmissionsQuery<'c, 'a, 's, C, R, N> {
        TaskSubmissionsQuery {
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
pub struct OptionchronoDateTimechronoUtcQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(
        &tokio_postgres::Row,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, tokio_postgres::Error>,
    mapper: fn(Option<chrono::DateTime<chrono::Utc>>) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> OptionchronoDateTimechronoUtcQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(Option<chrono::DateTime<chrono::Utc>>) -> R,
    ) -> OptionchronoDateTimechronoUtcQuery<'c, 'a, 's, C, R, N> {
        OptionchronoDateTimechronoUtcQuery {
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
pub struct I64Query<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<i64, tokio_postgres::Error>,
    mapper: fn(i64) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> I64Query<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(self, mapper: fn(i64) -> R) -> I64Query<'c, 'a, 's, C, R, N> {
        I64Query {
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
pub struct CandidateAssignmentsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CandidateAssignmentsBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateAssignmentsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateAssignmentsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateAssignmentsBorrowed) -> R,
    ) -> CandidateAssignmentsQuery<'c, 'a, 's, C, R, N> {
        CandidateAssignmentsQuery {
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
pub struct InsertTaskStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_task() -> InsertTaskStmt {
    InsertTaskStmt(
        "INSERT INTO tasks (id, company_id, specialization, grades, skills, title, body, kind) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        None,
    )
}
impl InsertTaskStmt {
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
        T3: crate::ArraySql<Item = T2>,
        T4: crate::StringSql,
        T5: crate::ArraySql<Item = T4>,
        T6: crate::StringSql,
        T7: crate::StringSql,
        T8: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
        specialization: &'a T1,
        grades: &'a T3,
        skills: &'a T5,
        title: &'a T6,
        body: &'a T7,
        kind: &'a T8,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    company_id,
                    specialization,
                    grades,
                    skills,
                    title,
                    body,
                    kind,
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
    T3: crate::ArraySql<Item = T2>,
    T4: crate::StringSql,
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::StringSql,
    T8: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertTaskParams<T1, T2, T3, T4, T5, T6, T7, T8>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertTaskStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertTaskParams<T1, T2, T3, T4, T5, T6, T7, T8>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.company_id,
            &params.specialization,
            &params.grades,
            &params.skills,
            &params.title,
            &params.body,
            &params.kind,
        ))
    }
}
pub struct CompanyTasksStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_tasks() -> CompanyTasksStmt {
    CompanyTasksStmt(
        "SELECT t.id, t.specialization, t.grades, t.skills, t.title, t.body, t.kind, t.active, t.created_at, count(a.id) FILTER (WHERE a.status IN ('submitted', 'reviewed')) AS submissions, count(a.id) FILTER (WHERE a.status = 'reviewed') AS reviewed FROM tasks t LEFT JOIN task_assignments a ON a.task_id = t.id WHERE t.company_id = $1 GROUP BY t.id ORDER BY t.created_at DESC",
        None,
    )
}
impl CompanyTasksStmt {
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
        company_id: &'a uuid::Uuid,
    ) -> CompanyTasksQuery<'c, 'a, 's, C, CompanyTasks, 1> {
        CompanyTasksQuery {
            client,
            params: [company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CompanyTasksBorrowed, tokio_postgres::Error> {
                    Ok(CompanyTasksBorrowed {
                        id: row.try_get(0)?,
                        specialization: row.try_get(1)?,
                        grades: row.try_get(2)?,
                        skills: row.try_get(3)?,
                        title: row.try_get(4)?,
                        body: row.try_get(5)?,
                        kind: row.try_get(6)?,
                        active: row.try_get(7)?,
                        created_at: row.try_get(8)?,
                        submissions: row.try_get(9)?,
                        reviewed: row.try_get(10)?,
                    })
                },
            mapper: |it| CompanyTasks::from(it),
        }
    }
}
pub struct SetTaskActiveStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_task_active() -> SetTaskActiveStmt {
    SetTaskActiveStmt(
        "UPDATE tasks SET active = $1 WHERE id = $2 AND company_id = $3",
        None,
    )
}
impl SetTaskActiveStmt {
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
        active: &'a bool,
        id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[active, id, company_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetTaskActiveParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetTaskActiveStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetTaskActiveParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.active, &params.id, &params.company_id))
    }
}
pub struct TaskSubmissionsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn task_submissions() -> TaskSubmissionsStmt {
    TaskSubmissionsStmt(
        "SELECT a.id, a.candidate_id, c.public_code, c.specialization, c.grade, a.status, a.answer, a.rating, a.review_comment, a.submitted_at FROM task_assignments a JOIN tasks t ON t.id = a.task_id JOIN candidates c ON c.user_id = a.candidate_id WHERE t.id = $1 AND t.company_id = $2 AND a.status IN ('submitted', 'reviewed') ORDER BY a.submitted_at DESC",
        None,
    )
}
impl TaskSubmissionsStmt {
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
        task_id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
    ) -> TaskSubmissionsQuery<'c, 'a, 's, C, TaskSubmissions, 2> {
        TaskSubmissionsQuery {
            client,
            params: [task_id, company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<TaskSubmissionsBorrowed, tokio_postgres::Error> {
                Ok(TaskSubmissionsBorrowed {
                    id: row.try_get(0)?,
                    candidate_id: row.try_get(1)?,
                    public_code: row.try_get(2)?,
                    specialization: row.try_get(3)?,
                    grade: row.try_get(4)?,
                    status: row.try_get(5)?,
                    answer: row.try_get(6)?,
                    rating: row.try_get(7)?,
                    review_comment: row.try_get(8)?,
                    submitted_at: row.try_get(9)?,
                })
            },
            mapper: |it| TaskSubmissions::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        TaskSubmissionsParams,
        TaskSubmissionsQuery<'c, 'a, 's, C, TaskSubmissions, 2>,
        C,
    > for TaskSubmissionsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a TaskSubmissionsParams,
    ) -> TaskSubmissionsQuery<'c, 'a, 's, C, TaskSubmissions, 2> {
        self.bind(client, &params.task_id, &params.company_id)
    }
}
pub struct ReviewSubmissionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn review_submission() -> ReviewSubmissionStmt {
    ReviewSubmissionStmt(
        "UPDATE task_assignments a SET status = 'reviewed', rating = $1, review_comment = $2, reviewed_at = now() FROM tasks t WHERE a.id = $3 AND a.task_id = t.id AND t.id = $4 AND t.company_id = $5 AND a.status IN ('submitted', 'reviewed')",
        None,
    )
}
impl ReviewSubmissionStmt {
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
        rating: &'a i16,
        review_comment: &'a Option<T1>,
        id: &'a uuid::Uuid,
        task_id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[rating, review_comment, id, task_id, company_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        ReviewSubmissionParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for ReviewSubmissionStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a ReviewSubmissionParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.rating,
            &params.review_comment,
            &params.id,
            &params.task_id,
            &params.company_id,
        ))
    }
}
pub struct ExpireAssignmentsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn expire_assignments() -> ExpireAssignmentsStmt {
    ExpireAssignmentsStmt(
        "UPDATE task_assignments SET status = 'expired' WHERE candidate_id = $1 AND status = 'assigned' AND due_at < now()",
        None,
    )
}
impl ExpireAssignmentsStmt {
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
pub struct LastAssignmentAtStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn last_assignment_at() -> LastAssignmentAtStmt {
    LastAssignmentAtStmt(
        "SELECT max(assigned_at) AS assigned_at FROM task_assignments WHERE candidate_id = $1",
        None,
    )
}
impl LastAssignmentAtStmt {
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
    ) -> OptionchronoDateTimechronoUtcQuery<'c, 'a, 's, C, Option<chrono::DateTime<chrono::Utc>>, 1>
    {
        OptionchronoDateTimechronoUtcQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct HasOpenAssignmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn has_open_assignment() -> HasOpenAssignmentStmt {
    HasOpenAssignmentStmt(
        "SELECT count(*) AS open FROM task_assignments WHERE candidate_id = $1 AND status = 'assigned'",
        None,
    )
}
impl HasOpenAssignmentStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 1> {
        I64Query {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct PickTaskForStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn pick_task_for() -> PickTaskForStmt {
    PickTaskForStmt(
        "SELECT t.id FROM tasks t, candidates c WHERE c.user_id = $1 AND t.active AND t.specialization = c.specialization AND c.grade = ANY (t.grades) AND NOT EXISTS (SELECT 1 FROM task_assignments a WHERE a.task_id = t.id AND a.candidate_id = c.user_id) ORDER BY (SELECT count(*) FROM candidate_skills s WHERE s.candidate_id = c.user_id AND s.skill = ANY (t.skills)) DESC, t.created_at LIMIT 1",
        None,
    )
}
impl PickTaskForStmt {
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
pub struct InsertAssignmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_assignment() -> InsertAssignmentStmt {
    InsertAssignmentStmt(
        "INSERT INTO task_assignments (id, task_id, candidate_id, due_at) VALUES ($1, $2, $3, $4)",
        None,
    )
}
impl InsertAssignmentStmt {
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
        task_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        due_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, task_id, candidate_id, due_at])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertAssignmentParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertAssignmentStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertAssignmentParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.task_id,
            &params.candidate_id,
            &params.due_at,
        ))
    }
}
pub struct CandidateAssignmentsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_assignments() -> CandidateAssignmentsStmt {
    CandidateAssignmentsStmt(
        "SELECT a.id, t.title, t.body, t.kind, t.skills, co.name AS company_name, a.status, a.answer, a.rating, a.review_comment, a.assigned_at, a.due_at, a.submitted_at, a.reviewed_at FROM task_assignments a JOIN tasks t ON t.id = a.task_id LEFT JOIN companies co ON co.id = t.company_id WHERE a.candidate_id = $1 ORDER BY a.assigned_at DESC",
        None,
    )
}
impl CandidateAssignmentsStmt {
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
    ) -> CandidateAssignmentsQuery<'c, 'a, 's, C, CandidateAssignments, 1> {
        CandidateAssignmentsQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateAssignmentsBorrowed, tokio_postgres::Error> {
                Ok(CandidateAssignmentsBorrowed {
                    id: row.try_get(0)?,
                    title: row.try_get(1)?,
                    body: row.try_get(2)?,
                    kind: row.try_get(3)?,
                    skills: row.try_get(4)?,
                    company_name: row.try_get(5)?,
                    status: row.try_get(6)?,
                    answer: row.try_get(7)?,
                    rating: row.try_get(8)?,
                    review_comment: row.try_get(9)?,
                    assigned_at: row.try_get(10)?,
                    due_at: row.try_get(11)?,
                    submitted_at: row.try_get(12)?,
                    reviewed_at: row.try_get(13)?,
                })
            },
            mapper: |it| CandidateAssignments::from(it),
        }
    }
}
pub struct SubmitAssignmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn submit_assignment() -> SubmitAssignmentStmt {
    SubmitAssignmentStmt(
        "UPDATE task_assignments SET status = 'submitted', answer = $1, submitted_at = now() WHERE id = $2 AND candidate_id = $3 AND status = 'assigned' AND due_at >= now()",
        None,
    )
}
impl SubmitAssignmentStmt {
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
        answer: &'a T1,
        id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[answer, id, candidate_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SubmitAssignmentParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SubmitAssignmentStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SubmitAssignmentParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.answer, &params.id, &params.candidate_id))
    }
}
pub struct SkipAssignmentStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn skip_assignment() -> SkipAssignmentStmt {
    SkipAssignmentStmt(
        "UPDATE task_assignments SET status = 'skipped' WHERE id = $1 AND candidate_id = $2 AND status = 'assigned'",
        None,
    )
}
impl SkipAssignmentStmt {
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
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id, candidate_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SkipAssignmentParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SkipAssignmentStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SkipAssignmentParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.id, &params.candidate_id))
    }
}
