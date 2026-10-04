// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertNeedParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::ArraySql<Item = T6>,
    T8: crate::StringSql,
    T9: crate::ArraySql<Item = T8>,
    T10: crate::StringSql,
    T11: crate::ArraySql<Item = T10>,
    T12: crate::StringSql,
    T13: crate::StringSql,
    T14: crate::ArraySql<Item = T13>,
> {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub title: T1,
    pub description: T2,
    pub specialization: T3,
    pub grades: T5,
    pub allow_adjacent_grades: bool,
    pub must_skills: T7,
    pub nice_skills: T9,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: T11,
    pub city: Option<T12>,
    pub industries: T14,
}
#[derive(Debug)]
pub struct UpdateNeedParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::ArraySql<Item = T6>,
    T8: crate::StringSql,
    T9: crate::ArraySql<Item = T8>,
    T10: crate::StringSql,
    T11: crate::ArraySql<Item = T10>,
    T12: crate::StringSql,
    T13: crate::StringSql,
    T14: crate::ArraySql<Item = T13>,
    T15: crate::StringSql,
> {
    pub title: T1,
    pub description: T2,
    pub specialization: T3,
    pub grades: T5,
    pub allow_adjacent_grades: bool,
    pub must_skills: T7,
    pub nice_skills: T9,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: T11,
    pub city: Option<T12>,
    pub industries: T14,
    pub status: T15,
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct CompanyNeedParams {
    pub company_id: uuid::Uuid,
    pub id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct DeleteNeedParams {
    pub id: uuid::Uuid,
    pub company_id: uuid::Uuid,
}
#[derive(Debug)]
pub struct StartShortlistGenerationParams<T1: crate::JsonSql> {
    pub filters: T1,
    pub id: uuid::Uuid,
}
#[derive(Debug)]
pub struct MarkShortlistStaleParams<T1: crate::ArraySql<Item = uuid::Uuid>> {
    pub need_id: uuid::Uuid,
    pub keep: T1,
}
#[derive(Debug)]
pub struct UpsertShortlistItemParams<T1: crate::JsonSql> {
    pub need_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub score: f64,
    pub rank: i32,
    pub breakdown: T1,
}
#[derive(Clone, Copy, Debug)]
pub struct PatchShortlistItemParams {
    pub pinned: Option<bool>,
    pub dismissed: Option<bool>,
    pub company_id: uuid::Uuid,
    pub need_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyNeeds {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grades: Vec<String>,
    pub allow_adjacent_grades: bool,
    pub must_skills: Vec<String>,
    pub nice_skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: Vec<String>,
    pub city: Option<String>,
    pub industries: Vec<String>,
    pub filters: serde_json::Value,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub shortlist_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub shortlisted: i64,
    pub invited: i64,
}
pub struct CompanyNeedsBorrowed<'a> {
    pub id: uuid::Uuid,
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grades: crate::ArrayIterator<'a, &'a str>,
    pub allow_adjacent_grades: bool,
    pub must_skills: crate::ArrayIterator<'a, &'a str>,
    pub nice_skills: crate::ArrayIterator<'a, &'a str>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: crate::ArrayIterator<'a, &'a str>,
    pub city: Option<&'a str>,
    pub industries: crate::ArrayIterator<'a, &'a str>,
    pub filters: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub status: &'a str,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub shortlist_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub shortlisted: i64,
    pub invited: i64,
}
impl<'a> From<CompanyNeedsBorrowed<'a>> for CompanyNeeds {
    fn from(
        CompanyNeedsBorrowed {
            id,
            title,
            description,
            specialization,
            grades,
            allow_adjacent_grades,
            must_skills,
            nice_skills,
            salary_min,
            salary_max,
            work_formats,
            city,
            industries,
            filters,
            status,
            created_at,
            updated_at,
            shortlist_generated_at,
            shortlisted,
            invited,
        }: CompanyNeedsBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            specialization: specialization.into(),
            grades: grades.map(|v| v.into()).collect(),
            allow_adjacent_grades,
            must_skills: must_skills.map(|v| v.into()).collect(),
            nice_skills: nice_skills.map(|v| v.into()).collect(),
            salary_min,
            salary_max,
            work_formats: work_formats.map(|v| v.into()).collect(),
            city: city.map(|v| v.into()),
            industries: industries.map(|v| v.into()).collect(),
            filters: serde_json::from_str(filters.0.get()).unwrap(),
            status: status.into(),
            created_at,
            updated_at,
            shortlist_generated_at,
            shortlisted,
            invited,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyNeed {
    pub id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grades: Vec<String>,
    pub allow_adjacent_grades: bool,
    pub must_skills: Vec<String>,
    pub nice_skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: Vec<String>,
    pub city: Option<String>,
    pub industries: Vec<String>,
    pub filters: serde_json::Value,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub shortlist_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub shortlisted: i64,
    pub invited: i64,
}
pub struct CompanyNeedBorrowed<'a> {
    pub id: uuid::Uuid,
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grades: crate::ArrayIterator<'a, &'a str>,
    pub allow_adjacent_grades: bool,
    pub must_skills: crate::ArrayIterator<'a, &'a str>,
    pub nice_skills: crate::ArrayIterator<'a, &'a str>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: crate::ArrayIterator<'a, &'a str>,
    pub city: Option<&'a str>,
    pub industries: crate::ArrayIterator<'a, &'a str>,
    pub filters: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub status: &'a str,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub shortlist_generated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub shortlisted: i64,
    pub invited: i64,
}
impl<'a> From<CompanyNeedBorrowed<'a>> for CompanyNeed {
    fn from(
        CompanyNeedBorrowed {
            id,
            title,
            description,
            specialization,
            grades,
            allow_adjacent_grades,
            must_skills,
            nice_skills,
            salary_min,
            salary_max,
            work_formats,
            city,
            industries,
            filters,
            status,
            created_at,
            updated_at,
            shortlist_generated_at,
            shortlisted,
            invited,
        }: CompanyNeedBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            specialization: specialization.into(),
            grades: grades.map(|v| v.into()).collect(),
            allow_adjacent_grades,
            must_skills: must_skills.map(|v| v.into()).collect(),
            nice_skills: nice_skills.map(|v| v.into()).collect(),
            salary_min,
            salary_max,
            work_formats: work_formats.map(|v| v.into()).collect(),
            city: city.map(|v| v.into()),
            industries: industries.map(|v| v.into()).collect(),
            filters: serde_json::from_str(filters.0.get()).unwrap(),
            status: status.into(),
            created_at,
            updated_at,
            shortlist_generated_at,
            shortlisted,
            invited,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct StartShortlistGeneration {
    pub previous: Option<chrono::DateTime<chrono::Utc>>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ShortlistItems {
    pub candidate_id: uuid::Uuid,
    pub score: f64,
    pub rank: i32,
    pub breakdown: serde_json::Value,
    pub in_current: bool,
    pub pinned: bool,
    pub dismissed: bool,
    pub first_seen_at: chrono::DateTime<chrono::Utc>,
}
pub struct ShortlistItemsBorrowed<'a> {
    pub candidate_id: uuid::Uuid,
    pub score: f64,
    pub rank: i32,
    pub breakdown: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub in_current: bool,
    pub pinned: bool,
    pub dismissed: bool,
    pub first_seen_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<ShortlistItemsBorrowed<'a>> for ShortlistItems {
    fn from(
        ShortlistItemsBorrowed {
            candidate_id,
            score,
            rank,
            breakdown,
            in_current,
            pinned,
            dismissed,
            first_seen_at,
        }: ShortlistItemsBorrowed<'a>,
    ) -> Self {
        Self {
            candidate_id,
            score,
            rank,
            breakdown: serde_json::from_str(breakdown.0.get()).unwrap(),
            in_current,
            pinned,
            dismissed,
            first_seen_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CompanyNeedsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyNeedsBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyNeedsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyNeedsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyNeedsBorrowed) -> R,
    ) -> CompanyNeedsQuery<'c, 'a, 's, C, R, N> {
        CompanyNeedsQuery {
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
pub struct CompanyNeedQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyNeedBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyNeedBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyNeedQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyNeedBorrowed) -> R,
    ) -> CompanyNeedQuery<'c, 'a, 's, C, R, N> {
        CompanyNeedQuery {
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
pub struct StartShortlistGenerationQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<StartShortlistGeneration, tokio_postgres::Error>,
    mapper: fn(StartShortlistGeneration) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> StartShortlistGenerationQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(StartShortlistGeneration) -> R,
    ) -> StartShortlistGenerationQuery<'c, 'a, 's, C, R, N> {
        StartShortlistGenerationQuery {
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
pub struct ShortlistItemsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<ShortlistItemsBorrowed, tokio_postgres::Error>,
    mapper: fn(ShortlistItemsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ShortlistItemsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ShortlistItemsBorrowed) -> R,
    ) -> ShortlistItemsQuery<'c, 'a, 's, C, R, N> {
        ShortlistItemsQuery {
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
pub struct InsertNeedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_need() -> InsertNeedStmt {
    InsertNeedStmt(
        "INSERT INTO hiring_needs (id, company_id, title, description, specialization, grades, allow_adjacent_grades, must_skills, nice_skills, salary_min, salary_max, work_formats, city, industries) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        None,
    )
}
impl InsertNeedStmt {
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
        T5: crate::ArraySql<Item = T4>,
        T6: crate::StringSql,
        T7: crate::ArraySql<Item = T6>,
        T8: crate::StringSql,
        T9: crate::ArraySql<Item = T8>,
        T10: crate::StringSql,
        T11: crate::ArraySql<Item = T10>,
        T12: crate::StringSql,
        T13: crate::StringSql,
        T14: crate::ArraySql<Item = T13>,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
        title: &'a T1,
        description: &'a T2,
        specialization: &'a T3,
        grades: &'a T5,
        allow_adjacent_grades: &'a bool,
        must_skills: &'a T7,
        nice_skills: &'a T9,
        salary_min: &'a i64,
        salary_max: &'a i64,
        work_formats: &'a T11,
        city: &'a Option<T12>,
        industries: &'a T14,
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
                    grades,
                    allow_adjacent_grades,
                    must_skills,
                    nice_skills,
                    salary_min,
                    salary_max,
                    work_formats,
                    city,
                    industries,
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
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::ArraySql<Item = T6>,
    T8: crate::StringSql,
    T9: crate::ArraySql<Item = T8>,
    T10: crate::StringSql,
    T11: crate::ArraySql<Item = T10>,
    T12: crate::StringSql,
    T13: crate::StringSql,
    T14: crate::ArraySql<Item = T13>,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertNeedParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertNeedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertNeedParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14>,
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
            &params.grades,
            &params.allow_adjacent_grades,
            &params.must_skills,
            &params.nice_skills,
            &params.salary_min,
            &params.salary_max,
            &params.work_formats,
            &params.city,
            &params.industries,
        ))
    }
}
pub struct UpdateNeedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_need() -> UpdateNeedStmt {
    UpdateNeedStmt(
        "UPDATE hiring_needs SET title = $1, description = $2, specialization = $3, grades = $4, allow_adjacent_grades = $5, must_skills = $6, nice_skills = $7, salary_min = $8, salary_max = $9, work_formats = $10, city = $11, industries = $12, status = $13, updated_at = now() WHERE id = $14 AND company_id = $15",
        None,
    )
}
impl UpdateNeedStmt {
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
        T5: crate::ArraySql<Item = T4>,
        T6: crate::StringSql,
        T7: crate::ArraySql<Item = T6>,
        T8: crate::StringSql,
        T9: crate::ArraySql<Item = T8>,
        T10: crate::StringSql,
        T11: crate::ArraySql<Item = T10>,
        T12: crate::StringSql,
        T13: crate::StringSql,
        T14: crate::ArraySql<Item = T13>,
        T15: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        title: &'a T1,
        description: &'a T2,
        specialization: &'a T3,
        grades: &'a T5,
        allow_adjacent_grades: &'a bool,
        must_skills: &'a T7,
        nice_skills: &'a T9,
        salary_min: &'a i64,
        salary_max: &'a i64,
        work_formats: &'a T11,
        city: &'a Option<T12>,
        industries: &'a T14,
        status: &'a T15,
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
                    grades,
                    allow_adjacent_grades,
                    must_skills,
                    nice_skills,
                    salary_min,
                    salary_max,
                    work_formats,
                    city,
                    industries,
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
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::ArraySql<Item = T6>,
    T8: crate::StringSql,
    T9: crate::ArraySql<Item = T8>,
    T10: crate::StringSql,
    T11: crate::ArraySql<Item = T10>,
    T12: crate::StringSql,
    T13: crate::StringSql,
    T14: crate::ArraySql<Item = T13>,
    T15: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdateNeedParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdateNeedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdateNeedParams<
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
            T11,
            T12,
            T13,
            T14,
            T15,
        >,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.title,
            &params.description,
            &params.specialization,
            &params.grades,
            &params.allow_adjacent_grades,
            &params.must_skills,
            &params.nice_skills,
            &params.salary_min,
            &params.salary_max,
            &params.work_formats,
            &params.city,
            &params.industries,
            &params.status,
            &params.id,
            &params.company_id,
        ))
    }
}
pub struct CompanyNeedsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_needs() -> CompanyNeedsStmt {
    CompanyNeedsStmt(
        "SELECT n.id, n.title, n.description, n.specialization, n.grades, n.allow_adjacent_grades, n.must_skills, n.nice_skills, n.salary_min, n.salary_max, n.work_formats, n.city, n.industries, n.filters, n.status, n.created_at, n.updated_at, n.shortlist_generated_at, (SELECT count(*) FROM shortlist_items s WHERE s.need_id = n.id AND s.in_current AND NOT s.dismissed) AS shortlisted, (SELECT count(*) FROM invitations i WHERE i.need_id = n.id) AS invited FROM hiring_needs n WHERE n.company_id = $1 ORDER BY n.created_at DESC",
        None,
    )
}
impl CompanyNeedsStmt {
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
    ) -> CompanyNeedsQuery<'c, 'a, 's, C, CompanyNeeds, 1> {
        CompanyNeedsQuery {
            client,
            params: [company_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CompanyNeedsBorrowed, tokio_postgres::Error> {
                    Ok(CompanyNeedsBorrowed {
                        id: row.try_get(0)?,
                        title: row.try_get(1)?,
                        description: row.try_get(2)?,
                        specialization: row.try_get(3)?,
                        grades: row.try_get(4)?,
                        allow_adjacent_grades: row.try_get(5)?,
                        must_skills: row.try_get(6)?,
                        nice_skills: row.try_get(7)?,
                        salary_min: row.try_get(8)?,
                        salary_max: row.try_get(9)?,
                        work_formats: row.try_get(10)?,
                        city: row.try_get(11)?,
                        industries: row.try_get(12)?,
                        filters: row.try_get(13)?,
                        status: row.try_get(14)?,
                        created_at: row.try_get(15)?,
                        updated_at: row.try_get(16)?,
                        shortlist_generated_at: row.try_get(17)?,
                        shortlisted: row.try_get(18)?,
                        invited: row.try_get(19)?,
                    })
                },
            mapper: |it| CompanyNeeds::from(it),
        }
    }
}
pub struct CompanyNeedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_need() -> CompanyNeedStmt {
    CompanyNeedStmt(
        "SELECT n.id, n.title, n.description, n.specialization, n.grades, n.allow_adjacent_grades, n.must_skills, n.nice_skills, n.salary_min, n.salary_max, n.work_formats, n.city, n.industries, n.filters, n.status, n.created_at, n.updated_at, n.shortlist_generated_at, (SELECT count(*) FROM shortlist_items s WHERE s.need_id = n.id AND s.in_current AND NOT s.dismissed) AS shortlisted, (SELECT count(*) FROM invitations i WHERE i.need_id = n.id) AS invited FROM hiring_needs n WHERE n.company_id = $1 AND n.id = $2",
        None,
    )
}
impl CompanyNeedStmt {
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
    ) -> CompanyNeedQuery<'c, 'a, 's, C, CompanyNeed, 2> {
        CompanyNeedQuery {
            client,
            params: [company_id, id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CompanyNeedBorrowed, tokio_postgres::Error> {
                    Ok(CompanyNeedBorrowed {
                        id: row.try_get(0)?,
                        title: row.try_get(1)?,
                        description: row.try_get(2)?,
                        specialization: row.try_get(3)?,
                        grades: row.try_get(4)?,
                        allow_adjacent_grades: row.try_get(5)?,
                        must_skills: row.try_get(6)?,
                        nice_skills: row.try_get(7)?,
                        salary_min: row.try_get(8)?,
                        salary_max: row.try_get(9)?,
                        work_formats: row.try_get(10)?,
                        city: row.try_get(11)?,
                        industries: row.try_get(12)?,
                        filters: row.try_get(13)?,
                        status: row.try_get(14)?,
                        created_at: row.try_get(15)?,
                        updated_at: row.try_get(16)?,
                        shortlist_generated_at: row.try_get(17)?,
                        shortlisted: row.try_get(18)?,
                        invited: row.try_get(19)?,
                    })
                },
            mapper: |it| CompanyNeed::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CompanyNeedParams,
        CompanyNeedQuery<'c, 'a, 's, C, CompanyNeed, 2>,
        C,
    > for CompanyNeedStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CompanyNeedParams,
    ) -> CompanyNeedQuery<'c, 'a, 's, C, CompanyNeed, 2> {
        self.bind(client, &params.company_id, &params.id)
    }
}
pub struct DeleteNeedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_need() -> DeleteNeedStmt {
    DeleteNeedStmt(
        "DELETE FROM hiring_needs WHERE id = $1 AND company_id = $2",
        None,
    )
}
impl DeleteNeedStmt {
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
        DeleteNeedParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for DeleteNeedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a DeleteNeedParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.id, &params.company_id))
    }
}
pub struct StartShortlistGenerationStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn start_shortlist_generation() -> StartShortlistGenerationStmt {
    StartShortlistGenerationStmt(
        "UPDATE hiring_needs n SET filters = $1, shortlist_generated_at = now() FROM (SELECT shortlist_generated_at AS previous FROM hiring_needs WHERE id = $2 FOR UPDATE) prev WHERE n.id = $2 RETURNING prev.previous, n.shortlist_generated_at AS generated_at",
        None,
    )
}
impl StartShortlistGenerationStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::JsonSql>(
        &'s self,
        client: &'c C,
        filters: &'a T1,
        id: &'a uuid::Uuid,
    ) -> StartShortlistGenerationQuery<'c, 'a, 's, C, StartShortlistGeneration, 2> {
        StartShortlistGenerationQuery {
            client,
            params: [filters, id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<StartShortlistGeneration, tokio_postgres::Error> {
                Ok(StartShortlistGeneration {
                    previous: row.try_get(0)?,
                    generated_at: row.try_get(1)?,
                })
            },
            mapper: |it| StartShortlistGeneration::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::JsonSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        StartShortlistGenerationParams<T1>,
        StartShortlistGenerationQuery<'c, 'a, 's, C, StartShortlistGeneration, 2>,
        C,
    > for StartShortlistGenerationStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a StartShortlistGenerationParams<T1>,
    ) -> StartShortlistGenerationQuery<'c, 'a, 's, C, StartShortlistGeneration, 2> {
        self.bind(client, &params.filters, &params.id)
    }
}
pub struct MarkShortlistStaleStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn mark_shortlist_stale() -> MarkShortlistStaleStmt {
    MarkShortlistStaleStmt(
        "UPDATE shortlist_items SET in_current = false, updated_at = now() WHERE need_id = $1 AND NOT (candidate_id = ANY ($2))",
        None,
    )
}
impl MarkShortlistStaleStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::ArraySql<Item = uuid::Uuid>>(
        &'s self,
        client: &'c C,
        need_id: &'a uuid::Uuid,
        keep: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[need_id, keep]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::ArraySql<Item = uuid::Uuid>>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        MarkShortlistStaleParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for MarkShortlistStaleStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a MarkShortlistStaleParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.need_id, &params.keep))
    }
}
pub struct UpsertShortlistItemStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn upsert_shortlist_item() -> UpsertShortlistItemStmt {
    UpsertShortlistItemStmt(
        "INSERT INTO shortlist_items (need_id, candidate_id, score, rank, breakdown, in_current) VALUES ($1, $2, $3, $4, $5, true) ON CONFLICT (need_id, candidate_id) DO UPDATE SET score = EXCLUDED.score, rank = EXCLUDED.rank, breakdown = EXCLUDED.breakdown, in_current = true, updated_at = now()",
        None,
    )
}
impl UpsertShortlistItemStmt {
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
        need_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        score: &'a f64,
        rank: &'a i32,
        breakdown: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[need_id, candidate_id, score, rank, breakdown])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::JsonSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpsertShortlistItemParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpsertShortlistItemStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpsertShortlistItemParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.need_id,
            &params.candidate_id,
            &params.score,
            &params.rank,
            &params.breakdown,
        ))
    }
}
pub struct ShortlistItemsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn shortlist_items() -> ShortlistItemsStmt {
    ShortlistItemsStmt(
        "SELECT candidate_id, score, rank, breakdown, in_current, pinned, dismissed, first_seen_at FROM shortlist_items WHERE need_id = $1 ORDER BY in_current DESC, rank",
        None,
    )
}
impl ShortlistItemsStmt {
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
        need_id: &'a uuid::Uuid,
    ) -> ShortlistItemsQuery<'c, 'a, 's, C, ShortlistItems, 1> {
        ShortlistItemsQuery {
            client,
            params: [need_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ShortlistItemsBorrowed, tokio_postgres::Error> {
                Ok(ShortlistItemsBorrowed {
                    candidate_id: row.try_get(0)?,
                    score: row.try_get(1)?,
                    rank: row.try_get(2)?,
                    breakdown: row.try_get(3)?,
                    in_current: row.try_get(4)?,
                    pinned: row.try_get(5)?,
                    dismissed: row.try_get(6)?,
                    first_seen_at: row.try_get(7)?,
                })
            },
            mapper: |it| ShortlistItems::from(it),
        }
    }
}
pub struct PatchShortlistItemStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn patch_shortlist_item() -> PatchShortlistItemStmt {
    PatchShortlistItemStmt(
        "UPDATE shortlist_items s SET pinned = coalesce($1, s.pinned), dismissed = coalesce($2, s.dismissed), updated_at = now() FROM hiring_needs n WHERE s.need_id = n.id AND n.company_id = $3 AND s.need_id = $4 AND s.candidate_id = $5",
        None,
    )
}
impl PatchShortlistItemStmt {
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
        pinned: &'a Option<bool>,
        dismissed: &'a Option<bool>,
        company_id: &'a uuid::Uuid,
        need_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[pinned, dismissed, company_id, need_id, candidate_id],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        PatchShortlistItemParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for PatchShortlistItemStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a PatchShortlistItemParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.pinned,
            &params.dismissed,
            &params.company_id,
            &params.need_id,
            &params.candidate_id,
        ))
    }
}
