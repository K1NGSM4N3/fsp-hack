// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct CandidateMarketParams<T1: crate::StringSql, T2: crate::StringSql> {
    pub candidate_id: uuid::Uuid,
    pub specialization: T1,
    pub grade: T2,
}
#[derive(Clone, Copy, Debug)]
pub struct RecordProfileViewParams {
    pub company_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CategoryOverview {
    pub specialization: String,
    pub grade: String,
    pub candidates: i64,
    pub with_fsp: i64,
    pub actively_looking: i64,
    pub median_expectation: Option<i64>,
}
pub struct CategoryOverviewBorrowed<'a> {
    pub specialization: &'a str,
    pub grade: &'a str,
    pub candidates: i64,
    pub with_fsp: i64,
    pub actively_looking: i64,
    pub median_expectation: Option<i64>,
}
impl<'a> From<CategoryOverviewBorrowed<'a>> for CategoryOverview {
    fn from(
        CategoryOverviewBorrowed {
            specialization,
            grade,
            candidates,
            with_fsp,
            actively_looking,
            median_expectation,
        }: CategoryOverviewBorrowed<'a>,
    ) -> Self {
        Self {
            specialization: specialization.into(),
            grade: grade.into(),
            candidates,
            with_fsp,
            actively_looking,
            median_expectation,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize)]
pub struct CandidateMarket {
    pub companies_viewed_you: i64,
    pub invitations_in_category: i64,
    pub open_needs: i64,
    pub peers: i64,
    pub median_salary_min: Option<i64>,
    pub median_salary_max: Option<i64>,
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CategoryOverviewQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CategoryOverviewBorrowed, tokio_postgres::Error>,
    mapper: fn(CategoryOverviewBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CategoryOverviewQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CategoryOverviewBorrowed) -> R,
    ) -> CategoryOverviewQuery<'c, 'a, 's, C, R, N> {
        CategoryOverviewQuery {
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
pub struct CandidateMarketQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CandidateMarket, tokio_postgres::Error>,
    mapper: fn(CandidateMarket) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateMarketQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateMarket) -> R,
    ) -> CandidateMarketQuery<'c, 'a, 's, C, R, N> {
        CandidateMarketQuery {
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
pub struct CategoryOverviewStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn category_overview() -> CategoryOverviewStmt {
    CategoryOverviewStmt(
        "SELECT c.specialization, c.grade, count(*) AS candidates, count(*) FILTER (WHERE f.has_achievements AND c.show_fsp) AS with_fsp, count(*) FILTER (WHERE c.search_status = 'active') AS actively_looking, (percentile_cont(0.5) WITHIN GROUP (ORDER BY c.salary_expectation_min))::bigint AS median_expectation FROM candidates c LEFT JOIN fsp_links f ON f.candidate_id = c.user_id WHERE c.visible AND c.search_status <> 'closed' AND c.specialization IS NOT NULL AND c.grade IS NOT NULL GROUP BY c.specialization, c.grade",
        None,
    )
}
impl CategoryOverviewStmt {
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
    ) -> CategoryOverviewQuery<'c, 'a, 's, C, CategoryOverview, 0> {
        CategoryOverviewQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CategoryOverviewBorrowed, tokio_postgres::Error> {
                Ok(CategoryOverviewBorrowed {
                    specialization: row.try_get(0)?,
                    grade: row.try_get(1)?,
                    candidates: row.try_get(2)?,
                    with_fsp: row.try_get(3)?,
                    actively_looking: row.try_get(4)?,
                    median_expectation: row.try_get(5)?,
                })
            },
            mapper: |it| CategoryOverview::from(it),
        }
    }
}
pub struct CandidateMarketStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_market() -> CandidateMarketStmt {
    CandidateMarketStmt(
        "SELECT (SELECT count(DISTINCT pv.company_id) FROM profile_views pv WHERE pv.candidate_id = $1 AND pv.viewed_on >= current_date - 30) AS companies_viewed_you, (SELECT count(*) FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id WHERE ic.specialization = $2 AND ic.grade = $3 AND i.created_at >= now() - interval '30 days') AS invitations_in_category, (SELECT count(*) FROM hiring_needs n WHERE n.status = 'active' AND n.specialization = $2 AND $3 = ANY (n.grades)) AS open_needs, (SELECT count(*) FROM candidates oc WHERE oc.specialization = $2 AND oc.grade = $3 AND oc.visible AND oc.search_status <> 'closed') AS peers, (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_min))::bigint FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id WHERE ic.specialization = $2 AND ic.grade = $3 AND i.created_at >= now() - interval '90 days') AS median_salary_min, (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_max))::bigint FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id WHERE ic.specialization = $2 AND ic.grade = $3 AND i.created_at >= now() - interval '90 days') AS median_salary_max",
        None,
    )
}
impl CandidateMarketStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>(
        &'s self,
        client: &'c C,
        candidate_id: &'a uuid::Uuid,
        specialization: &'a T1,
        grade: &'a T2,
    ) -> CandidateMarketQuery<'c, 'a, 's, C, CandidateMarket, 3> {
        CandidateMarketQuery {
            client,
            params: [candidate_id, specialization, grade],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<CandidateMarket, tokio_postgres::Error> {
                    Ok(CandidateMarket {
                        companies_viewed_you: row.try_get(0)?,
                        invitations_in_category: row.try_get(1)?,
                        open_needs: row.try_get(2)?,
                        peers: row.try_get(3)?,
                        median_salary_min: row.try_get(4)?,
                        median_salary_max: row.try_get(5)?,
                    })
                },
            mapper: |it| CandidateMarket::from(it),
        }
    }
}
impl<'c, 'a, 's, C: GenericClient, T1: crate::StringSql, T2: crate::StringSql>
    crate::client::async_::Params<
        'c,
        'a,
        's,
        CandidateMarketParams<T1, T2>,
        CandidateMarketQuery<'c, 'a, 's, C, CandidateMarket, 3>,
        C,
    > for CandidateMarketStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a CandidateMarketParams<T1, T2>,
    ) -> CandidateMarketQuery<'c, 'a, 's, C, CandidateMarket, 3> {
        self.bind(
            client,
            &params.candidate_id,
            &params.specialization,
            &params.grade,
        )
    }
}
pub struct RecordProfileViewStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn record_profile_view() -> RecordProfileViewStmt {
    RecordProfileViewStmt(
        "INSERT INTO profile_views (company_id, candidate_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        None,
    )
}
impl RecordProfileViewStmt {
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
        company_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[company_id, candidate_id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        RecordProfileViewParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for RecordProfileViewStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a RecordProfileViewParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.company_id, &params.candidate_id))
    }
}
