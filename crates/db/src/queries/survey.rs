// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertSurveyParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::ArraySql<Item = T6>,
    T8: crate::JsonSql,
> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub direction: T1,
    pub specialization: T2,
    pub claimed_grade: T3,
    pub skills: T5,
    pub industries: T7,
    pub experience_years: Option<f32>,
    pub self_assessment: T8,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SurveyHistory {
    pub id: uuid::Uuid,
    pub direction: String,
    pub specialization: String,
    pub claimed_grade: String,
    pub skills: Vec<String>,
    pub industries: Vec<String>,
    pub experience_years: Option<f32>,
    pub self_assessment: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
pub struct SurveyHistoryBorrowed<'a> {
    pub id: uuid::Uuid,
    pub direction: &'a str,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub industries: crate::ArrayIterator<'a, &'a str>,
    pub experience_years: Option<f32>,
    pub self_assessment: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<SurveyHistoryBorrowed<'a>> for SurveyHistory {
    fn from(
        SurveyHistoryBorrowed {
            id,
            direction,
            specialization,
            claimed_grade,
            skills,
            industries,
            experience_years,
            self_assessment,
            created_at,
        }: SurveyHistoryBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            direction: direction.into(),
            specialization: specialization.into(),
            claimed_grade: claimed_grade.into(),
            skills: skills.map(|v| v.into()).collect(),
            industries: industries.map(|v| v.into()).collect(),
            experience_years,
            self_assessment: serde_json::from_str(self_assessment.0.get()).unwrap(),
            created_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LatestSurvey {
    pub id: uuid::Uuid,
    pub direction: String,
    pub specialization: String,
    pub claimed_grade: String,
    pub skills: Vec<String>,
    pub industries: Vec<String>,
    pub experience_years: Option<f32>,
    pub self_assessment: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
pub struct LatestSurveyBorrowed<'a> {
    pub id: uuid::Uuid,
    pub direction: &'a str,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub industries: crate::ArrayIterator<'a, &'a str>,
    pub experience_years: Option<f32>,
    pub self_assessment: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<LatestSurveyBorrowed<'a>> for LatestSurvey {
    fn from(
        LatestSurveyBorrowed {
            id,
            direction,
            specialization,
            claimed_grade,
            skills,
            industries,
            experience_years,
            self_assessment,
            created_at,
        }: LatestSurveyBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            direction: direction.into(),
            specialization: specialization.into(),
            claimed_grade: claimed_grade.into(),
            skills: skills.map(|v| v.into()).collect(),
            industries: industries.map(|v| v.into()).collect(),
            experience_years,
            self_assessment: serde_json::from_str(self_assessment.0.get()).unwrap(),
            created_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct SurveyHistoryQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<SurveyHistoryBorrowed, tokio_postgres::Error>,
    mapper: fn(SurveyHistoryBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> SurveyHistoryQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(SurveyHistoryBorrowed) -> R,
    ) -> SurveyHistoryQuery<'c, 'a, 's, C, R, N> {
        SurveyHistoryQuery {
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
pub struct LatestSurveyQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<LatestSurveyBorrowed, tokio_postgres::Error>,
    mapper: fn(LatestSurveyBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> LatestSurveyQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(LatestSurveyBorrowed) -> R,
    ) -> LatestSurveyQuery<'c, 'a, 's, C, R, N> {
        LatestSurveyQuery {
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
pub struct InsertSurveyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_survey() -> InsertSurveyStmt {
    InsertSurveyStmt(
        "INSERT INTO survey_responses (id, candidate_id, direction, specialization, claimed_grade, skills, industries, experience_years, self_assessment) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        None,
    )
}
impl InsertSurveyStmt {
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
        T8: crate::JsonSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        direction: &'a T1,
        specialization: &'a T2,
        claimed_grade: &'a T3,
        skills: &'a T5,
        industries: &'a T7,
        experience_years: &'a Option<f32>,
        self_assessment: &'a T8,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    candidate_id,
                    direction,
                    specialization,
                    claimed_grade,
                    skills,
                    industries,
                    experience_years,
                    self_assessment,
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
    T8: crate::JsonSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertSurveyParams<T1, T2, T3, T4, T5, T6, T7, T8>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertSurveyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertSurveyParams<T1, T2, T3, T4, T5, T6, T7, T8>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.candidate_id,
            &params.direction,
            &params.specialization,
            &params.claimed_grade,
            &params.skills,
            &params.industries,
            &params.experience_years,
            &params.self_assessment,
        ))
    }
}
pub struct SurveyHistoryStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn survey_history() -> SurveyHistoryStmt {
    SurveyHistoryStmt(
        "SELECT id, direction, specialization, claimed_grade, skills, industries, experience_years, self_assessment, created_at FROM survey_responses WHERE candidate_id = $1 ORDER BY created_at DESC",
        None,
    )
}
impl SurveyHistoryStmt {
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
    ) -> SurveyHistoryQuery<'c, 'a, 's, C, SurveyHistory, 1> {
        SurveyHistoryQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<SurveyHistoryBorrowed, tokio_postgres::Error> {
                    Ok(SurveyHistoryBorrowed {
                        id: row.try_get(0)?,
                        direction: row.try_get(1)?,
                        specialization: row.try_get(2)?,
                        claimed_grade: row.try_get(3)?,
                        skills: row.try_get(4)?,
                        industries: row.try_get(5)?,
                        experience_years: row.try_get(6)?,
                        self_assessment: row.try_get(7)?,
                        created_at: row.try_get(8)?,
                    })
                },
            mapper: |it| SurveyHistory::from(it),
        }
    }
}
pub struct LatestSurveyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn latest_survey() -> LatestSurveyStmt {
    LatestSurveyStmt(
        "SELECT id, direction, specialization, claimed_grade, skills, industries, experience_years, self_assessment, created_at FROM survey_responses WHERE candidate_id = $1 ORDER BY created_at DESC LIMIT 1",
        None,
    )
}
impl LatestSurveyStmt {
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
    ) -> LatestSurveyQuery<'c, 'a, 's, C, LatestSurvey, 1> {
        LatestSurveyQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<LatestSurveyBorrowed, tokio_postgres::Error> {
                    Ok(LatestSurveyBorrowed {
                        id: row.try_get(0)?,
                        direction: row.try_get(1)?,
                        specialization: row.try_get(2)?,
                        claimed_grade: row.try_get(3)?,
                        skills: row.try_get(4)?,
                        industries: row.try_get(5)?,
                        experience_years: row.try_get(6)?,
                        self_assessment: row.try_get(7)?,
                        created_at: row.try_get(8)?,
                    })
                },
            mapper: |it| LatestSurvey::from(it),
        }
    }
}
