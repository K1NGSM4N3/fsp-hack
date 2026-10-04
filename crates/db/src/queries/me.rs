// This file was generated with `clorinde`. Do not modify.

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateOnboarding {
    pub public_code: String,
    pub has_profile: bool,
    pub has_survey: bool,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub active_session_id: Option<uuid::Uuid>,
}
pub struct CandidateOnboardingBorrowed<'a> {
    pub public_code: &'a str,
    pub has_profile: bool,
    pub has_survey: bool,
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub active_session_id: Option<uuid::Uuid>,
}
impl<'a> From<CandidateOnboardingBorrowed<'a>> for CandidateOnboarding {
    fn from(
        CandidateOnboardingBorrowed {
            public_code,
            has_profile,
            has_survey,
            specialization,
            grade,
            active_session_id,
        }: CandidateOnboardingBorrowed<'a>,
    ) -> Self {
        Self {
            public_code: public_code.into(),
            has_profile,
            has_survey,
            specialization: specialization.map(|v| v.into()),
            grade: grade.map(|v| v.into()),
            active_session_id,
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyByOwner {
    pub id: uuid::Uuid,
    pub name: String,
}
pub struct CompanyByOwnerBorrowed<'a> {
    pub id: uuid::Uuid,
    pub name: &'a str,
}
impl<'a> From<CompanyByOwnerBorrowed<'a>> for CompanyByOwner {
    fn from(CompanyByOwnerBorrowed { id, name }: CompanyByOwnerBorrowed<'a>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CandidateOnboardingQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<CandidateOnboardingBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateOnboardingBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateOnboardingQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateOnboardingBorrowed) -> R,
    ) -> CandidateOnboardingQuery<'c, 'a, 's, C, R, N> {
        CandidateOnboardingQuery {
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
pub struct CompanyByOwnerQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyByOwnerBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyByOwnerBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyByOwnerQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyByOwnerBorrowed) -> R,
    ) -> CompanyByOwnerQuery<'c, 'a, 's, C, R, N> {
        CompanyByOwnerQuery {
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
pub struct CandidateOnboardingStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_onboarding() -> CandidateOnboardingStmt {
    CandidateOnboardingStmt(
        "SELECT c.public_code, c.full_name_enc IS NOT NULL                                            AS has_profile, EXISTS (SELECT 1 FROM survey_responses s WHERE s.candidate_id = c.user_id) AS has_survey, c.specialization, c.grade, (SELECT a.id FROM assessment_sessions a WHERE a.candidate_id = c.user_id AND a.status = 'in_progress')      AS active_session_id FROM candidates c WHERE c.user_id = $1",
        None,
    )
}
impl CandidateOnboardingStmt {
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
    ) -> CandidateOnboardingQuery<'c, 'a, 's, C, CandidateOnboarding, 1> {
        CandidateOnboardingQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateOnboardingBorrowed, tokio_postgres::Error> {
                Ok(CandidateOnboardingBorrowed {
                    public_code: row.try_get(0)?,
                    has_profile: row.try_get(1)?,
                    has_survey: row.try_get(2)?,
                    specialization: row.try_get(3)?,
                    grade: row.try_get(4)?,
                    active_session_id: row.try_get(5)?,
                })
            },
            mapper: |it| CandidateOnboarding::from(it),
        }
    }
}
pub struct CompanyByOwnerStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_by_owner() -> CompanyByOwnerStmt {
    CompanyByOwnerStmt(
        "SELECT id, name FROM companies WHERE owner_user_id = $1",
        None,
    )
}
impl CompanyByOwnerStmt {
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
        owner_user_id: &'a uuid::Uuid,
    ) -> CompanyByOwnerQuery<'c, 'a, 's, C, CompanyByOwner, 1> {
        CompanyByOwnerQuery {
            client,
            params: [owner_user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CompanyByOwnerBorrowed, tokio_postgres::Error> {
                Ok(CompanyByOwnerBorrowed {
                    id: row.try_get(0)?,
                    name: row.try_get(1)?,
                })
            },
            mapper: |it| CompanyByOwner::from(it),
        }
    }
}
pub struct DeleteCandidateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_candidate() -> DeleteCandidateStmt {
    DeleteCandidateStmt("DELETE FROM candidates WHERE user_id = $1", None)
}
impl DeleteCandidateStmt {
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
pub struct DeleteCompanyOfStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn delete_company_of() -> DeleteCompanyOfStmt {
    DeleteCompanyOfStmt("DELETE FROM companies WHERE owner_user_id = $1", None)
}
impl DeleteCompanyOfStmt {
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
        owner_user_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[owner_user_id]).await
    }
}
pub struct HideCandidateStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn hide_candidate() -> HideCandidateStmt {
    HideCandidateStmt(
        "UPDATE candidates SET visible = false, updated_at = now() WHERE user_id = $1",
        None,
    )
}
impl HideCandidateStmt {
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
