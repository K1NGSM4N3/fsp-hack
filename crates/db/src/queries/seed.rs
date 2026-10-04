// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct SeedVerifiedUserParams<T1: crate::BytesSql, T2: crate::BytesSql, T3: crate::StringSql> {
    pub id: uuid::Uuid,
    pub role: crate::types::UserRole,
    pub email_enc: T1,
    pub email_bidx: T2,
    pub password_hash: T3,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug)]
pub struct SeedCandidateParams<
    T1: crate::StringSql,
    T2: crate::BytesSql,
    T3: crate::BytesSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::ArraySql<Item = T5>,
    T7: crate::StringSql,
    T8: crate::StringSql,
    T9: crate::ArraySql<Item = T8>,
    T10: crate::StringSql,
    T11: crate::ArraySql<Item = T10>,
    T12: crate::StringSql,
    T13: crate::ArraySql<Item = T12>,
    T14: crate::JsonSql,
> {
    pub user_id: uuid::Uuid,
    pub public_code: T1,
    pub full_name_enc: T2,
    pub contacts_enc: T3,
    pub city: Option<T4>,
    pub relocation: bool,
    pub work_formats: T6,
    pub salary_expectation_min: Option<i64>,
    pub about: T7,
    pub experience_years: Option<f32>,
    pub roles: T9,
    pub soft_skills: T11,
    pub industries: T13,
    pub experience: T14,
    pub search_status: crate::types::SearchStatus,
    pub show_name: bool,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
#[derive(Debug)]
pub struct SeedCompanyParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
> {
    pub id: uuid::Uuid,
    pub owner_user_id: uuid::Uuid,
    pub name: T1,
    pub description: T2,
    pub industry: T3,
    pub city: T4,
    pub size: T5,
    pub contact_email: T6,
    pub contact_telegram: T7,
}
#[derive(Debug)]
pub struct SeedFspLinkParams<T1: crate::StringSql, T2: crate::JsonSql> {
    pub candidate_id: uuid::Uuid,
    pub fsp_id: T1,
    pub achievements: T2,
}
#[derive(Debug)]
pub struct SeedSessionParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::JsonSql,
    T5: crate::StringSql,
> {
    pub id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub specialization: T1,
    pub claimed_grade: T2,
    pub attempt_kind: T3,
    pub status: crate::types::AssessmentStatus,
    pub state: T4,
    pub theta: f64,
    pub se: f64,
    pub measured_grade: Option<T5>,
    pub confident_above_next: bool,
    pub person_fit_lz: f64,
    pub aberrant: bool,
    pub items_count: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: chrono::DateTime<chrono::Utc>,
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
pub struct DemoSeededStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn demo_seeded() -> DemoSeededStmt {
    DemoSeededStmt("SELECT count(*) AS count FROM companies", None)
}
impl DemoSeededStmt {
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
    ) -> I64Query<'c, 'a, 's, C, i64, 0> {
        I64Query {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |row| Ok(row.try_get(0)?),
            mapper: |it| it,
        }
    }
}
pub struct SeedVerifiedUserStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn seed_verified_user() -> SeedVerifiedUserStmt {
    SeedVerifiedUserStmt(
        "INSERT INTO users (id, role, email_enc, email_bidx, password_hash, email_verified_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $6)",
        None,
    )
}
impl SeedVerifiedUserStmt {
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
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        role: &'a crate::types::UserRole,
        email_enc: &'a T1,
        email_bidx: &'a T2,
        password_hash: &'a T3,
        created_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[id, role, email_enc, email_bidx, password_hash, created_at],
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
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SeedVerifiedUserParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SeedVerifiedUserStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SeedVerifiedUserParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.role,
            &params.email_enc,
            &params.email_bidx,
            &params.password_hash,
            &params.created_at,
        ))
    }
}
pub struct SeedCandidateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn seed_candidate() -> SeedCandidateStmt {
    SeedCandidateStmt(
        "INSERT INTO candidates (user_id, public_code, full_name_enc, contacts_enc, city, relocation, work_formats, salary_expectation_min, about, experience_years, roles, soft_skills, industries, experience, search_status, show_name, last_activity_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
        None,
    )
}
impl SeedCandidateStmt {
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
        T2: crate::BytesSql,
        T3: crate::BytesSql,
        T4: crate::StringSql,
        T5: crate::StringSql,
        T6: crate::ArraySql<Item = T5>,
        T7: crate::StringSql,
        T8: crate::StringSql,
        T9: crate::ArraySql<Item = T8>,
        T10: crate::StringSql,
        T11: crate::ArraySql<Item = T10>,
        T12: crate::StringSql,
        T13: crate::ArraySql<Item = T12>,
        T14: crate::JsonSql,
    >(
        &'s self,
        client: &'c C,
        user_id: &'a uuid::Uuid,
        public_code: &'a T1,
        full_name_enc: &'a T2,
        contacts_enc: &'a T3,
        city: &'a Option<T4>,
        relocation: &'a bool,
        work_formats: &'a T6,
        salary_expectation_min: &'a Option<i64>,
        about: &'a T7,
        experience_years: &'a Option<f32>,
        roles: &'a T9,
        soft_skills: &'a T11,
        industries: &'a T13,
        experience: &'a T14,
        search_status: &'a crate::types::SearchStatus,
        show_name: &'a bool,
        last_activity_at: &'a chrono::DateTime<chrono::Utc>,
        created_at: &'a chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
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
                    show_name,
                    last_activity_at,
                    created_at,
                ],
            )
            .await
    }
}
impl<
    'a,
    C: GenericClient + Send + Sync,
    T1: crate::StringSql,
    T2: crate::BytesSql,
    T3: crate::BytesSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::ArraySql<Item = T5>,
    T7: crate::StringSql,
    T8: crate::StringSql,
    T9: crate::ArraySql<Item = T8>,
    T10: crate::StringSql,
    T11: crate::ArraySql<Item = T10>,
    T12: crate::StringSql,
    T13: crate::ArraySql<Item = T12>,
    T14: crate::JsonSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SeedCandidateParams<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SeedCandidateStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SeedCandidateParams<
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
        >,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.user_id,
            &params.public_code,
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
            &params.search_status,
            &params.show_name,
            &params.last_activity_at,
            &params.created_at,
        ))
    }
}
pub struct SeedCompanyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn seed_company() -> SeedCompanyStmt {
    SeedCompanyStmt(
        "INSERT INTO companies (id, owner_user_id, name, description, industry, city, size, contact_email, contact_telegram) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        None,
    )
}
impl SeedCompanyStmt {
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
        T6: crate::StringSql,
        T7: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        owner_user_id: &'a uuid::Uuid,
        name: &'a T1,
        description: &'a T2,
        industry: &'a T3,
        city: &'a T4,
        size: &'a T5,
        contact_email: &'a T6,
        contact_telegram: &'a T7,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    owner_user_id,
                    name,
                    description,
                    industry,
                    city,
                    size,
                    contact_email,
                    contact_telegram,
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
    T6: crate::StringSql,
    T7: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SeedCompanyParams<T1, T2, T3, T4, T5, T6, T7>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SeedCompanyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SeedCompanyParams<T1, T2, T3, T4, T5, T6, T7>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.owner_user_id,
            &params.name,
            &params.description,
            &params.industry,
            &params.city,
            &params.size,
            &params.contact_email,
            &params.contact_telegram,
        ))
    }
}
pub struct SeedFspLinkStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn seed_fsp_link() -> SeedFspLinkStmt {
    SeedFspLinkStmt(
        "INSERT INTO fsp_links (candidate_id, fsp_id, linked_via, last_sync_at, achievements) VALUES ($1, $2, 'seed', now(), $3)",
        None,
    )
}
impl SeedFspLinkStmt {
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
        candidate_id: &'a uuid::Uuid,
        fsp_id: &'a T1,
        achievements: &'a T2,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(self.0, &[candidate_id, fsp_id, achievements])
            .await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql, T2: crate::JsonSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SeedFspLinkParams<T1, T2>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SeedFspLinkStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SeedFspLinkParams<T1, T2>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.candidate_id,
            &params.fsp_id,
            &params.achievements,
        ))
    }
}
pub struct SeedSessionStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn seed_session() -> SeedSessionStmt {
    SeedSessionStmt(
        "INSERT INTO assessment_sessions (id, candidate_id, specialization, claimed_grade, attempt_kind, status, state, theta, se, measured_grade, confident_above_next, person_fit_lz, aberrant, items_count, started_at, finished_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        None,
    )
}
impl SeedSessionStmt {
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
        T5: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        specialization: &'a T1,
        claimed_grade: &'a T2,
        attempt_kind: &'a T3,
        status: &'a crate::types::AssessmentStatus,
        state: &'a T4,
        theta: &'a f64,
        se: &'a f64,
        measured_grade: &'a Option<T5>,
        confident_above_next: &'a bool,
        person_fit_lz: &'a f64,
        aberrant: &'a bool,
        items_count: &'a i32,
        started_at: &'a chrono::DateTime<chrono::Utc>,
        finished_at: &'a chrono::DateTime<chrono::Utc>,
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
                    status,
                    state,
                    theta,
                    se,
                    measured_grade,
                    confident_above_next,
                    person_fit_lz,
                    aberrant,
                    items_count,
                    started_at,
                    finished_at,
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
    T5: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        SeedSessionParams<T1, T2, T3, T4, T5>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for SeedSessionStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a SeedSessionParams<T1, T2, T3, T4, T5>,
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
            &params.status,
            &params.state,
            &params.theta,
            &params.se,
            &params.measured_grade,
            &params.confident_above_next,
            &params.person_fit_lz,
            &params.aberrant,
            &params.items_count,
            &params.started_at,
            &params.finished_at,
        ))
    }
}
