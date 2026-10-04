// This file was generated with `clorinde`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct AcceptedInvitationBasisParams {
    pub company_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct ApplicationBasisParams {
    pub company_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
}
#[derive(Debug)]
pub struct InsertContactAccessParams<
    T1: crate::StringSql,
    T2: crate::StringSql,
    T3: crate::StringSql,
> {
    pub id: uuid::Uuid,
    pub employer_user_id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    pub candidate_id: uuid::Uuid,
    pub basis: T1,
    pub basis_id: uuid::Uuid,
    pub ip: Option<T2>,
    pub user_agent: Option<T3>,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CandidateSecrets {
    pub full_name_enc: Option<Vec<u8>>,
    pub contacts_enc: Option<Vec<u8>>,
}
pub struct CandidateSecretsBorrowed<'a> {
    pub full_name_enc: Option<&'a [u8]>,
    pub contacts_enc: Option<&'a [u8]>,
}
impl<'a> From<CandidateSecretsBorrowed<'a>> for CandidateSecrets {
    fn from(
        CandidateSecretsBorrowed {
            full_name_enc,
            contacts_enc,
        }: CandidateSecretsBorrowed<'a>,
    ) -> Self {
        Self {
            full_name_enc: full_name_enc.map(|v| v.into()),
            contacts_enc: contacts_enc.map(|v| v.into()),
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
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
pub struct CandidateSecretsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<CandidateSecretsBorrowed, tokio_postgres::Error>,
    mapper: fn(CandidateSecretsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> CandidateSecretsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(CandidateSecretsBorrowed) -> R,
    ) -> CandidateSecretsQuery<'c, 'a, 's, C, R, N> {
        CandidateSecretsQuery {
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
pub struct AcceptedInvitationBasisStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn accepted_invitation_basis() -> AcceptedInvitationBasisStmt {
    AcceptedInvitationBasisStmt(
        "SELECT id FROM invitations WHERE company_id = $1 AND candidate_id = $2 AND status = 'accepted' ORDER BY responded_at DESC LIMIT 1",
        None,
    )
}
impl AcceptedInvitationBasisStmt {
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
        candidate_id: &'a uuid::Uuid,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2> {
        UuidUuidQuery {
            client,
            params: [company_id, candidate_id],
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
        AcceptedInvitationBasisParams,
        UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2>,
        C,
    > for AcceptedInvitationBasisStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a AcceptedInvitationBasisParams,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2> {
        self.bind(client, &params.company_id, &params.candidate_id)
    }
}
pub struct ApplicationBasisStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn application_basis() -> ApplicationBasisStmt {
    ApplicationBasisStmt(
        "SELECT a.id FROM applications a JOIN vacancies v ON v.id = a.vacancy_id WHERE v.company_id = $1 AND a.candidate_id = $2 AND a.status <> 'withdrawn' ORDER BY a.created_at DESC LIMIT 1",
        None,
    )
}
impl ApplicationBasisStmt {
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
        candidate_id: &'a uuid::Uuid,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2> {
        UuidUuidQuery {
            client,
            params: [company_id, candidate_id],
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
        ApplicationBasisParams,
        UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2>,
        C,
    > for ApplicationBasisStmt
{
    fn params(
        &'s self,
        client: &'c C,
        params: &'a ApplicationBasisParams,
    ) -> UuidUuidQuery<'c, 'a, 's, C, uuid::Uuid, 2> {
        self.bind(client, &params.company_id, &params.candidate_id)
    }
}
pub struct CandidateSecretsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn candidate_secrets() -> CandidateSecretsStmt {
    CandidateSecretsStmt(
        "SELECT full_name_enc, contacts_enc FROM candidates WHERE user_id = $1",
        None,
    )
}
impl CandidateSecretsStmt {
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
    ) -> CandidateSecretsQuery<'c, 'a, 's, C, CandidateSecrets, 1> {
        CandidateSecretsQuery {
            client,
            params: [user_id],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<CandidateSecretsBorrowed, tokio_postgres::Error> {
                Ok(CandidateSecretsBorrowed {
                    full_name_enc: row.try_get(0)?,
                    contacts_enc: row.try_get(1)?,
                })
            },
            mapper: |it| CandidateSecrets::from(it),
        }
    }
}
pub struct InsertContactAccessStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn insert_contact_access() -> InsertContactAccessStmt {
    InsertContactAccessStmt(
        "INSERT INTO contact_access_log (id, employer_user_id, company_id, candidate_id, basis, basis_id, ip, user_agent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        None,
    )
}
impl InsertContactAccessStmt {
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
    >(
        &'s self,
        client: &'c C,
        id: &'a uuid::Uuid,
        employer_user_id: &'a uuid::Uuid,
        company_id: &'a uuid::Uuid,
        candidate_id: &'a uuid::Uuid,
        basis: &'a T1,
        basis_id: &'a uuid::Uuid,
        ip: &'a Option<T2>,
        user_agent: &'a Option<T3>,
    ) -> Result<u64, tokio_postgres::Error> {
        client
            .execute(
                self.0,
                &[
                    id,
                    employer_user_id,
                    company_id,
                    candidate_id,
                    basis,
                    basis_id,
                    ip,
                    user_agent,
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
>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        InsertContactAccessParams<T1, T2, T3>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for InsertContactAccessStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a InsertContactAccessParams<T1, T2, T3>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(
            client,
            &params.id,
            &params.employer_user_id,
            &params.company_id,
            &params.candidate_id,
            &params.basis,
            &params.basis_id,
            &params.ip,
            &params.user_agent,
        ))
    }
}
