//! Contact disclosure (V-2, E-7). `CandidateContacts` can only be built from a `ContactGrant`,
//! and a grant can only be obtained through `ContactGrant::obtain`, which checks the legal basis
//! (accepted invitation or the candidate's own application) and writes the audit log.
//! Employer-facing DTOs simply have no contact fields.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgExecutor};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{crypto::Crypto, error::ApiResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContactBasis {
    Invitation,
    Application,
}

impl ContactBasis {
    pub const fn slug(self) -> &'static str {
        match self {
            ContactBasis::Invitation => "invitation",
            ContactBasis::Application => "application",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s {
            "invitation" => Some(ContactBasis::Invitation),
            "application" => Some(ContactBasis::Application),
            _ => None,
        }
    }
}

/// How the contacts left the platform: the employer's cabinet, the company's ATS over the API,
/// or a webhook payload sent to the ATS right after acceptance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessChannel {
    Cabinet,
    Api,
    Webhook,
}

impl AccessChannel {
    pub const fn slug(self) -> &'static str {
        match self {
            AccessChannel::Cabinet => "cabinet",
            AccessChannel::Api => "api",
            AccessChannel::Webhook => "webhook",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s {
            "cabinet" => Some(AccessChannel::Cabinet),
            "api" => Some(AccessChannel::Api),
            "webhook" => Some(AccessChannel::Webhook),
            _ => None,
        }
    }
}

/// One line of the candidate's «Кто открывал ваши контакты»: the company and the legal basis.
/// The employee, IP and browser stay in the audit log and are not part of this view.
#[derive(Serialize, ToSchema)]
pub struct ContactAccessEntry {
    pub company_id: Uuid,
    /// `None` when the company has since deleted its account.
    pub company_name: Option<String>,
    pub basis: ContactBasis,
    pub channel: AccessChannel,
    pub accessed_at: DateTime<Utc>,
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
    pub channel: AccessChannel,
    pub api_key_id: Option<Uuid>,
}

impl ContactGrant {
    /// Takes a connection so the check, the log entry and whatever the grant is used for (an
    /// outbox event) commit together.
    pub async fn obtain(
        conn: &mut PgConnection,
        ctx: &AccessContext<'_>,
        candidate_id: Uuid,
    ) -> ApiResult<Option<ContactGrant>> {
        let invitation =
            db::contacts::accepted_invitation_basis(&mut *conn, ctx.company_id, candidate_id)
                .await?;
        let (basis, basis_id) = match invitation {
            Some(id) => (ContactBasis::Invitation, id),
            None => match db::contacts::application_basis(&mut *conn, ctx.company_id, candidate_id)
                .await?
            {
                Some(id) => (ContactBasis::Application, id),
                None => return Ok(None),
            },
        };
        db::contacts::insert_contact_access(
            &mut *conn,
            &db::contacts::ContactAccess {
                id: Uuid::now_v7(),
                employer_user_id: ctx.employer_user_id,
                company_id: ctx.company_id,
                candidate_id,
                basis: basis.slug(),
                basis_id,
                ip: ctx.ip,
                user_agent: ctx.user_agent,
                channel: ctx.channel.slug(),
                api_key_id: ctx.api_key_id,
            },
        )
        .await?;
        Ok(Some(ContactGrant {
            candidate_id,
            basis,
            basis_id,
        }))
    }

    pub fn candidate_id(&self) -> Uuid {
        self.candidate_id
    }

    pub fn basis(&self) -> ContactBasis {
        self.basis
    }

    pub fn basis_id(&self) -> Uuid {
        self.basis_id
    }
}

impl CandidateContacts {
    pub async fn reveal(
        db: impl PgExecutor<'_>,
        crypto: &Crypto,
        grant: ContactGrant,
    ) -> ApiResult<CandidateContacts> {
        let row = db::contacts::candidate_secrets(db, grant.candidate_id).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basis_slug_round_trips() {
        for b in [ContactBasis::Invitation, ContactBasis::Application] {
            assert_eq!(ContactBasis::from_slug(b.slug()), Some(b));
        }
        assert_eq!(ContactBasis::from_slug("profile_view"), None);
        for c in [
            AccessChannel::Cabinet,
            AccessChannel::Api,
            AccessChannel::Webhook,
        ] {
            assert_eq!(AccessChannel::from_slug(c.slug()), Some(c));
        }
    }
}
