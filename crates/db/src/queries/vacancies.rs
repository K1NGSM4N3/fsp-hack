// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertVacancyParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::ArraySql<Item = T5>,
    T7: crate::StringSql,
    T8: crate::StringSql,
> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub title: T1,
    pub description: T2,
    pub specialization: T3,
    pub grade: T4,
    pub skills: T6,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: T7,
    pub city: Option<T8>,
    pub status: crate::types::VacancyStatus,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}
#[derive(Debug)]
pub struct UpdateVacancyParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::ArraySql<Item = T5>,
    T7: crate::StringSql,
    T8: crate::StringSql,
> {
    pub title: T1,
    pub description: T2,
    pub specialization: T3,
    pub grade: T4,
    pub skills: T6,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: T7,
    pub city: Option<T8>,
    pub status: crate::types::VacancyStatus,
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct CompanyVacancyParams {
    pub company_id: uuid::Uuid,
    pub id: uuid::Uuid,
}
#[derive(Debug)]
pub struct PublishedVacanciesParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
> {
    pub specialization: Option<T1>,
    pub grade: Option<T2>,
    pub work_format: Option<T3>,
    pub salary_from: Option<i64>,
    pub q: Option<T4>,
    pub limit: i64,
    pub offset: i64,
}
#[derive(Debug)]
pub struct InsertApplicationParams<T1: crate::StringSql> {
    pub id: uuid::Uuid,
    pub vacancy_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub cover_letter: T1,
}
#[derive(Debug)]
pub struct ReopenApplicationParams<T1: crate::StringSql> {
    pub cover_letter: T1,
    pub vacancy_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct WithdrawApplicationParams {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct VacancyApplicationsParams {
    pub vacancy_id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct SetApplicationStatusParams {
    pub status: crate::types::ApplicationStatus,
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyVacancies {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grade: String,
    pub skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: String,
    pub city: Option<String>,
    pub status: crate::types::VacancyStatus,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub applications: i64,
}
pub struct CompanyVacanciesBorrowed<'a> {
    pub id: uuid::Uuid,
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: &'a str,
    pub city: Option<&'a str>,
    pub status: crate::types::VacancyStatus,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub applications: i64,
}
impl<'a> From<CompanyVacanciesBorrowed<'a>> for CompanyVacancies {
    fn from(
        CompanyVacanciesBorrowed {
            id,
            title,
            description,
            specialization,
            grade,
            skills,
            salary_min,
            salary_max,
            work_format,
            city,
            status,
            published_at,
            created_at,
            updated_at,
            applications,
        }: CompanyVacanciesBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            specialization: specialization.into(),
            grade: grade.into(),
            skills: skills.map(|v| v.into()).collect(),
            salary_min,
            salary_max,
            work_format: work_format.into(),
            city: city.map(|v| v.into()),
            status,
            published_at,
            created_at,
            updated_at,
            applications,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyVacancy {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grade: String,
    pub skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: String,
    pub city: Option<String>,
    pub status: crate::types::VacancyStatus,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub applications: i64,
}
pub struct CompanyVacancyBorrowed<'a> {
    pub id: uuid::Uuid,
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: &'a str,
    pub city: Option<&'a str>,
    pub status: crate::types::VacancyStatus,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub applications: i64,
}
impl<'a> From<CompanyVacancyBorrowed<'a>> for CompanyVacancy {
    fn from(
        CompanyVacancyBorrowed {
            id,
            title,
            description,
            specialization,
            grade,
            skills,
            salary_min,
            salary_max,
            work_format,
            city,
            status,
            published_at,
            created_at,
            updated_at,
            applications,
        }: CompanyVacancyBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            specialization: specialization.into(),
            grade: grade.into(),
            skills: skills.map(|v| v.into()).collect(),
            salary_min,
            salary_max,
            work_format: work_format.into(),
            city: city.map(|v| v.into()),
            status,
            published_at,
            created_at,
            updated_at,
            applications,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PublishedVacancies {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: String,
    pub company_city: Option<String>,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grade: String,
    pub skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: String,
    pub city: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total: i64,
}
pub struct PublishedVacanciesBorrowed<'a> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: &'a str,
    pub company_city: Option<&'a str>,
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: &'a str,
    pub city: Option<&'a str>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total: i64,
}
impl<'a> From<PublishedVacanciesBorrowed<'a>> for PublishedVacancies {
    fn from(
        PublishedVacanciesBorrowed {
            id,
            company_id,
            company_name,
            company_city,
            title,
            description,
            specialization,
            grade,
            skills,
            salary_min,
            salary_max,
            work_format,
            city,
            published_at,
            total,
        }: PublishedVacanciesBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            company_id,
            company_name: company_name.into(),
            company_city: company_city.map(|v| v.into()),
            title: title.into(),
            description: description.into(),
            specialization: specialization.into(),
            grade: grade.into(),
            skills: skills.map(|v| v.into()).collect(),
            salary_min,
            salary_max,
            work_format: work_format.into(),
            city: city.map(|v| v.into()),
            published_at,
            total,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PublishedVacancy {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: String,
    pub company_city: Option<String>,
    pub company_description: String,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grade: String,
    pub skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: String,
    pub city: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}
pub struct PublishedVacancyBorrowed<'a> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub company_name: &'a str,
    pub company_city: Option<&'a str>,
    pub company_description: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: &'a str,
    pub city: Option<&'a str>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl<'a> From<PublishedVacancyBorrowed<'a>> for PublishedVacancy {
    fn from(
        PublishedVacancyBorrowed {
            id,
            company_id,
            company_name,
            company_city,
            company_description,
            title,
            description,
            specialization,
            grade,
            skills,
            salary_min,
            salary_max,
            work_format,
            city,
            published_at,
        }: PublishedVacancyBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            company_id,
            company_name: company_name.into(),
            company_city: company_city.map(|v| v.into()),
            company_description: company_description.into(),
            title: title.into(),
            description: description.into(),
            specialization: specialization.into(),
            grade: grade.into(),
            skills: skills.map(|v| v.into()).collect(),
            salary_min,
            salary_max,
            work_format: work_format.into(),
            city: city.map(|v| v.into()),
            published_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateApplications {
    pub id: uuid::Uuid,
    pub vacancy_id: uuid::Uuid,
    pub vacancy_title: String,
    pub company_name: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub cover_letter: String,
    pub status: crate::types::ApplicationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
pub struct CandidateApplicationsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub vacancy_id: uuid::Uuid,
    pub vacancy_title: &'a str,
    pub company_name: &'a str,
    pub salary_min: i64,
    pub salary_max: i64,
    pub cover_letter: &'a str,
    pub status: crate::types::ApplicationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<CandidateApplicationsBorrowed<'a>> for CandidateApplications {
    fn from(
        CandidateApplicationsBorrowed {
            id,
            vacancy_id,
            vacancy_title,
            company_name,
            salary_min,
            salary_max,
            cover_letter,
            status,
            created_at,
            updated_at,
        }: CandidateApplicationsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            vacancy_id,
            vacancy_title: vacancy_title.into(),
            company_name: company_name.into(),
            salary_min,
            salary_max,
            cover_letter: cover_letter.into(),
            status,
            created_at,
            updated_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct VacancyApplications {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub theta: Option<f64>,
    pub cover_letter: String,
    pub status: crate::types::ApplicationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
pub struct VacancyApplicationsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub public_code: &'a str,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub theta: Option<f64>,
    pub cover_letter: &'a str,
    pub status: crate::types::ApplicationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<VacancyApplicationsBorrowed<'a>> for VacancyApplications {
    fn from(
        VacancyApplicationsBorrowed {
            id,
            candidate_id,
            public_code,
            specialization,
            grade,
            theta,
            cover_letter,
            status,
            created_at,
            updated_at,
        }: VacancyApplicationsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            candidate_id,
            public_code: public_code.into(),
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
            theta,
            cover_letter: cover_letter.into(),
            status,
            created_at,
            updated_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CompanyVacanciesQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyVacanciesBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyVacanciesBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyVacanciesQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyVacanciesBorrowed) -> R,
    ) -> CompanyVacanciesQuery<'c, 'a, 's, C, R, N> {
        CompanyVacanciesQuery {
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
pub struct CompanyVacancyQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyVacancyBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyVacancyBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyVacancyQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyVacancyBorrowed) -> R,
    ) -> CompanyVacancyQuery<'c, 'a, 's, C, R, N> {
        CompanyVacancyQuery {
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
pub struct PublishedVacanciesQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<PublishedVacanciesBorrowed, tokio_postgres::Error>,
    mapper: fn(PublishedVacanciesBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> PublishedVacanciesQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(PublishedVacanciesBorrowed) -> R,
    ) -> PublishedVacanciesQuery<'c, 'a, 's, C, R, N> {
        PublishedVacanciesQuery {
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
pub struct PublishedVacancyQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<PublishedVacancyBorrowed, tokio_postgres::Error>,
    mapper: fn(PublishedVacancyBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> PublishedVacancyQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(PublishedVacancyBorrowed) -> R,
    ) -> PublishedVacancyQuery<'c, 'a, 's, C, R, N> {
        PublishedVacancyQuery {
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
pub struct CandidateApplicationsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CandidateApplicationsBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateApplicationsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateApplicationsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateApplicationsBorrowed) -> R,
    ) -> CandidateApplicationsQuery<'c, 'a, 's, C, R, N> {
        CandidateApplicationsQuery {
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
pub struct VacancyApplicationsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<VacancyApplicationsBorrowed, tokio_postgres::Error>,
    mapper: fn(VacancyApplicationsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> VacancyApplicationsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(VacancyApplicationsBorrowed) -> R,
    ) -> VacancyApplicationsQuery<'c, 'a, 's, C, R, N> {
        VacancyApplicationsQuery {
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
pub struct InsertVacancyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_vacancy() -> InsertVacancyStmt {
    InsertVacancyStmt(
        "INSERT INTO vacancies (id, company_id, title, description, specialization, grade, skills, salary_min, salary_max, work_format, city, status, published_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        None,
    )
}
impl InsertVacancyStmt {
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
        T6: crate::ArraySql<Item = T5>,
        T7: crate::StringSql,
        T8: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
        title: &'a T1,
        description: &'a T2,
        specialization: &'a T3,
        grade: &'a T4,
        skills: &'a T6,
        salary_min: &'a i64,
        salary_max: &'a i64,
        work_format: &'a T7,
        city: &'a Option<T8>,
        status: &'a crate::types::VacancyStatus,
        published_at: &'a Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    company_id,
                    title,
                    description,
                    specialization,
                    grade,
                    skills,
                    salary_min,
                    salary_max,
                    work_format,
                    city,
                    status,
                    published_at,
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
    T6: crate::ArraySql<Item = T5>,
    T7: crate::StringSql,
    T8: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertVacancyParams<T1, T2, T3, T4, T5, T6, T7, T8>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertVacancyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertVacancyParams<T1, T2, T3, T4, T5, T6, T7, T8>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.company_id,
            &params.title,
            &params.description,
            &params.specialization,
            &params.grade,
            &params.skills,
            &params.salary_min,
            &params.salary_max,
            &params.work_format,
            &params.city,
            &params.status,
            &params.published_at,
        ))
    }
}
pub struct UpdateVacancyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_vacancy() -> UpdateVacancyStmt {
    UpdateVacancyStmt(
        "UPDATE vacancies SET title = $1, description = $2, specialization = $3, grade = $4, skills = $5, salary_min = $6, salary_max = $7, work_format = $8, city = $9, status = $10, published_at = CASE WHEN $10 = 'published'::vacancy_status AND published_at IS NULL THEN now() ELSE published_at END, updated_at = now() WHERE id = $11 AND company_id = $12",
        None,
    )
}
impl UpdateVacancyStmt {
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
        T6: crate::ArraySql<Item = T5>,
        T7: crate::StringSql,
        T8: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        title: &'a T1,
        description: &'a T2,
        specialization: &'a T3,
        grade: &'a T4,
        skills: &'a T6,
        salary_min: &'a i64,
        salary_max: &'a i64,
        work_format: &'a T7,
        city: &'a Option<T8>,
        status: &'a crate::types::VacancyStatus,
        id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    title,
                    description,
                    specialization,
                    grade,
                    skills,
                    salary_min,
                    salary_max,
                    work_format,
                    city,
                    status,
                    id,
                    company_id,
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
    T6: crate::ArraySql<Item = T5>,
    T7: crate::StringSql,
    T8: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdateVacancyParams<T1, T2, T3, T4, T5, T6, T7, T8>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdateVacancyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdateVacancyParams<T1, T2, T3, T4, T5, T6, T7, T8>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.title,
            &params.description,
            &params.specialization,
            &params.grade,
            &params.skills,
            &params.salary_min,
            &params.salary_max,
            &params.work_format,
            &params.city,
            &params.status,
            &params.id,
            &params.company_id,
        ))
    }
}
pub struct CompanyVacanciesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_vacancies() -> CompanyVacanciesStmt {
    CompanyVacanciesStmt(
        "SELECT v.id, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max, v.work_format, v.city, v.status, v.published_at, v.created_at, v.updated_at, (SELECT count(*) FROM applications a WHERE a.vacancy_id = v.id AND a.status <> 'withdrawn') AS applications FROM vacancies v WHERE v.company_id = $1 ORDER BY v.created_at DESC",
        None,
    )
}
impl CompanyVacanciesStmt {
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
    ) -> CompanyVacanciesQuery<'c, 'a, 's, C, CompanyVacancies, 1> {
        CompanyVacanciesQuery {
            client,
            params: [company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CompanyVacanciesBorrowed, tokio_postgres::Error> {
                Ok(CompanyVacanciesBorrowed {
                    id: row.try_get(0)?,
                    title: row.try_get(1)?,
                    description: row.try_get(2)?,
                    specialization: row.try_get(3)?,
                    grade: row.try_get(4)?,
                    skills: row.try_get(5)?,
                    salary_min: row.try_get(6)?,
                    salary_max: row.try_get(7)?,
                    work_format: row.try_get(8)?,
                    city: row.try_get(9)?,
                    status: row.try_get(10)?,
                    published_at: row.try_get(11)?,
                    created_at: row.try_get(12)?,
                    updated_at: row.try_get(13)?,
                    applications: row.try_get(14)?,
                })
            },
            mapper: |it| CompanyVacancies::from(it),
        }
    }
}
pub struct CompanyVacancyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_vacancy() -> CompanyVacancyStmt {
    CompanyVacancyStmt(
        "SELECT v.id, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max, v.work_format, v.city, v.status, v.published_at, v.created_at, v.updated_at, (SELECT count(*) FROM applications a WHERE a.vacancy_id = v.id AND a.status <> 'withdrawn') AS applications FROM vacancies v WHERE v.company_id = $1 AND v.id = $2",
        None,
    )
}
impl CompanyVacancyStmt {
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
    ) -> CompanyVacancyQuery<'c, 'a, 's, C, CompanyVacancy, 2> {
        CompanyVacancyQuery {
            client,
            params: [company_id, id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CompanyVacancyBorrowed, tokio_postgres::Error> {
                Ok(CompanyVacancyBorrowed {
                    id: row.try_get(0)?,
                    title: row.try_get(1)?,
                    description: row.try_get(2)?,
                    specialization: row.try_get(3)?,
                    grade: row.try_get(4)?,
                    skills: row.try_get(5)?,
                    salary_min: row.try_get(6)?,
                    salary_max: row.try_get(7)?,
                    work_format: row.try_get(8)?,
                    city: row.try_get(9)?,
                    status: row.try_get(10)?,
                    published_at: row.try_get(11)?,
                    created_at: row.try_get(12)?,
                    updated_at: row.try_get(13)?,
                    applications: row.try_get(14)?,
                })
            },
            mapper: |it| CompanyVacancy::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CompanyVacancyParams,
        CompanyVacancyQuery<'c, 'a, 's, C, CompanyVacancy, 2>,
        C,
    > for CompanyVacancyStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CompanyVacancyParams,
    ) -> CompanyVacancyQuery<'c, 'a, 's, C, CompanyVacancy, 2> {
        self.bind(client, &params.company_id, &params.id)
    }
}
pub struct PublishedVacanciesStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn published_vacancies() -> PublishedVacanciesStmt {
    PublishedVacanciesStmt(
        "SELECT v.id, v.company_id, co.name AS company_name, co.city AS company_city, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max, v.work_format, v.city, v.published_at, count(*) OVER () AS total FROM vacancies v JOIN companies co ON co.id = v.company_id WHERE v.status = 'published' AND ($1::text IS NULL OR v.specialization = $1) AND ($2::text IS NULL OR v.grade = $2) AND ($3::text IS NULL OR v.work_format = $3) AND ($4::bigint IS NULL OR v.salary_max >= $4) AND ($5::text IS NULL OR v.search_tsv @@ websearch_to_tsquery('russian', $5)) ORDER BY v.published_at DESC LIMIT $6 OFFSET $7",
        None,
    )
}
impl PublishedVacanciesStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<
        'c,
        'a,
        's,
        C: GenericClient,
        T1: crate::StringSql,
        T2: crate::StringSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        specialization: &'a Option<T1>,
        grade: &'a Option<T2>,
        work_format: &'a Option<T3>,
        salary_from: &'a Option<i64>,
        q: &'a Option<T4>,
        limit: &'a i64,
        offset: &'a i64,
    ) -> PublishedVacanciesQuery<'c, 'a, 's, C, PublishedVacancies, 7> {
        PublishedVacanciesQuery {
            client,
            params: [specialization, grade, work_format, salary_from, q, limit, offset],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<PublishedVacanciesBorrowed, tokio_postgres::Error> {
                Ok(PublishedVacanciesBorrowed {
                    id: row.try_get(0)?,
                    company_id: row.try_get(1)?,
                    company_name: row.try_get(2)?,
                    company_city: row.try_get(3)?,
                    title: row.try_get(4)?,
                    description: row.try_get(5)?,
                    specialization: row.try_get(6)?,
                    grade: row.try_get(7)?,
                    skills: row.try_get(8)?,
                    salary_min: row.try_get(9)?,
                    salary_max: row.try_get(10)?,
                    work_format: row.try_get(11)?,
                    city: row.try_get(12)?,
                    published_at: row.try_get(13)?,
                    total: row.try_get(14)?,
                })
            },
            mapper: |it| PublishedVacancies::from(it),
        }
    }
}
impl<
    'c,
    'a,
    's,
    C: GenericClient,
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        PublishedVacanciesParams<T1, T2, T3, T4>,
        PublishedVacanciesQuery<'c, 'a, 's, C, PublishedVacancies, 7>,
        C,
    > for PublishedVacanciesStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a PublishedVacanciesParams<T1, T2, T3, T4>,
    ) -> PublishedVacanciesQuery<'c, 'a, 's, C, PublishedVacancies, 7> {
        self.bind(
            client,
            &params.specialization,
            &params.grade,
            &params.work_format,
            &params.salary_from,
            &params.q,
            &params.limit,
            &params.offset,
        )
    }
}
pub struct PublishedVacancyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn published_vacancy() -> PublishedVacancyStmt {
    PublishedVacancyStmt(
        "SELECT v.id, v.company_id, co.name AS company_name, co.city AS company_city, co.description AS company_description, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max, v.work_format, v.city, v.published_at FROM vacancies v JOIN companies co ON co.id = v.company_id WHERE v.id = $1 AND v.status = 'published'",
        None,
    )
}
impl PublishedVacancyStmt {
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
    ) -> PublishedVacancyQuery<'c, 'a, 's, C, PublishedVacancy, 1> {
        PublishedVacancyQuery {
            client,
            params: [id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<PublishedVacancyBorrowed, tokio_postgres::Error> {
                Ok(PublishedVacancyBorrowed {
                    id: row.try_get(0)?,
                    company_id: row.try_get(1)?,
                    company_name: row.try_get(2)?,
                    company_city: row.try_get(3)?,
                    company_description: row.try_get(4)?,
                    title: row.try_get(5)?,
                    description: row.try_get(6)?,
                    specialization: row.try_get(7)?,
                    grade: row.try_get(8)?,
                    skills: row.try_get(9)?,
                    salary_min: row.try_get(10)?,
                    salary_max: row.try_get(11)?,
                    work_format: row.try_get(12)?,
                    city: row.try_get(13)?,
                    published_at: row.try_get(14)?,
                })
            },
            mapper: |it| PublishedVacancy::from(it),
        }
    }
}
pub struct InsertApplicationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_application() -> InsertApplicationStmt {
    InsertApplicationStmt(
        "INSERT INTO applications (id, vacancy_id, candidate_id, cover_letter) VALUES ($1, $2, $3, $4)",
        None,
    )
}
impl InsertApplicationStmt {
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
        id: &'a uuid::Uuid,
        vacancy_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        cover_letter: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[id, vacancy_id, candidate_id, cover_letter])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertApplicationParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertApplicationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertApplicationParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.vacancy_id,
            &params.candidate_id,
            &params.cover_letter,
        ))
    }
}
pub struct ReopenApplicationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn reopen_application() -> ReopenApplicationStmt {
    ReopenApplicationStmt(
        "UPDATE applications SET status = 'sent', cover_letter = $1, updated_at = now() WHERE vacancy_id = $2 AND candidate_id = $3 AND status = 'withdrawn' RETURNING id",
        None,
    )
}
impl ReopenApplicationStmt {
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
        cover_letter: &'a T1,
        vacancy_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 3> {
        UuidUuidQuery {
            client,
            params: [cover_letter, vacancy_id, candidate_id],
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
        ReopenApplicationParams<T1>,
        UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 3>,
        C,
    > for ReopenApplicationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ReopenApplicationParams<T1>,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 3> {
        self.bind(
            client,
            &params.cover_letter,
            &params.vacancy_id,
            &params.candidate_id,
        )
    }
}
pub struct CandidateApplicationsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_applications() -> CandidateApplicationsStmt {
    CandidateApplicationsStmt(
        "SELECT a.id, a.vacancy_id, v.title AS vacancy_title, co.name AS company_name, v.salary_min, v.salary_max, a.cover_letter, a.status, a.created_at, a.updated_at FROM applications a JOIN vacancies v ON v.id = a.vacancy_id JOIN companies co ON co.id = v.company_id WHERE a.candidate_id = $1 ORDER BY a.created_at DESC",
        None,
    )
}
impl CandidateApplicationsStmt {
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
    ) -> CandidateApplicationsQuery<'c, 'a, 's, C, CandidateApplications, 1> {
        CandidateApplicationsQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateApplicationsBorrowed, tokio_postgres::Error> {
                Ok(CandidateApplicationsBorrowed {
                    id: row.try_get(0)?,
                    vacancy_id: row.try_get(1)?,
                    vacancy_title: row.try_get(2)?,
                    company_name: row.try_get(3)?,
                    salary_min: row.try_get(4)?,
                    salary_max: row.try_get(5)?,
                    cover_letter: row.try_get(6)?,
                    status: row.try_get(7)?,
                    created_at: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                })
            },
            mapper: |it| CandidateApplications::from(it),
        }
    }
}
pub struct WithdrawApplicationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn withdraw_application() -> WithdrawApplicationStmt {
    WithdrawApplicationStmt(
        "UPDATE applications SET status = 'withdrawn', updated_at = now() WHERE id = $1 AND candidate_id = $2 AND status IN ('sent', 'viewed')",
        None,
    )
}
impl WithdrawApplicationStmt {
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
        WithdrawApplicationParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for WithdrawApplicationStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a WithdrawApplicationParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.id, &params.candidate_id))
    }
}
pub struct VacancyApplicationsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn vacancy_applications() -> VacancyApplicationsStmt {
    VacancyApplicationsStmt(
        "SELECT a.id, a.candidate_id, c.public_code, c.specialization, c.grade, c.theta, a.cover_letter, a.status, a.created_at, a.updated_at FROM applications a JOIN vacancies v ON v.id = a.vacancy_id JOIN candidates c ON c.user_id = a.candidate_id WHERE v.id = $1 AND v.company_id = $2 AND a.status <> 'withdrawn' ORDER BY a.created_at DESC",
        None,
    )
}
impl VacancyApplicationsStmt {
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
        vacancy_id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
    ) -> VacancyApplicationsQuery<'c, 'a, 's, C, VacancyApplications, 2> {
        VacancyApplicationsQuery {
            client,
            params: [vacancy_id, company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<VacancyApplicationsBorrowed, tokio_postgres::Error> {
                Ok(VacancyApplicationsBorrowed {
                    id: row.try_get(0)?,
                    candidate_id: row.try_get(1)?,
                    public_code: row.try_get(2)?,
                    specialization: row.try_get(3)?,
                    grade: row.try_get(4)?,
                    theta: row.try_get(5)?,
                    cover_letter: row.try_get(6)?,
                    status: row.try_get(7)?,
                    created_at: row.try_get(8)?,
                    updated_at: row.try_get(9)?,
                })
            },
            mapper: |it| VacancyApplications::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        VacancyApplicationsParams,
        VacancyApplicationsQuery<'c, 'a, 's, C, VacancyApplications, 2>,
        C,
    > for VacancyApplicationsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a VacancyApplicationsParams,
    ) -> VacancyApplicationsQuery<'c, 'a, 's, C, VacancyApplications, 2> {
        self.bind(client, &params.vacancy_id, &params.company_id)
    }
}
pub struct SetApplicationStatusStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn set_application_status() -> SetApplicationStatusStmt {
    SetApplicationStatusStmt(
        "UPDATE applications a SET status = $1, updated_at = now() FROM vacancies v WHERE a.id = $2 AND v.id = a.vacancy_id AND v.company_id = $3 AND a.status NOT IN ('withdrawn')",
        None,
    )
}
impl SetApplicationStatusStmt {
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
        status: &'a crate::types::ApplicationStatus,
        id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[status, id, company_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SetApplicationStatusParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SetApplicationStatusStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SetApplicationStatusParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.status, &params.id, &params.company_id))
    }
}
