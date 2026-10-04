// This file was generated with `clorinde`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct CompanyInvitationsSinceParams {
    pub company_id: uuid::Uuid,
    pub since: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug)]
pub struct InvitationByIdempotencyKeyParams<T1: crate::StringSql> {
    pub company_id: uuid::Uuid,
    pub idempotency_key: T1,
}
#[derive(Debug)]
pub struct InsertInvitationParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub need_id: Option<uuid::Uuid>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: T1,
    pub message: T2,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<T3>,
    pub contact: T4,
    pub idempotency_key: Option<T5>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Clone, Copy, Debug)]
pub struct InsertInvitationEventParams {
    pub id: uuid::Uuid,
    pub invitation_id: uuid::Uuid,
    pub status: crate::types::InvitationStatus,
    pub actor_user_id: Option<uuid::Uuid>,
}
#[derive(Clone, Copy, Debug)]
pub struct EmployerInvitationParams {
    pub company_id: uuid::Uuid,
    pub id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct CandidateInvitationParams {
    pub candidate_id: uuid::Uuid,
    pub id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct MarkInvitationViewedParams {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug)]
pub struct RespondInvitationParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<T1>,
    pub decline_comment: Option<T2>,
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct WithdrawInvitationParams {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct InvitationEvents {
    pub status: crate::types::InvitationStatus,
    pub at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EmployerInvitations {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub need_id: Option<uuid::Uuid>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: String,
    pub message: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<String>,
    pub contact: String,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
pub struct EmployerInvitationsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: &'a str,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub need_id: Option<uuid::Uuid>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: &'a str,
    pub message: &'a str,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<&'a str>,
    pub contact: &'a str,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<&'a str>,
    pub decline_comment: Option<&'a str>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<EmployerInvitationsBorrowed<'a>> for EmployerInvitations {
    fn from(
        EmployerInvitationsBorrowed {
            id,
            candidate_id,
            public_code,
            specialization,
            grade,
            need_id,
            vacancy_id,
            title,
            message,
            salary_min,
            salary_max,
            work_format,
            contact,
            status,
            decline_reason,
            decline_comment,
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }: EmployerInvitationsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            candidate_id,
            public_code: public_code.into(),
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
            need_id,
            vacancy_id,
            title: title.into(),
            message: message.into(),
            salary_min,
            salary_max,
            work_format: work_format.map(|v| v.into()),
            contact: contact.into(),
            status,
            decline_reason: decline_reason.map(|v| v.into()),
            decline_comment: decline_comment.map(|v| v.into()),
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EmployerInvitation {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub need_id: Option<uuid::Uuid>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: String,
    pub message: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<String>,
    pub contact: String,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
pub struct EmployerInvitationBorrowed<'a> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: &'a str,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub need_id: Option<uuid::Uuid>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: &'a str,
    pub message: &'a str,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<&'a str>,
    pub contact: &'a str,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<&'a str>,
    pub decline_comment: Option<&'a str>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<EmployerInvitationBorrowed<'a>> for EmployerInvitation {
    fn from(
        EmployerInvitationBorrowed {
            id,
            candidate_id,
            public_code,
            specialization,
            grade,
            need_id,
            vacancy_id,
            title,
            message,
            salary_min,
            salary_max,
            work_format,
            contact,
            status,
            decline_reason,
            decline_comment,
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }: EmployerInvitationBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            candidate_id,
            public_code: public_code.into(),
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
            need_id,
            vacancy_id,
            title: title.into(),
            message: message.into(),
            salary_min,
            salary_max,
            work_format: work_format.map(|v| v.into()),
            contact: contact.into(),
            status,
            decline_reason: decline_reason.map(|v| v.into()),
            decline_comment: decline_comment.map(|v| v.into()),
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateInvitations {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: String,
    pub company_description: String,
    pub company_industry: Option<String>,
    pub company_city: Option<String>,
    pub company_website: Option<String>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: String,
    pub message: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<String>,
    pub contact: String,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
pub struct CandidateInvitationsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: &'a str,
    pub company_description: &'a str,
    pub company_industry: Option<&'a str>,
    pub company_city: Option<&'a str>,
    pub company_website: Option<&'a str>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: &'a str,
    pub message: &'a str,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<&'a str>,
    pub contact: &'a str,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<&'a str>,
    pub decline_comment: Option<&'a str>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<CandidateInvitationsBorrowed<'a>> for CandidateInvitations {
    fn from(
        CandidateInvitationsBorrowed {
            id,
            company_id,
            company_name,
            company_description,
            company_industry,
            company_city,
            company_website,
            vacancy_id,
            title,
            message,
            salary_min,
            salary_max,
            work_format,
            contact,
            status,
            decline_reason,
            decline_comment,
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }: CandidateInvitationsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            company_id,
            company_name: company_name.into(),
            company_description: company_description.into(),
            company_industry: company_industry.map(|v| v.into()),
            company_city: company_city.map(|v| v.into()),
            company_website: company_website.map(|v| v.into()),
            vacancy_id,
            title: title.into(),
            message: message.into(),
            salary_min,
            salary_max,
            work_format: work_format.map(|v| v.into()),
            contact: contact.into(),
            status,
            decline_reason: decline_reason.map(|v| v.into()),
            decline_comment: decline_comment.map(|v| v.into()),
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateInvitation {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: String,
    pub company_description: String,
    pub company_industry: Option<String>,
    pub company_city: Option<String>,
    pub company_website: Option<String>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: String,
    pub message: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<String>,
    pub contact: String,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
pub struct CandidateInvitationBorrowed<'a> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: &'a str,
    pub company_description: &'a str,
    pub company_industry: Option<&'a str>,
    pub company_city: Option<&'a str>,
    pub company_website: Option<&'a str>,
    pub vacancy_id: Option<uuid::Uuid>,
    pub title: &'a str,
    pub message: &'a str,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<&'a str>,
    pub contact: &'a str,
    pub status: crate::types::InvitationStatus,
    pub decline_reason: Option<&'a str>,
    pub decline_comment: Option<&'a str>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub responded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<CandidateInvitationBorrowed<'a>> for CandidateInvitation {
    fn from(
        CandidateInvitationBorrowed {
            id,
            company_id,
            company_name,
            company_description,
            company_industry,
            company_city,
            company_website,
            vacancy_id,
            title,
            message,
            salary_min,
            salary_max,
            work_format,
            contact,
            status,
            decline_reason,
            decline_comment,
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }: CandidateInvitationBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            company_id,
            company_name: company_name.into(),
            company_description: company_description.into(),
            company_industry: company_industry.map(|v| v.into()),
            company_city: company_city.map(|v| v.into()),
            company_website: company_website.map(|v| v.into()),
            vacancy_id,
            title: title.into(),
            message: message.into(),
            salary_min,
            salary_max,
            work_format: work_format.map(|v| v.into()),
            contact: contact.into(),
            status,
            decline_reason: decline_reason.map(|v| v.into()),
            decline_comment: decline_comment.map(|v| v.into()),
            created_at,
            viewed_at,
            responded_at,
            expires_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CompanyInvitationStatusCounts {
    pub status: crate::types::InvitationStatus,
    pub count: i64,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyDeclineReasons {
    pub reason: Option<String>,
    pub count: i64,
}
pub struct CompanyDeclineReasonsBorrowed<'a> {
    pub reason: Option<&'a str>,
    pub count: i64,
}
impl<'a> From<CompanyDeclineReasonsBorrowed<'a>> for CompanyDeclineReasons {
    fn from(
        CompanyDeclineReasonsBorrowed { reason, count }: CompanyDeclineReasonsBorrowed<'a>,
    ) -> Self {
        Self {
            reason: reason.map(|v| v.into()),
            count,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CompanyResponseRate {
    pub answered: i64,
    pub total: i64,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
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
pub struct InvitationEventsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<InvitationEvents, tokio_postgres::Error>,
    mapper: fn(InvitationEvents) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> InvitationEventsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(InvitationEvents) -> R,
    ) -> InvitationEventsQuery<'c, 'a, 's, C, R, N> {
        InvitationEventsQuery {
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
pub struct EmployerInvitationsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<EmployerInvitationsBorrowed, tokio_postgres::Error>,
    mapper: fn(EmployerInvitationsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> EmployerInvitationsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(EmployerInvitationsBorrowed) -> R,
    ) -> EmployerInvitationsQuery<'c, 'a, 's, C, R, N> {
        EmployerInvitationsQuery {
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
pub struct EmployerInvitationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<EmployerInvitationBorrowed, tokio_postgres::Error>,
    mapper: fn(EmployerInvitationBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> EmployerInvitationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(EmployerInvitationBorrowed) -> R,
    ) -> EmployerInvitationQuery<'c, 'a, 's, C, R, N> {
        EmployerInvitationQuery {
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
pub struct CandidateInvitationsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CandidateInvitationsBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateInvitationsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateInvitationsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateInvitationsBorrowed) -> R,
    ) -> CandidateInvitationsQuery<'c, 'a, 's, C, R, N> {
        CandidateInvitationsQuery {
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
pub struct CandidateInvitationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CandidateInvitationBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateInvitationBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateInvitationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateInvitationBorrowed) -> R,
    ) -> CandidateInvitationQuery<'c, 'a, 's, C, R, N> {
        CandidateInvitationQuery {
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
pub struct CompanyInvitationStatusCountsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CompanyInvitationStatusCounts, tokio_postgres::Error>,
    mapper: fn(CompanyInvitationStatusCounts) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyInvitationStatusCountsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyInvitationStatusCounts) -> R,
    ) -> CompanyInvitationStatusCountsQuery<'c, 'a, 's, C, R, N> {
        CompanyInvitationStatusCountsQuery {
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
pub struct CompanyDeclineReasonsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CompanyDeclineReasonsBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyDeclineReasonsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyDeclineReasonsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyDeclineReasonsBorrowed) -> R,
    ) -> CompanyDeclineReasonsQuery<'c, 'a, 's, C, R, N> {
        CompanyDeclineReasonsQuery {
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
pub struct CompanyResponseRateQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyResponseRate, tokio_postgres::Error>,
    mapper: fn(CompanyResponseRate) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyResponseRateQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyResponseRate) -> R,
    ) -> CompanyResponseRateQuery<'c, 'a, 's, C, R, N> {
        CompanyResponseRateQuery {
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
pub struct ExpireStaleInvitationsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn expire_stale_invitations() -> ExpireStaleInvitationsStmt {
    ExpireStaleInvitationsStmt(
        "UPDATE invitations SET status = 'expired' WHERE status IN ('sent', 'viewed') AND expires_at < now()",
        None,
    )
}
impl ExpireStaleInvitationsStmt {
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
pub struct CompanyInvitationsSinceStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_invitations_since() -> CompanyInvitationsSinceStmt {
    CompanyInvitationsSinceStmt(
        "SELECT count(*) AS count FROM invitations WHERE company_id = $1 AND created_at >= $2",
        None,
    )
}
impl CompanyInvitationsSinceStmt {
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
        since: &'a chrono::DateTime<chrono::Utc>,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        I64Query {
            client,
            params: [company_id, since],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CompanyInvitationsSinceParams,
        I64Query<'c, 'a, 's, C, i64, 2>,
        C,
    > for CompanyInvitationsSinceStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CompanyInvitationsSinceParams,
    ) -> I64Query<'c, 'a, 's, C, i64, 2> {
        self.bind(client, &params.company_id, &params.since)
    }
}
pub struct InvitationByIdempotencyKeyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn invitation_by_idempotency_key() -> InvitationByIdempotencyKeyStmt {
    InvitationByIdempotencyKeyStmt(
        "SELECT id FROM invitations WHERE company_id = $1 AND idempotency_key = $2",
        None,
    )
}
impl InvitationByIdempotencyKeyStmt {
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
        company_id: &'a uuid::Uuid,
        idempotency_key: &'a T1,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2> {
        UuidUuidQuery {
            client,
            params: [company_id, idempotency_key],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        InvitationByIdempotencyKeyParams<T1>,
        UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2>,
        C,
    > for InvitationByIdempotencyKeyStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a InvitationByIdempotencyKeyParams<T1>,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2> {
        self.bind(client, &params.company_id, &params.idempotency_key)
    }
}
pub struct InsertInvitationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_invitation() -> InsertInvitationStmt {
    InsertInvitationStmt(
        "INSERT INTO invitations (id, company_id, candidate_id, need_id, vacancy_id, title, message, salary_min, salary_max, work_format, contact, idempotency_key, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        None,
    )
}
impl InsertInvitationStmt {
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
        T4: crate::StringSql,
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        need_id: &'a Option<uuid::Uuid>,
        vacancy_id: &'a Option<uuid::Uuid>,
        title: &'a T1,
        message: &'a T2,
        salary_min: &'a i64,
        salary_max: &'a i64,
        work_format: &'a Option<T3>,
        contact: &'a T4,
        idempotency_key: &'a Option<T5>,
        expires_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    company_id,
                    candidate_id,
                    need_id,
                    vacancy_id,
                    title,
                    message,
                    salary_min,
                    salary_max,
                    work_format,
                    contact,
                    idempotency_key,
                    expires_at,
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
    T4: crate::StringSql,
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertInvitationParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertInvitationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertInvitationParams<T1, T2, T3, T4, T5>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.company_id,
            &params.candidate_id,
            &params.need_id,
            &params.vacancy_id,
            &params.title,
            &params.message,
            &params.salary_min,
            &params.salary_max,
            &params.work_format,
            &params.contact,
            &params.idempotency_key,
            &params.expires_at,
        ))
    }
}
pub struct InsertInvitationEventStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_invitation_event() -> InsertInvitationEventStmt {
    InsertInvitationEventStmt(
        "INSERT INTO invitation_events (id, invitation_id, status, actor_user_id) VALUES ($1, $2, $3, $4)",
        None,
    )
}
impl InsertInvitationEventStmt {
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
        invitation_id: &'a uuid::Uuid,
        status: &'a crate::types::InvitationStatus,
        actor_user_id: &'a Option<uuid::Uuid>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, invitation_id, status, actor_user_id])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertInvitationEventParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertInvitationEventStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertInvitationEventParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.invitation_id,
            &params.status,
            &params.actor_user_id,
        ))
    }
}
pub struct InvitationEventsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn invitation_events() -> InvitationEventsStmt {
    InvitationEventsStmt(
        "SELECT status, at FROM invitation_events WHERE invitation_id = $1 ORDER BY at",
        None,
    )
}
impl InvitationEventsStmt {
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
        invitation_id: &'a uuid::Uuid,
    ) -> InvitationEventsQuery<'c, 'a, 's, C, InvitationEvents, 1> {
        InvitationEventsQuery {
            client,
            params: [invitation_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<InvitationEvents, tokio_postgres::Error> {
                    Ok(InvitationEvents {
                        status: row.try_get(0)?,
                        at: row.try_get(1)?,
                    })
                },
            mapper: |it| InvitationEvents::from(it),
        }
    }
}
pub struct EmployerInvitationsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn employer_invitations() -> EmployerInvitationsStmt {
    EmployerInvitationsStmt(
        "SELECT i.id, i.candidate_id, c.public_code, c.specialization, c.grade, i.need_id, i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status, i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at FROM invitations i JOIN candidates c ON c.user_id = i.candidate_id WHERE i.company_id = $1 ORDER BY i.created_at DESC",
        None,
    )
}
impl EmployerInvitationsStmt {
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
    ) -> EmployerInvitationsQuery<'c, 'a, 's, C, EmployerInvitations, 1> {
        EmployerInvitationsQuery {
            client,
            params: [company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<EmployerInvitationsBorrowed, tokio_postgres::Error> {
                Ok(EmployerInvitationsBorrowed {
                    id: row.try_get(0)?,
                    candidate_id: row.try_get(1)?,
                    public_code: row.try_get(2)?,
                    specialization: row.try_get(3)?,
                    grade: row.try_get(4)?,
                    need_id: row.try_get(5)?,
                    vacancy_id: row.try_get(6)?,
                    title: row.try_get(7)?,
                    message: row.try_get(8)?,
                    salary_min: row.try_get(9)?,
                    salary_max: row.try_get(10)?,
                    work_format: row.try_get(11)?,
                    contact: row.try_get(12)?,
                    status: row.try_get(13)?,
                    decline_reason: row.try_get(14)?,
                    decline_comment: row.try_get(15)?,
                    created_at: row.try_get(16)?,
                    viewed_at: row.try_get(17)?,
                    responded_at: row.try_get(18)?,
                    expires_at: row.try_get(19)?,
                })
            },
            mapper: |it| EmployerInvitations::from(it),
        }
    }
}
pub struct EmployerInvitationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn employer_invitation() -> EmployerInvitationStmt {
    EmployerInvitationStmt(
        "SELECT i.id, i.candidate_id, c.public_code, c.specialization, c.grade, i.need_id, i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status, i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at FROM invitations i JOIN candidates c ON c.user_id = i.candidate_id WHERE i.company_id = $1 AND i.id = $2",
        None,
    )
}
impl EmployerInvitationStmt {
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
        id: &'a uuid::Uuid,
    ) -> EmployerInvitationQuery<'c, 'a, 's, C, EmployerInvitation, 2> {
        EmployerInvitationQuery {
            client,
            params: [company_id, id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<EmployerInvitationBorrowed, tokio_postgres::Error> {
                Ok(EmployerInvitationBorrowed {
                    id: row.try_get(0)?,
                    candidate_id: row.try_get(1)?,
                    public_code: row.try_get(2)?,
                    specialization: row.try_get(3)?,
                    grade: row.try_get(4)?,
                    need_id: row.try_get(5)?,
                    vacancy_id: row.try_get(6)?,
                    title: row.try_get(7)?,
                    message: row.try_get(8)?,
                    salary_min: row.try_get(9)?,
                    salary_max: row.try_get(10)?,
                    work_format: row.try_get(11)?,
                    contact: row.try_get(12)?,
                    status: row.try_get(13)?,
                    decline_reason: row.try_get(14)?,
                    decline_comment: row.try_get(15)?,
                    created_at: row.try_get(16)?,
                    viewed_at: row.try_get(17)?,
                    responded_at: row.try_get(18)?,
                    expires_at: row.try_get(19)?,
                })
            },
            mapper: |it| EmployerInvitation::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        EmployerInvitationParams,
        EmployerInvitationQuery<'c, 'a, 's, C, EmployerInvitation, 2>,
        C,
    > for EmployerInvitationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a EmployerInvitationParams,
    ) -> EmployerInvitationQuery<'c, 'a, 's, C, EmployerInvitation, 2> {
        self.bind(client, &params.company_id, &params.id)
    }
}
pub struct CandidateInvitationsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_invitations() -> CandidateInvitationsStmt {
    CandidateInvitationsStmt(
        "SELECT i.id, i.company_id, co.name AS company_name, co.description AS company_description, co.industry AS company_industry, co.city AS company_city, co.website AS company_website, i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status, i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at FROM invitations i JOIN companies co ON co.id = i.company_id WHERE i.candidate_id = $1 ORDER BY i.created_at DESC",
        None,
    )
}
impl CandidateInvitationsStmt {
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
    ) -> CandidateInvitationsQuery<'c, 'a, 's, C, CandidateInvitations, 1> {
        CandidateInvitationsQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateInvitationsBorrowed, tokio_postgres::Error> {
                Ok(CandidateInvitationsBorrowed {
                    id: row.try_get(0)?,
                    company_id: row.try_get(1)?,
                    company_name: row.try_get(2)?,
                    company_description: row.try_get(3)?,
                    company_industry: row.try_get(4)?,
                    company_city: row.try_get(5)?,
                    company_website: row.try_get(6)?,
                    vacancy_id: row.try_get(7)?,
                    title: row.try_get(8)?,
                    message: row.try_get(9)?,
                    salary_min: row.try_get(10)?,
                    salary_max: row.try_get(11)?,
                    work_format: row.try_get(12)?,
                    contact: row.try_get(13)?,
                    status: row.try_get(14)?,
                    decline_reason: row.try_get(15)?,
                    decline_comment: row.try_get(16)?,
                    created_at: row.try_get(17)?,
                    viewed_at: row.try_get(18)?,
                    responded_at: row.try_get(19)?,
                    expires_at: row.try_get(20)?,
                })
            },
            mapper: |it| CandidateInvitations::from(it),
        }
    }
}
pub struct CandidateInvitationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_invitation() -> CandidateInvitationStmt {
    CandidateInvitationStmt(
        "SELECT i.id, i.company_id, co.name AS company_name, co.description AS company_description, co.industry AS company_industry, co.city AS company_city, co.website AS company_website, i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status, i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at FROM invitations i JOIN companies co ON co.id = i.company_id WHERE i.candidate_id = $1 AND i.id = $2",
        None,
    )
}
impl CandidateInvitationStmt {
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
        id: &'a uuid::Uuid,
    ) -> CandidateInvitationQuery<'c, 'a, 's, C, CandidateInvitation, 2> {
        CandidateInvitationQuery {
            client,
            params: [candidate_id, id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateInvitationBorrowed, tokio_postgres::Error> {
                Ok(CandidateInvitationBorrowed {
                    id: row.try_get(0)?,
                    company_id: row.try_get(1)?,
                    company_name: row.try_get(2)?,
                    company_description: row.try_get(3)?,
                    company_industry: row.try_get(4)?,
                    company_city: row.try_get(5)?,
                    company_website: row.try_get(6)?,
                    vacancy_id: row.try_get(7)?,
                    title: row.try_get(8)?,
                    message: row.try_get(9)?,
                    salary_min: row.try_get(10)?,
                    salary_max: row.try_get(11)?,
                    work_format: row.try_get(12)?,
                    contact: row.try_get(13)?,
                    status: row.try_get(14)?,
                    decline_reason: row.try_get(15)?,
                    decline_comment: row.try_get(16)?,
                    created_at: row.try_get(17)?,
                    viewed_at: row.try_get(18)?,
                    responded_at: row.try_get(19)?,
                    expires_at: row.try_get(20)?,
                })
            },
            mapper: |it| CandidateInvitation::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CandidateInvitationParams,
        CandidateInvitationQuery<'c, 'a, 's, C, CandidateInvitation, 2>,
        C,
    > for CandidateInvitationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CandidateInvitationParams,
    ) -> CandidateInvitationQuery<'c, 'a, 's, C, CandidateInvitation, 2> {
        self.bind(client, &params.candidate_id, &params.id)
    }
}
pub struct MarkInvitationViewedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn mark_invitation_viewed() -> MarkInvitationViewedStmt {
    MarkInvitationViewedStmt(
        "UPDATE invitations SET status = 'viewed', viewed_at = now() WHERE id = $1 AND candidate_id = $2 AND status = 'sent'",
        None,
    )
}
impl MarkInvitationViewedStmt {
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
        MarkInvitationViewedParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for MarkInvitationViewedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a MarkInvitationViewedParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.id, &params.candidate_id))
    }
}
pub struct RespondInvitationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn respond_invitation() -> RespondInvitationStmt {
    RespondInvitationStmt(
        "UPDATE invitations SET status = $1, responded_at = now(), decline_reason = $2, decline_comment = $3 WHERE id = $4 AND candidate_id = $5 AND status IN ('sent', 'viewed') AND expires_at >= now()",
        None,
    )
}
impl RespondInvitationStmt {
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
        status: &'a crate::types::InvitationStatus,
        decline_reason: &'a Option<T1>,
        decline_comment: &'a Option<T2>,
        id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[status, decline_reason, decline_comment, id, candidate_id],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        RespondInvitationParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for RespondInvitationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a RespondInvitationParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.status,
            &params.decline_reason,
            &params.decline_comment,
            &params.id,
            &params.candidate_id,
        ))
    }
}
pub struct WithdrawInvitationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn withdraw_invitation() -> WithdrawInvitationStmt {
    WithdrawInvitationStmt(
        "UPDATE invitations SET status = 'withdrawn', responded_at = now() WHERE id = $1 AND company_id = $2 AND status IN ('sent', 'viewed')",
        None,
    )
}
impl WithdrawInvitationStmt {
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
        company_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id, company_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        WithdrawInvitationParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for WithdrawInvitationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a WithdrawInvitationParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.id, &params.company_id))
    }
}
pub struct CompanyInvitationStatusCountsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_invitation_status_counts() -> CompanyInvitationStatusCountsStmt {
    CompanyInvitationStatusCountsStmt(
        "SELECT status, count(*) AS count FROM invitations WHERE company_id = $1 GROUP BY status",
        None,
    )
}
impl CompanyInvitationStatusCountsStmt {
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
    ) -> CompanyInvitationStatusCountsQuery<'c, 'a, 's, C, CompanyInvitationStatusCounts, 1> {
        CompanyInvitationStatusCountsQuery {
            client,
            params: [company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CompanyInvitationStatusCounts, tokio_postgres::Error> {
                Ok(CompanyInvitationStatusCounts {
                    status: row.try_get(0)?,
                    count: row.try_get(1)?,
                })
            },
            mapper: |it| CompanyInvitationStatusCounts::from(it),
        }
    }
}
pub struct CompanyDeclineReasonsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_decline_reasons() -> CompanyDeclineReasonsStmt {
    CompanyDeclineReasonsStmt(
        "SELECT decline_reason AS reason, count(*) AS count FROM invitations WHERE company_id = $1 AND status = 'declined' GROUP BY decline_reason ORDER BY count DESC",
        None,
    )
}
impl CompanyDeclineReasonsStmt {
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
    ) -> CompanyDeclineReasonsQuery<'c, 'a, 's, C, CompanyDeclineReasons, 1> {
        CompanyDeclineReasonsQuery {
            client,
            params: [company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CompanyDeclineReasonsBorrowed, tokio_postgres::Error> {
                Ok(CompanyDeclineReasonsBorrowed {
                    reason: row.try_get(0)?,
                    count: row.try_get(1)?,
                })
            },
            mapper: |it| CompanyDeclineReasons::from(it),
        }
    }
}
pub struct CompanyResponseRateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_response_rate() -> CompanyResponseRateStmt {
    CompanyResponseRateStmt(
        "SELECT count(*) FILTER (WHERE status IN ('accepted', 'declined')) AS answered, count(*) AS total FROM invitations WHERE company_id = $1 AND status <> 'withdrawn'",
        None,
    )
}
impl CompanyResponseRateStmt {
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
    ) -> CompanyResponseRateQuery<'c, 'a, 's, C, CompanyResponseRate, 1> {
        CompanyResponseRateQuery {
            client,
            params: [company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CompanyResponseRate, tokio_postgres::Error> {
                    Ok(CompanyResponseRate {
                        answered: row.try_get(0)?,
                        total: row.try_get(1)?,
                    })
                },
            mapper: |it| CompanyResponseRate::from(it),
        }
    }
}
