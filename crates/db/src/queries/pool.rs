// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct CandidatePoolParams<T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql> {
    pub text_query: Option<T1>,
    pub specialization: T2,
    pub grade: Option<T3>,
}
#[derive(Debug)]
pub struct CompanyRelationsParams<T1: crate::ArraySql<Item = uuid::Uuid>> {
    pub company_id: uuid::Uuid,
    pub ids: T1,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidatePool {
    pub user_id: uuid::Uuid,
    pub public_code: String,
    pub specialization: String,
    pub grade: String,
    pub theta: Option<f64>,
    pub salary_expectation_min: Option<i64>,
    pub work_formats: Vec<String>,
    pub city: Option<String>,
    pub relocation: bool,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub industries: Vec<String>,
    pub show_fsp: bool,
    pub achievements: Option<serde_json::Value>,
    pub skills: Vec<String>,
    pub skill_sources: Vec<String>,
    pub text_rank: f32,
}
pub struct CandidatePoolBorrowed<'a> {
    pub user_id: uuid::Uuid,
    pub public_code: &'a str,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub theta: Option<f64>,
    pub salary_expectation_min: Option<i64>,
    pub work_formats: crate::ArrayIterator<'a, &'a str>,
    pub city: Option<&'a str>,
    pub relocation: bool,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub industries: crate::ArrayIterator<'a, &'a str>,
    pub show_fsp: bool,
    pub achievements: Option<postgres_types::Json<&'a serde_json::value::RawValue>>,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub skill_sources: crate::ArrayIterator<'a, &'a str>,
    pub text_rank: f32,
}
impl<'a> From<CandidatePoolBorrowed<'a>> for CandidatePool {
    fn from(
        CandidatePoolBorrowed {
            user_id,
            public_code,
            specialization,
            grade,
            theta,
            salary_expectation_min,
            work_formats,
            city,
            relocation,
            last_activity_at,
            industries,
            show_fsp,
            achievements,
            skills,
            skill_sources,
            text_rank,
        }: CandidatePoolBorrowed<'a>,
    ) -> Self {
        Self {
            user_id,
            public_code: public_code.into(),
            specialization: specialization.into(),
            grade: grade.into(),
            theta,
            salary_expectation_min,
            work_formats: work_formats.map(|v| v.into()).collect(),
            city: city.map(|v| v.into()),
            relocation,
            last_activity_at,
            industries: industries.map(|v| v.into()).collect(),
            show_fsp,
            achievements: achievements.map(|v| serde_json::from_str(v.0.get()).unwrap()),
            skills: skills.map(|v| v.into()).collect(),
            skill_sources: skill_sources.map(|v| v.into()).collect(),
            text_rank,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateCards {
    pub user_id: uuid::Uuid,
    pub public_code: String,
    pub full_name_enc: Option<Vec<u8>>,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub city: Option<String>,
    pub relocation: bool,
    pub work_formats: Vec<String>,
    pub salary_expectation_min: Option<i64>,
    pub about: String,
    pub experience_years: Option<f32>,
    pub roles: Vec<String>,
    pub soft_skills: Vec<String>,
    pub industries: Vec<String>,
    pub experience: serde_json::Value,
    pub search_status: crate::types::SearchStatus,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub theta: Option<f64>,
    pub category_assigned_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub visible: bool,
    pub achievements: Option<serde_json::Value>,
    pub skills: Vec<String>,
    pub skill_sources: Vec<String>,
    pub task_rating: Option<f32>,
    pub tasks_done: i64,
}
pub struct CandidateCardsBorrowed<'a> {
    pub user_id: uuid::Uuid,
    pub public_code: &'a str,
    pub full_name_enc: Option<&'a [u8]>,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub city: Option<&'a str>,
    pub relocation: bool,
    pub work_formats: crate::ArrayIterator<'a, &'a str>,
    pub salary_expectation_min: Option<i64>,
    pub about: &'a str,
    pub experience_years: Option<f32>,
    pub roles: crate::ArrayIterator<'a, &'a str>,
    pub soft_skills: crate::ArrayIterator<'a, &'a str>,
    pub industries: crate::ArrayIterator<'a, &'a str>,
    pub experience: postgres_types::Json<&'a serde_json::value::RawValue>,
    pub search_status: crate::types::SearchStatus,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub theta: Option<f64>,
    pub category_assigned_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub visible: bool,
    pub achievements: Option<postgres_types::Json<&'a serde_json::value::RawValue>>,
    pub skills: crate::ArrayIterator<'a, &'a str>,
    pub skill_sources: crate::ArrayIterator<'a, &'a str>,
    pub task_rating: Option<f32>,
    pub tasks_done: i64,
}
impl<'a> From<CandidateCardsBorrowed<'a>> for CandidateCards {
    fn from(
        CandidateCardsBorrowed {
            user_id,
            public_code,
            full_name_enc,
            show_name,
            show_fsp,
            show_experience,
            city,
            relocation,
            work_formats,
            salary_expectation_min,
            about,
            experience_years,
            roles,
            soft_skills,
            industries,
            experience,
            search_status,
            specialization,
            grade,
            theta,
            category_assigned_at,
            last_activity_at,
            visible,
            achievements,
            skills,
            skill_sources,
            task_rating,
            tasks_done,
        }: CandidateCardsBorrowed<'a>,
    ) -> Self {
        Self {
            user_id,
            public_code: public_code.into(),
            full_name_enc: full_name_enc.map(|v| v.into()),
            show_name,
            show_fsp,
            show_experience,
            city: city.map(|v| v.into()),
            relocation,
            work_formats: work_formats.map(|v| v.into()).collect(),
            salary_expectation_min,
            about: about.into(),
            experience_years,
            roles: roles.map(|v| v.into()).collect(),
            soft_skills: soft_skills.map(|v| v.into()).collect(),
            industries: industries.map(|v| v.into()).collect(),
            experience: serde_json::from_str(experience.0.get()).unwrap(),
            search_status,
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
            theta,
            category_assigned_at,
            last_activity_at,
            visible,
            achievements: achievements.map(|v| serde_json::from_str(v.0.get()).unwrap()),
            skills: skills.map(|v| v.into()).collect(),
            skill_sources: skill_sources.map(|v| v.into()).collect(),
            task_rating,
            tasks_done,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CompanyRelations {
    pub candidate_id: uuid::Uuid,
    pub invitation_status: Option<crate::types::InvitationStatus>,
    pub applied: bool,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CandidatePoolQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CandidatePoolBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidatePoolBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidatePoolQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidatePoolBorrowed) -> R,
    ) -> CandidatePoolQuery<'c, 'a, 's, C, R, N> {
        CandidatePoolQuery {
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
pub struct CandidateCardsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CandidateCardsBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateCardsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateCardsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateCardsBorrowed) -> R,
    ) -> CandidateCardsQuery<'c, 'a, 's, C, R, N> {
        CandidateCardsQuery {
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
pub struct CompanyRelationsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyRelations, tokio_postgres::Error>,
    mapper: fn(CompanyRelations) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyRelationsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyRelations) -> R,
    ) -> CompanyRelationsQuery<'c, 'a, 's, C, R, N> {
        CompanyRelationsQuery {
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
pub struct CandidatePoolStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_pool() -> CandidatePoolStmt {
    CandidatePoolStmt(
        "SELECT c.user_id, c.public_code, c.specialization, c.grade, c.theta, c.salary_expectation_min, c.work_formats, c.city, c.relocation, c.last_activity_at, c.industries, c.show_fsp, f.achievements, coalesce(array_agg(s.skill ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skills, coalesce(array_agg(s.source ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skill_sources, CASE WHEN $1::text IS NULL THEN 0::real ELSE ts_rank_cd(c.about_tsv, to_tsquery('russian', $1))::real END AS text_rank FROM candidates c JOIN users u ON u.id = c.user_id AND u.deleted_at IS NULL LEFT JOIN fsp_links f ON f.candidate_id = c.user_id LEFT JOIN candidate_skills s ON s.candidate_id = c.user_id WHERE c.visible AND c.search_status <> 'closed' AND c.specialization = $2 AND c.grade IS NOT NULL AND ($3::text IS NULL OR c.grade = $3) GROUP BY c.user_id, f.achievements",
        None,
    )
}
impl CandidatePoolStmt {
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
    >(
        &'s self,
        client: &'c C,
        text_query: &'a Option<T1>,
        specialization: &'a T2,
        grade: &'a Option<T3>,
    ) -> CandidatePoolQuery<'c, 'a, 's, C, CandidatePool, 3> {
        CandidatePoolQuery {
            client,
            params: [text_query, specialization, grade],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CandidatePoolBorrowed, tokio_postgres::Error> {
                    Ok(CandidatePoolBorrowed {
                        user_id: row.try_get(0)?,
                        public_code: row.try_get(1)?,
                        specialization: row.try_get(2)?,
                        grade: row.try_get(3)?,
                        theta: row.try_get(4)?,
                        salary_expectation_min: row.try_get(5)?,
                        work_formats: row.try_get(6)?,
                        city: row.try_get(7)?,
                        relocation: row.try_get(8)?,
                        last_activity_at: row.try_get(9)?,
                        industries: row.try_get(10)?,
                        show_fsp: row.try_get(11)?,
                        achievements: row.try_get(12)?,
                        skills: row.try_get(13)?,
                        skill_sources: row.try_get(14)?,
                        text_rank: row.try_get(15)?,
                    })
                },
            mapper: |it| CandidatePool::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql, T3: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CandidatePoolParams<T1, T2, T3>,
        CandidatePoolQuery<'c, 'a, 's, C, CandidatePool, 3>,
        C,
    > for CandidatePoolStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CandidatePoolParams<T1, T2, T3>,
    ) -> CandidatePoolQuery<'c, 'a, 's, C, CandidatePool, 3> {
        self.bind(
            client,
            &params.text_query,
            &params.specialization,
            &params.grade,
        )
    }
}
pub struct CandidateCardsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_cards() -> CandidateCardsStmt {
    CandidateCardsStmt(
        "SELECT c.user_id, c.public_code, c.full_name_enc, c.show_name, c.show_fsp, c.show_experience, c.city, c.relocation, c.work_formats, c.salary_expectation_min, c.about, c.experience_years, c.roles, c.soft_skills, c.industries, c.experience, c.search_status, c.specialization, c.grade, c.theta, c.category_assigned_at, c.last_activity_at, c.visible, f.achievements, coalesce(array_agg(s.skill ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skills, coalesce(array_agg(s.source ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skill_sources, (SELECT avg(a.rating)::real FROM task_assignments a WHERE a.candidate_id = c.user_id AND a.status = 'reviewed') AS task_rating, (SELECT count(*) FROM task_assignments a WHERE a.candidate_id = c.user_id AND a.status IN ('submitted', 'reviewed')) AS tasks_done FROM candidates c LEFT JOIN fsp_links f ON f.candidate_id = c.user_id LEFT JOIN candidate_skills s ON s.candidate_id = c.user_id WHERE c.user_id = ANY ($1) GROUP BY c.user_id, f.achievements",
        None,
    )
}
impl CandidateCardsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::ArraySql<Item = uuid::Uuid>>(
        &'s self,
        client: &'c C,
        ids: &'a T1,
    ) -> CandidateCardsQuery<'c, 'a, 's, C, CandidateCards, 1> {
        CandidateCardsQuery {
            client,
            params: [ids],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateCardsBorrowed, tokio_postgres::Error> {
                Ok(CandidateCardsBorrowed {
                    user_id: row.try_get(0)?,
                    public_code: row.try_get(1)?,
                    full_name_enc: row.try_get(2)?,
                    show_name: row.try_get(3)?,
                    show_fsp: row.try_get(4)?,
                    show_experience: row.try_get(5)?,
                    city: row.try_get(6)?,
                    relocation: row.try_get(7)?,
                    work_formats: row.try_get(8)?,
                    salary_expectation_min: row.try_get(9)?,
                    about: row.try_get(10)?,
                    experience_years: row.try_get(11)?,
                    roles: row.try_get(12)?,
                    soft_skills: row.try_get(13)?,
                    industries: row.try_get(14)?,
                    experience: row.try_get(15)?,
                    search_status: row.try_get(16)?,
                    specialization: row.try_get(17)?,
                    grade: row.try_get(18)?,
                    theta: row.try_get(19)?,
                    category_assigned_at: row.try_get(20)?,
                    last_activity_at: row.try_get(21)?,
                    visible: row.try_get(22)?,
                    achievements: row.try_get(23)?,
                    skills: row.try_get(24)?,
                    skill_sources: row.try_get(25)?,
                    task_rating: row.try_get(26)?,
                    tasks_done: row.try_get(27)?,
                })
            },
            mapper: |it| CandidateCards::from(it),
        }
    }
}
pub struct CompanyRelationsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_relations() -> CompanyRelationsStmt {
    CompanyRelationsStmt(
        "SELECT c.user_id AS candidate_id, (SELECT i.status FROM invitations i WHERE i.company_id = $1 AND i.candidate_id = c.user_id ORDER BY i.created_at DESC LIMIT 1) AS invitation_status, EXISTS (SELECT 1 FROM applications a JOIN vacancies v ON v.id = a.vacancy_id WHERE v.company_id = $1 AND a.candidate_id = c.user_id AND a.status <> 'withdrawn') AS applied FROM candidates c WHERE c.user_id = ANY ($2)",
        None,
    )
}
impl CompanyRelationsStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::ArraySql<Item = uuid::Uuid>>(
        &'s self,
        client: &'c C,
        company_id: &'a uuid::Uuid,
        ids: &'a T1,
    ) -> CompanyRelationsQuery<'c, 'a, 's, C, CompanyRelations, 2> {
        CompanyRelationsQuery {
            client,
            params: [company_id, ids],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CompanyRelations, tokio_postgres::Error> {
                    Ok(CompanyRelations {
                        candidate_id: row.try_get(0)?,
                        invitation_status: row.try_get(1)?,
                        applied: row.try_get(2)?,
                    })
                },
            mapper: |it| CompanyRelations::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::ArraySql<Item = uuid::Uuid>>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CompanyRelationsParams<T1>,
        CompanyRelationsQuery<'c, 'a, 's, C, CompanyRelations, 2>,
        C,
    > for CompanyRelationsStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CompanyRelationsParams<T1>,
    ) -> CompanyRelationsQuery<'c, 'a, 's, C, CompanyRelations, 2> {
        self.bind(client, &params.company_id, &params.ids)
    }
}
