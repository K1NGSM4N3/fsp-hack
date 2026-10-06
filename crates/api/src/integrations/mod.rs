//! Integration with employers' ATS (architecture.md §11, docs/solution.md §10): a transactional
//! outbox of integration events delivered as signed webhooks, company API keys for the inbound
//! API, and the demo provisioning that connects the demo company to the ATS mock.

pub mod dispatcher;
pub mod events;
pub mod keys;
pub mod outcomes;
pub mod signature;
pub mod webhook;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::DemoIntegration,
    crypto::{Crypto, CryptoError, normalize_email},
    error::ApiResult,
};
use keys::{KEY_PREFIX, Scope, Scopes};
use webhook::WebhookSender;

fn secret_context(company_id: Uuid) -> Vec<u8> {
    let mut ctx = b"webhook_secret:".to_vec();
    ctx.extend_from_slice(company_id.as_bytes());
    ctx
}

pub fn seal_secret(
    crypto: &Crypto,
    company_id: Uuid,
    secret: &str,
) -> Result<Vec<u8>, CryptoError> {
    crypto
        .fields
        .encrypt_bound(secret.as_bytes(), &secret_context(company_id))
}

pub fn open_secret(
    crypto: &Crypto,
    company_id: Uuid,
    sealed: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    crypto
        .fields
        .decrypt_bound(sealed, &secret_context(company_id))
}

/// Idempotent: the key is inserted by hash, the webhook only if the company has none, so a
/// restart changes nothing and a webhook reconfigured in the cabinet is left alone.
pub async fn provision_demo(
    pool: &PgPool,
    crypto: &Crypto,
    sender: &WebhookSender,
    demo: &DemoIntegration,
) -> ApiResult<()> {
    let bidx = crypto.blind_index(&normalize_email(&demo.owner_email));
    let Some(company) = db::integrations::company_by_owner_bidx(pool, &bidx).await? else {
        tracing::warn!(owner = %demo.owner_email, "demo ATS: owner company not found, skipped");
        return Ok(());
    };
    if !demo.api_key.starts_with(KEY_PREFIX) {
        tracing::warn!("ATS_DEMO_API_KEY must start with {KEY_PREFIX}; skipped");
        return Ok(());
    }
    let key = keys::from_secret(demo.api_key.clone());
    let added = db::integrations::insert_api_key(
        pool,
        &db::integrations::NewApiKey {
            id: Uuid::now_v7(),
            company_id: company,
            name: "Демо-ATS (ключ из .env)",
            prefix: &key.prefix,
            key_hash: &key.hash,
            scopes: &Scopes::of(&Scope::ALL).to_strings(),
            created_by: None,
        },
    )
    .await?;
    let mut webhook = false;
    if let (Some(url), Some(secret)) = (&demo.webhook_url, &demo.webhook_secret) {
        match sender.check_url(url) {
            Ok(url) => {
                webhook = db::integrations::insert_webhook(
                    pool,
                    company,
                    url.as_str(),
                    &seal_secret(crypto, company, secret)?,
                    &signature::secret_hint(secret),
                )
                .await?
                    > 0;
            }
            Err(e) => {
                tracing::warn!(error = %e, "ATS_DEMO_WEBHOOK_URL rejected; add its host to WEBHOOK_DEV_HOSTS")
            }
        }
    }
    tracing::info!(
        key_added = added > 0,
        webhook_added = webhook,
        "demo ATS integration ready"
    );
    Ok(())
}
