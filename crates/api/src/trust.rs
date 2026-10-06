//! Anti-fake signals wired to the database (X-1): the corporate domain, salary flags on publish
//! and invite, near-duplicate vacancies, and the trust level that sets the invitation quota.
//! The rules themselves live in `matching::trust`.

use catalog::Category;
use chrono::Utc;
use matching::trust::{
    DomainCheck, MarketBand, Signature, TrustAssessment, TrustFlag, TrustSignals, assess,
    check_domain, salary_flags,
    similarity::{closest, vacancy_text},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{crypto::Crypto, error::ApiResult};

/// Recent published vacancies a new one is compared with; a day of a large job board.
const RECENT_LIMIT: i64 = 2000;
const BACKFILL_BATCH: i64 = 500;

fn count(v: i64) -> u16 {
    v.clamp(0, i64::from(u16::MAX)) as u16
}

pub fn signals(row: &db::trust::CompanySignals) -> TrustSignals {
    TrustSignals {
        created_unix: row.owner_created_at.timestamp(),
        domain_verified: row.verified_domain.is_some(),
        complaints: count(row.complaints),
        declines: count(row.declines),
        bad_declines: count(row.bad_declines),
        invitations: count(row.invitations),
        withdrawn: count(row.withdrawn),
        closed: count(row.closed),
        answered: count(row.answered),
        accepted: count(row.accepted),
        salary_flags: count(row.salary_flags),
        own_duplicates: count(row.own_duplicates),
        copies: count(row.copies),
        frozen_until: row.frozen_until.map(|t| t.timestamp()),
    }
}

pub async fn company_assessment(
    pool: &PgPool,
    company: Uuid,
) -> ApiResult<(TrustAssessment, db::trust::CompanySignals)> {
    let row = db::trust::company_signals(pool, Some(company))
        .await?
        .into_iter()
        .next()
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok((assess(&signals(&row), Utc::now().timestamp()), row))
}

/// Re-checks the owner's login email against the website and stores the verified domain.
pub async fn refresh_domain(
    pool: &PgPool,
    crypto: &Crypto,
    owner_user_id: Uuid,
) -> ApiResult<Option<DomainCheck>> {
    let Some(owner) = db::trust::owner_identity(pool, owner_user_id).await? else {
        return Ok(None);
    };
    let email = crypto.decrypt_str(&owner.email_enc)?;
    let check = check_domain(
        &email,
        owner.email_verified_at.is_some(),
        owner.website.as_deref(),
    );
    if owner.verified_domain.as_deref() != check.verified_domain() {
        db::trust::set_verified_domain(pool, owner.company_id, check.verified_domain()).await?;
    }
    Ok(Some(check))
}

pub async fn market_band(pool: &PgPool, category: Category) -> ApiResult<Option<MarketBand>> {
    let band =
        db::trust::expectation_band(pool, category.specialization.slug(), category.grade.slug())
            .await?;
    Ok(match (band.median, band.p90) {
        (Some(median), Some(p90)) => Some(MarketBand {
            samples: u32::try_from(band.samples).unwrap_or(u32::MAX),
            median,
            p90,
        }),
        _ => None,
    })
}

pub async fn salary_check(
    pool: &PgPool,
    category: Category,
    min: i64,
    max: i64,
) -> ApiResult<(Option<MarketBand>, Vec<TrustFlag>)> {
    let band = market_band(pool, category).await?;
    Ok((band, salary_flags(category, min, max, band)))
}

/// Stores the MinHash signature of the vacancy text (or marks it too short to compare).
pub async fn sign_vacancy(
    pool: &PgPool,
    id: Uuid,
    title: &str,
    description: &str,
) -> ApiResult<Option<Signature>> {
    let signature = Signature::of_text(&vacancy_text(title, description));
    let bytes = signature.as_ref().map(Signature::to_bytes);
    db::trust::set_vacancy_signature(pool, id, bytes.as_deref()).await?;
    Ok(signature)
}

/// Signatures of rows with the metadata kept apart: the scan touches only the compact array.
fn split(rows: Vec<db::trust::SignedVacancy>) -> (Vec<Signature>, Vec<db::trust::SignedVacancy>) {
    rows.into_iter()
        .filter_map(|r| Signature::from_bytes(&r.text_signature).map(|s| (s, r)))
        .unzip()
}

/// Flags written when a vacancy is (re)published: salary against the category market, a near-copy
/// of the company's own published vacancy, a near-copy of another company's one.
pub async fn flag_published_vacancy(
    pool: &PgPool,
    company: Uuid,
    id: Uuid,
    category: Category,
    salary: (i64, i64),
    signature: Option<&Signature>,
) -> ApiResult<Vec<TrustFlag>> {
    let (_, mut flags) = salary_check(pool, category, salary.0, salary.1).await?;
    if let Some(sig) = signature {
        let (sigs, meta) =
            split(db::trust::recent_published_signatures(pool, id, RECENT_LIMIT).await?);
        let (own, other): (Vec<usize>, Vec<usize>) =
            (0..meta.len()).partition(|&i| meta[i].company_id == company);
        if let Some((i, similarity)) = closest(sig, own.iter().map(|&i| &sigs[i])) {
            let m = &meta[own[i]];
            flags.push(TrustFlag::DuplicateOwn {
                vacancy_id: m.id.to_string(),
                title: m.title.clone(),
                similarity,
            });
        }
        if let Some((i, similarity)) = closest(sig, other.iter().map(|&i| &sigs[i])) {
            let m = &meta[other[i]];
            flags.push(TrustFlag::CopyOfOther {
                vacancy_id: m.id.to_string(),
                company: m.company_name.clone(),
                title: m.title.clone(),
                similarity,
            });
        }
    }
    db::trust::set_vacancy_flags(pool, id, &serde_json::to_value(&flags)?).await?;
    Ok(flags)
}

/// Salary flags of an invitation against the invited candidate's category.
pub async fn flag_invitation(
    pool: &PgPool,
    id: Uuid,
    category: Category,
    salary: (i64, i64),
) -> ApiResult<Vec<TrustFlag>> {
    let (_, flags) = salary_check(pool, category, salary.0, salary.1).await?;
    if !flags.is_empty() {
        db::trust::set_invitation_flags(pool, id, &serde_json::to_value(&flags)?).await?;
    }
    Ok(flags)
}

#[derive(Debug, Clone)]
pub struct SimilarOwn {
    pub id: Uuid,
    pub title: String,
    pub status: db::types::VacancyStatus,
    pub similarity: f32,
}

/// The company's own drafts and published vacancies whose text is nearly the same, best first.
pub async fn similar_own(
    pool: &PgPool,
    company: Uuid,
    exclude: Option<Uuid>,
    title: &str,
    description: &str,
) -> ApiResult<Vec<SimilarOwn>> {
    let Some(sig) = Signature::of_text(&vacancy_text(title, description)) else {
        return Ok(Vec::new());
    };
    let (sigs, meta) = split(db::trust::company_signatures(pool, company, exclude).await?);
    let mut out: Vec<SimilarOwn> = sigs
        .iter()
        .zip(meta)
        .map(|(s, m)| (sig.similarity(s), m))
        .filter(|(s, _)| *s >= matching::trust::DUPLICATE_THRESHOLD)
        .map(|(similarity, m)| SimilarOwn {
            id: m.id,
            title: m.title,
            status: m.status,
            similarity,
        })
        .collect();
    out.sort_by(|a, b| b.similarity.total_cmp(&a.similarity));
    Ok(out)
}

/// Signs vacancies stored before signatures existed (and the demo seed). Idempotent.
pub async fn backfill_signatures(pool: &PgPool) -> ApiResult<usize> {
    let mut total = 0;
    loop {
        let batch = db::trust::unsigned_vacancies(pool, BACKFILL_BATCH).await?;
        if batch.is_empty() {
            return Ok(total);
        }
        for v in &batch {
            sign_vacancy(pool, v.id, &v.title, &v.description).await?;
        }
        total += batch.len();
    }
}
