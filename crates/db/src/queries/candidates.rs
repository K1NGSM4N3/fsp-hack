// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertCandidateParams<T1: crate::StringSql> {
    pub user_id: uuid::Uuid,
    pub public_code: T1,
}
#[derive(Debug)]
pub struct UpdateCandidateProfileParams<
    T1: crate::BytesSql,
    T2: crate::BytesSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::StringSql,
    T8: crate::ArraySql<Item = T7>,
    T9: crate::StringSql,
    T10: crate::ArraySql<Item = T9>,
    T11: crate::StringSql,
    T12: crate::ArraySql<Item = T11>,
    T13: crate::JsonSql,
> {
    pub full_name_enc: Option<T1>,
    pub contacts_enc: Option<T2>,
    pub city: Option<T3>,
    pub relocation: bool,
    pub work_formats: T5,
    pub salary_expectation_min: Option<i64>,
    pub about: T6,
    pub experience_years: Option<f32>,
    pub roles: T8,
    pub soft_skills: T10,
    pub industries: T12,
    pub experience: T13,
    pub user_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct UpdateCandidatePrivacyParams {
    pub visible: bool,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub search_status: crate::types::SearchStatus,
    pub user_id: uuid::Uuid,
}
#[derive(Debug)]
pub struct DeleteCandidateSkillsBySourceParams<T1: crate::StringSql> {
    pub candidate_id: uuid::Uuid,
    pub source: T1,
}
#[derive(Debug)]
pub struct InsertCandidateSkillParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub candidate_id: uuid::Uuid,
    pub skill: T1,
    pub source: T2,
}
#[derive(Debug)]
pub struct InsertResumeUploadParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::ArraySql<Item = T2>,
> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub filename: T1,
    pub size_bytes: i32,
    pub recognized_fields: T3,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateProfile {
    pub user_id: uuid::Uuid,
    pub public_code: String,
    pub full_name_enc: Option<Vec<u8>>,
    pub contacts_enc: Option<Vec<u8>>,
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
    pub visible: bool,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub theta: Option<f64>,
    pub category_assigned_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
pub struct CandidateProfileBorrowed<'a> {
    pub user_id: uuid::Uuid,
    pub public_code: &'a str,
    pub full_name_enc: Option<&'a [u8]>,
    pub contacts_enc: Option<&'a [u8]>,
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
    pub visible: bool,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub theta: Option<f64>,
    pub category_assigned_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<CandidateProfileBorrowed<'a>> for CandidateProfile {
    fn from(
        CandidateProfileBorrowed {
            user_id,
            public_code,
            full_name_enc,
            contacts_enc,
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
            visible,
            show_name,
            show_fsp,
            show_experience,
            specialization,
            grade,
            theta,
            category_assigned_at,
            last_activity_at,
            updated_at,
        }: CandidateProfileBorrowed<'a>,
    ) -> Self {
        Self {
            user_id,
            public_code: public_code.into(),
            full_name_enc: full_name_enc.map(|v| v.into()),
            contacts_enc: contacts_enc.map(|v| v.into()),
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
            visible,
            show_name,
            show_fsp,
            show_experience,
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
            theta,
            category_assigned_at,
            last_activity_at,
            updated_at,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateSkills {
    pub skill: String,
    pub source: String,
}
pub struct CandidateSkillsBorrowed<'a> {
    pub skill: &'a str,
    pub source: &'a str,
}
impl<'a> From<CandidateSkillsBorrowed<'a>> for CandidateSkills {
    fn from(CandidateSkillsBorrowed { skill, source }: CandidateSkillsBorrowed<'a>) -> Self {
        Self {
            skill: skill.into(),
            source: source.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateInvitable {
    pub visible: bool,
    pub search_status: crate::types::SearchStatus,
    pub specialization: Option<String>,
    pub grade: Option<String>,
}
pub struct CandidateInvitableBorrowed<'a> {
    pub visible: bool,
    pub search_status: crate::types::SearchStatus,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
}
impl<'a> From<CandidateInvitableBorrowed<'a>> for CandidateInvitable {
    fn from(
        CandidateInvitableBorrowed {
            visible,
            search_status,
            specialization,
            grade,
        }: CandidateInvitableBorrowed<'a>,
    ) -> Self {
        Self {
            visible,
            search_status,
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CandidateProfileQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CandidateProfileBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateProfileBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateProfileQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateProfileBorrowed) -> R,
    ) -> CandidateProfileQuery<'c, 'a, 's, C, R, N> {
        CandidateProfileQuery {
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
pub struct CandidateSkillsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CandidateSkillsBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateSkillsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateSkillsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateSkillsBorrowed) -> R,
    ) -> CandidateSkillsQuery<'c, 'a, 's, C, R, N> {
        CandidateSkillsQuery {
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
pub struct CandidateInvitableQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CandidateInvitableBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateInvitableBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateInvitableQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateInvitableBorrowed) -> R,
    ) -> CandidateInvitableQuery<'c, 'a, 's, C, R, N> {
        CandidateInvitableQuery {
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
pub struct InsertCandidateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_candidate() -> InsertCandidateStmt {
    InsertCandidateStmt(
        "INSERT INTO candidates (user_id, public_code) VALUES ($1, $2)",
        None,
    )
}
impl InsertCandidateStmt {
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
        user_id: &'a uuid::Uuid,
        public_code: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[user_id, public_code]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertCandidateParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertCandidateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertCandidateParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.user_id, &params.public_code))
    }
}
pub struct CandidateProfileStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_profile() -> CandidateProfileStmt {
    CandidateProfileStmt(
        "SELECT user_id, public_code, full_name_enc, contacts_enc, city, relocation, work_formats, salary_expectation_min, about, experience_years, roles, soft_skills, industries, experience, search_status, visible, show_name, show_fsp, show_experience, specialization, grade, theta, category_assigned_at, last_activity_at, updated_at FROM candidates WHERE user_id = $1",
        None,
    )
}
impl CandidateProfileStmt {
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
    ) -> CandidateProfileQuery<'c, 'a, 's, C, CandidateProfile, 1> {
        CandidateProfileQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateProfileBorrowed, tokio_postgres::Error> {
                Ok(CandidateProfileBorrowed {
                    user_id: row.try_get(0)?,
                    public_code: row.try_get(1)?,
                    full_name_enc: row.try_get(2)?,
                    contacts_enc: row.try_get(3)?,
                    city: row.try_get(4)?,
                    relocation: row.try_get(5)?,
                    work_formats: row.try_get(6)?,
                    salary_expectation_min: row.try_get(7)?,
                    about: row.try_get(8)?,
                    experience_years: row.try_get(9)?,
                    roles: row.try_get(10)?,
                    soft_skills: row.try_get(11)?,
                    industries: row.try_get(12)?,
                    experience: row.try_get(13)?,
                    search_status: row.try_get(14)?,
                    visible: row.try_get(15)?,
                    show_name: row.try_get(16)?,
                    show_fsp: row.try_get(17)?,
                    show_experience: row.try_get(18)?,
                    specialization: row.try_get(19)?,
                    grade: row.try_get(20)?,
                    theta: row.try_get(21)?,
                    category_assigned_at: row.try_get(22)?,
                    last_activity_at: row.try_get(23)?,
                    updated_at: row.try_get(24)?,
                })
            },
            mapper: |it| CandidateProfile::from(it),
        }
    }
}
pub struct UpdateCandidateProfileStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_candidate_profile() -> UpdateCandidateProfileStmt {
    UpdateCandidateProfileStmt(
        "UPDATE candidates SET full_name_enc = $1, contacts_enc = $2, city = $3, relocation = $4, work_formats = $5, salary_expectation_min = $6, about = $7, experience_years = $8, roles = $9, soft_skills = $10, industries = $11, experience = $12, updated_at = now(), last_activity_at = now() WHERE user_id = $13",
        None,
    )
}
impl UpdateCandidateProfileStmt {
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
        T1: crate::BytesSql,
        T2: crate::BytesSql,
        T3: crate::StringSql,
        T4: crate::StringSql,
        T5: crate::ArraySql<Item = T4>,
        T6: crate::StringSql,
        T7: crate::StringSql,
        T8: crate::ArraySql<Item = T7>,
        T9: crate::StringSql,
        T10: crate::ArraySql<Item = T9>,
        T11: crate::StringSql,
        T12: crate::ArraySql<Item = T11>,
        T13: crate::JsonSql,
    >(
        &'s self,
        client: &'c C,
        full_name_enc: &'a Option<T1>,
        contacts_enc: &'a Option<T2>,
        city: &'a Option<T3>,
        relocation: &'a bool,
        work_formats: &'a T5,
        salary_expectation_min: &'a Option<i64>,
        about: &'a T6,
        experience_years: &'a Option<f32>,
        roles: &'a T8,
        soft_skills: &'a T10,
        industries: &'a T12,
        experience: &'a T13,
        user_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    full_name_enc,
                    contacts_enc,
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
                    user_id,
                ],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::BytesSql,
    T2: crate::BytesSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::ArraySql<Item = T4>,
    T6: crate::StringSql,
    T7: crate::StringSql,
    T8: crate::ArraySql<Item = T7>,
    T9: crate::StringSql,
    T10: crate::ArraySql<Item = T9>,
    T11: crate::StringSql,
    T12: crate::ArraySql<Item = T11>,
    T13: crate::JsonSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdateCandidateProfileParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdateCandidateProfileStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdateCandidateProfileParams<
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
        >,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.full_name_enc,
            &params.contacts_enc,
            &params.city,
            &params.relocation,
            &params.work_formats,
            &params.salary_expectation_min,
            &params.about,
            &params.experience_years,
            &params.roles,
            &params.soft_skills,
            &params.industries,
            &params.experience,
            &params.user_id,
        ))
    }
}
pub struct UpdateCandidatePrivacyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_candidate_privacy() -> UpdateCandidatePrivacyStmt {
    UpdateCandidatePrivacyStmt(
        "UPDATE candidates SET visible = $1, show_name = $2, show_fsp = $3, show_experience = $4, search_status = $5, updated_at = now() WHERE user_id = $6",
        None,
    )
}
impl UpdateCandidatePrivacyStmt {
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
        visible: &'a bool,
        show_name: &'a bool,
        show_fsp: &'a bool,
        show_experience: &'a bool,
        search_status: &'a crate::types::SearchStatus,
        user_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    visible,
                    show_name,
                    show_fsp,
                    show_experience,
                    search_status,
                    user_id,
                ],
            )
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdateCandidatePrivacyParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdateCandidatePrivacyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdateCandidatePrivacyParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.visible,
            &params.show_name,
            &params.show_fsp,
            &params.show_experience,
            &params.search_status,
            &params.user_id,
        ))
    }
}
pub struct CandidateSkillsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_skills() -> CandidateSkillsStmt {
    CandidateSkillsStmt(
        "SELECT skill, source FROM candidate_skills WHERE candidate_id = $1 ORDER BY skill, source",
        None,
    )
}
impl CandidateSkillsStmt {
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
    ) -> CandidateSkillsQuery<'c, 'a, 's, C, CandidateSkills, 1> {
        CandidateSkillsQuery {
            client,
            params: [candidate_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateSkillsBorrowed, tokio_postgres::Error> {
                Ok(CandidateSkillsBorrowed {
                    skill: row.try_get(0)?,
                    source: row.try_get(1)?,
                })
            },
            mapper: |it| CandidateSkills::from(it),
        }
    }
}
pub struct DeleteCandidateSkillsBySourceStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_candidate_skills_by_source() -> DeleteCandidateSkillsBySourceStmt {
    DeleteCandidateSkillsBySourceStmt(
        "DELETE FROM candidate_skills WHERE candidate_id = $1 AND source = $2",
        None,
    )
}
impl DeleteCandidateSkillsBySourceStmt {
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
        candidate_id: &'a uuid::Uuid,
        source: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[candidate_id, source]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        DeleteCandidateSkillsBySourceParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for DeleteCandidateSkillsBySourceStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a DeleteCandidateSkillsBySourceParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.candidate_id, &params.source))
    }
}
pub struct InsertCandidateSkillStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_candidate_skill() -> InsertCandidateSkillStmt {
    InsertCandidateSkillStmt(
        "INSERT INTO candidate_skills (candidate_id, skill, source) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        None,
    )
}
impl InsertCandidateSkillStmt {
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
        skill: &'a T1,
        source: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[candidate_id, skill, source]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertCandidateSkillParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertCandidateSkillStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertCandidateSkillParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.candidate_id, &params.skill, &params.source))
    }
}
pub struct InsertResumeUploadStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_resume_upload() -> InsertResumeUploadStmt {
    InsertResumeUploadStmt(
        "INSERT INTO resume_uploads (id, candidate_id, filename, size_bytes, recognized_fields) VALUES ($1, $2, $3, $4, $5)",
        None,
    )
}
impl InsertResumeUploadStmt {
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
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        filename: &'a T1,
        size_bytes: &'a i32,
        recognized_fields: &'a T3,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[id, candidate_id, filename, size_bytes, recognized_fields],
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
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertResumeUploadParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertResumeUploadStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertResumeUploadParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.candidate_id,
            &params.filename,
            &params.size_bytes,
            &params.recognized_fields,
        ))
    }
}
pub struct TouchCandidateActivityStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn touch_candidate_activity() -> TouchCandidateActivityStmt {
    TouchCandidateActivityStmt(
        "UPDATE candidates SET last_activity_at = now() WHERE user_id = $1",
        None,
    )
}
impl TouchCandidateActivityStmt {
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
        user_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[user_id]).await
    }
}
pub struct CandidateInvitableStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_invitable() -> CandidateInvitableStmt {
    CandidateInvitableStmt(
        "SELECT visible, search_status, specialization, grade FROM candidates WHERE user_id = $1",
        None,
    )
}
impl CandidateInvitableStmt {
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
    ) -> CandidateInvitableQuery<'c, 'a, 's, C, CandidateInvitable, 1> {
        CandidateInvitableQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateInvitableBorrowed, tokio_postgres::Error> {
                Ok(CandidateInvitableBorrowed {
                    visible: row.try_get(0)?,
                    search_status: row.try_get(1)?,
                    specialization: row.try_get(2)?,
                    grade: row.try_get(3)?,
                })
            },
            mapper: |it| CandidateInvitable::from(it),
        }
    }
}
