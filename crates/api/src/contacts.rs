//! Contact disclosure (V-2, E-7). `CandidateContacts` can only be built from a `ContactGrant`,
//! and a grant can only be obtained through `ContactGrant::obtain`, which checks the legal basis
//! (accepted invitation or the candidate's own application) and writes the audit log.
//! Employer-facing DTOs simply have no contact fields.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{crypto::Crypto, error::ApiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContactBasis {
    Invitation,
    Application,
}

pub struct ContactGrant {
    candidate_id: Uuid,
    basis: ContactBasis,
    basis_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct Contacts {
    pub phone: Option<String>,
    pub telegram: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CandidateContacts {
    pub candidate_id: Uuid,
    pub full_name: Option<String>,
    #[serde(flatten)]
    pub contacts: Contacts,
    pub basis: ContactBasis,
    pub basis_id: Uuid,
}

pub struct AccessContext<'a> {
    pub employer_user_id: Uuid,
    pub company_id: Uuid,
    pub ip: Option<&'a str>,
    pub user_agent: Option<&'a str>,
}

impl ContactGrant {
    pub async fn obtain(
        client: &deadpool_postgres::Object,
        ctx: &AccessContext<'_>,
        candidate_id: Uuid,
    ) -> ApiResult<Option<ContactGrant>> {
        let invitation = db::queries::contacts::accepted_invitation_basis()
            .bind(client, &ctx.company_id, &candidate_id)
            .opt()
            .await?;
        let (basis, basis_id) = match invitation {
            Some(id) => (ContactBasis::Invitation, id),
            None => match db::queries::contacts::application_basis()
                .bind(client, &ctx.company_id, &candidate_id)
                .opt()
                .await?
            {
                Some(id) => (ContactBasis::Application, id),
                None => return Ok(None),
            },
        };
        let basis_slug = match basis {
            ContactBasis::Invitation => "invitation",
            ContactBasis::Application => "application",
        };
        db::queries::contacts::insert_contact_access()
            .bind(
                client,
                &Uuid::now_v7(),
                &ctx.employer_user_id,
                &ctx.company_id,
                &candidate_id,
                &basis_slug,
                &basis_id,
                &ctx.ip,
                &ctx.user_agent,
            )
            .await?;
        Ok(Some(ContactGrant {
            candidate_id,
            basis,
            basis_id,
        }))
    }
}

impl CandidateContacts {
    pub async fn reveal(
        client: &deadpool_postgres::Object,
        crypto: &Crypto,
        grant: ContactGrant,
    ) -> ApiResult<CandidateContacts> {
        let row = db::queries::contacts::candidate_secrets()
            .bind(client, &grant.candidate_id)
            .one()
            .await?;
        Ok(CandidateContacts {
            candidate_id: grant.candidate_id,
            full_name: row
                .full_name_enc
                .as_deref()
                .map(|b| crypto.decrypt_str(b))
                .transpose()?,
            contacts: row
                .contacts_enc
                .as_deref()
                .map(|b| crypto.decrypt_json(b))
                .transpose()?
                .unwrap_or_default(),
            basis: grant.basis,
            basis_id: grant.basis_id,
        })
    }
}
