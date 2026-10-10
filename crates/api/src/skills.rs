use std::borrow::Cow;

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{error::ApiResult, pool};

pub fn is_known(slug: &str) -> bool {
    matching::lookup(slug).is_some()
}

pub fn display_name(slug: &str) -> &str {
    matching::lookup(slug).map_or(slug, |d| d.name)
}

/// Maps aliases ("постгрес", "k8s") to canonical slugs, keeping order and dropping duplicates.
/// Unknown entries are returned as the error so the caller can report them.
pub fn canonicalize(input: &[String]) -> Result<Vec<String>, Vec<String>> {
    let mut out: Vec<String> = Vec::with_capacity(input.len());
    let mut unknown = Vec::new();
    for raw in input {
        match matching::lookup(raw).or_else(|| matching::normalize(raw)) {
            Some(def) if !out.iter().any(|s| s == def.slug) => out.push(def.slug.to_owned()),
            Some(_) => {}
            None => unknown.push(raw.clone()),
        }
    }
    if unknown.is_empty() {
        Ok(out)
    } else {
        Err(unknown)
    }
}

pub fn unknown_message(unknown: &[String]) -> Cow<'static, str> {
    Cow::Owned(format!("Неизвестные навыки: {}", unknown.join(", ")))
}

/// Rewrites the candidate's `fsp` skills to what the stored FSP snapshot confirms
/// (`matching::confirmed_skills`, docs/architecture.md section 7.4). Runs in the transaction that writes the
/// snapshot, so the two never disagree; `None` — no link, no such skills. Rows of other sources
/// are separate and untouched: a skill the test confirmed stays confirmed by the test.
pub async fn replace_fsp_skills(
    tx: &mut PgConnection,
    candidate: Uuid,
    snapshot: Option<&serde_json::Value>,
) -> Result<(), sqlx::Error> {
    db::candidates::delete_candidate_skills_by_source(&mut *tx, candidate, "fsp").await?;
    for slug in matching::confirmed_skills(&pool::achievements(snapshot)) {
        db::candidates::insert_candidate_skill(&mut *tx, candidate, slug, "fsp").await?;
    }
    Ok(())
}

/// Brings every candidate's `fsp` skills in line with their snapshot: links written before
/// the skills were derived, or after the discipline mapping changed. Returns how many changed.
pub async fn backfill_fsp_skills(pg: &PgPool) -> ApiResult<usize> {
    let mut changed = 0;
    for row in db::fsp::fsp_skill_states(pg).await? {
        let want = matching::confirmed_skills(&pool::achievements(row.achievements.as_ref()));
        if want
            .iter()
            .copied()
            .eq(row.fsp_skills.iter().map(String::as_str))
        {
            continue;
        }
        let mut tx = pg.begin().await?;
        replace_fsp_skills(&mut tx, row.candidate_id, row.achievements.as_ref()).await?;
        tx.commit().await?;
        changed += 1;
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_aliases() {
        let input = vec![
            "постгрес".to_owned(),
            "postgresql".to_owned(),
            "k8s".to_owned(),
        ];
        assert_eq!(
            canonicalize(&input).unwrap(),
            vec!["postgresql", "kubernetes"]
        );
        assert_eq!(
            canonicalize(&["нечто".to_owned()]).unwrap_err(),
            vec!["нечто"]
        );
    }
}
