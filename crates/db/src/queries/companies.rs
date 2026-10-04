// This file was generated with `clorinde`. Do not modify.

#[derive(Debug)]
pub struct InsertCompanyParams<T1: crate::StringSql> {
    pub id: uuid::Uuid,
    pub owner_user_id: uuid::Uuid,
    pub name: T1,
}
#[derive(Debug)]
pub struct UpdateCompanyParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
    T4: crate::StringSql,
    T5: crate::StringSql,
    T6: crate::StringSql,
    T7: crate::StringSql,
    T8: crate::StringSql,
    T9: crate::StringSql,
> {
    pub name: T1,
    pub description: T2,
    pub industry: Option<T3>,
    pub website: Option<T4>,
    pub city: Option<T5>,
    pub size: Option<T6>,
    pub contact_email: Option<T7>,
    pub contact_phone: Option<T8>,
    pub contact_telegram: Option<T9>,
    pub owner_user_id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CompanyOfOwner {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub city: Option<String>,
    pub size: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_telegram: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
pub struct CompanyOfOwnerBorrowed<'a> {
    pub id: uuid::Uuid,
    pub name: &'a str,
    pub description: &'a str,
    pub industry: Option<&'a str>,
    pub website: Option<&'a str>,
    pub city: Option<&'a str>,
    pub size: Option<&'a str>,
    pub contact_email: Option<&'a str>,
    pub contact_phone: Option<&'a str>,
    pub contact_telegram: Option<&'a str>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
impl<'a> From<CompanyOfOwnerBorrowed<'a>> for CompanyOfOwner {
    fn from(
        CompanyOfOwnerBorrowed {
            id,
            name,
            description,
            industry,
            website,
            city,
            size,
            contact_email,
            contact_phone,
            contact_telegram,
            updated_at,
        }: CompanyOfOwnerBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            industry: industry.map(|v| v.into()),
            website: website.map(|v| v.into()),
            city: city.map(|v| v.into()),
            size: size.map(|v| v.into()),
            contact_email: contact_email.map(|v| v.into()),
            contact_phone: contact_phone.map(|v| v.into()),
            contact_telegram: contact_telegram.map(|v| v.into()),
            updated_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct CompanyOfOwnerQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CompanyOfOwnerBorrowed, tokio_postgres::Error>,
    mapper: fn(CompanyOfOwnerBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CompanyOfOwnerQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CompanyOfOwnerBorrowed) -> R,
    ) -> CompanyOfOwnerQuery<'c, 'a, 's, C, R, N> {
        CompanyOfOwnerQuery {
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
pub struct InsertCompanyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_company() -> InsertCompanyStmt {
    InsertCompanyStmt(
        "INSERT INTO companies (id, owner_user_id, name) VALUES ($1, $2, $3)",
        None,
    )
}
impl InsertCompanyStmt {
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
        owner_user_id: &'a uuid::Uuid,
        name: &'a T1,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[id, owner_user_id, name]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertCompanyParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertCompanyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertCompanyParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.id, &params.owner_user_id, &params.name))
    }
}
pub struct CompanyOfOwnerStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn company_of_owner() -> CompanyOfOwnerStmt {
    CompanyOfOwnerStmt(
        "SELECT id, name, description, industry, website, city, size, contact_email, contact_phone, contact_telegram, updated_at FROM companies WHERE owner_user_id = $1",
        None,
    )
}
impl CompanyOfOwnerStmt {
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
    ) -> CompanyOfOwnerQuery<'c, 'a, 's, C, CompanyOfOwner, 1> {
        CompanyOfOwnerQuery {
            client,
            params: [owner_user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CompanyOfOwnerBorrowed, tokio_postgres::Error> {
                Ok(CompanyOfOwnerBorrowed {
                    id: row.try_get(0)?,
                    name: row.try_get(1)?,
                    description: row.try_get(2)?,
                    industry: row.try_get(3)?,
                    website: row.try_get(4)?,
                    city: row.try_get(5)?,
                    size: row.try_get(6)?,
                    contact_email: row.try_get(7)?,
                    contact_phone: row.try_get(8)?,
                    contact_telegram: row.try_get(9)?,
                    updated_at: row.try_get(10)?,
                })
            },
            mapper: |it| CompanyOfOwner::from(it),
        }
    }
}
pub struct UpdateCompanyStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn update_company() -> UpdateCompanyStmt {
    UpdateCompanyStmt(
        "UPDATE companies SET name = $1, description = $2, industry = $3, website = $4, city = $5, size = $6, contact_email = $7, contact_phone = $8, contact_telegram = $9, updated_at = now() WHERE owner_user_id = $10",
        None,
    )
}
impl UpdateCompanyStmt {
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
        T8: crate::StringSql,
        T9: crate::StringSql,
    >(
        &'s self,
        client: &'c C,
        name: &'a T1,
        description: &'a T2,
        industry: &'a Option<T3>,
        website: &'a Option<T4>,
        city: &'a Option<T5>,
        size: &'a Option<T6>,
        contact_email: &'a Option<T7>,
        contact_phone: &'a Option<T8>,
        contact_telegram: &'a Option<T9>,
        owner_user_id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    name,
                    description,
                    industry,
                    website,
                    city,
                    size,
                    contact_email,
                    contact_phone,
                    contact_telegram,
                    owner_user_id,
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
    T8: crate::StringSql,
    T9: crate::StringSql,
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        UpdateCompanyParams<T1, T2, T3, T4, T5, T6, T7, T8, T9>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for UpdateCompanyStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a UpdateCompanyParams<T1, T2, T3, T4, T5, T6, T7, T8, T9>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.name,
            &params.description,
            &params.industry,
            &params.website,
            &params.city,
            &params.size,
            &params.contact_email,
            &params.contact_phone,
            &params.contact_telegram,
            &params.owner_user_id,
        ))
    }
}
