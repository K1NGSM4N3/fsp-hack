//! Embeds the queue of migration 0017. Every replica runs one worker; `claim_jobs` leases rows
//! with `FOR UPDATE SKIP LOCKED`, so replicas share the queue without taking the same row, and a
//! replica that dies mid-batch only delays it until the lease runs out. A text whose hash and model
//! match the stored vector is not sent to the model again. After profile or need vectors land, the
//! candidate cache generation is bumped: the next search reloads the snapshot and the need vectors.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use db::{embeddings::Vector, types::EmbeddingKind};
use embedder::TextKind;
use matching::semantic::{self, text_hash};
use sqlx::PgPool;
use uuid::Uuid;

use super::{EmbedError, EmbedderClient, profile_text};
use crate::{cache::CandidateCache, state::AppState, trust};

const BATCH: i64 = 32;
/// Longer than an embedder request may take ([`crate::config::EmbeddingsConfig::timeout`]).
const LEASE_SECS: f64 = 120.0;
const GAUGE_EVERY: Duration = Duration::from_secs(15);
/// Retry delays after the n-th failure: 5 s, 30 s, 2 min, then every 10 min.
const RETRY_SECS: [u64; 4] = [5, 30, 120, 600];

pub fn retry_delay(failures: u32) -> Duration {
    let i = (failures.max(1) as usize - 1).min(RETRY_SECS.len() - 1);
    Duration::from_secs(RETRY_SECS[i])
}

/// The stored vector is missing, from another model or from another text.
pub fn needs_embedding(
    stored_model: Option<&str>,
    stored_hash: Option<i64>,
    model: &str,
    hash: i64,
) -> bool {
    stored_model != Some(model) || stored_hash != Some(hash)
}

/// Which side of a comparison a kind of text is on (multilingual-e5 prefixes): a profile is
/// searched for, a need searches; vacancies are compared with each other, so they take the
/// symmetric `query` side.
pub fn text_kind(kind: EmbeddingKind) -> TextKind {
    match kind {
        EmbeddingKind::Candidate => TextKind::Passage,
        EmbeddingKind::Need | EmbeddingKind::Vacancy => TextKind::Query,
    }
}

/// One claimed entity as read from the database.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: Uuid,
    pub enqueued_at: chrono::DateTime<chrono::Utc>,
    pub company: Uuid,
    pub text: String,
    pub hash: i64,
    pub embed: bool,
    /// A published vacancy: checked for paraphrases of other companies' vacancies.
    pub published: bool,
    /// The text became empty while a vector from an earlier text is stored: it is dropped.
    pub stale: bool,
}

/// What a batch did: jobs settled and the model that actually answered.
struct Step {
    settled: usize,
    model: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum StepError {
    #[error(transparent)]
    Embedder(#[from] EmbedError),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub struct Worker {
    pool: PgPool,
    client: Arc<EmbedderClient>,
    candidates: Arc<CandidateCache>,
    interval: Duration,
}

pub fn spawn(state: &AppState) {
    let (Some(client), Some(config)) = (state.embedder.clone(), state.config.embeddings.as_ref())
    else {
        return;
    };
    let worker = Worker {
        pool: state.pool.clone(),
        client,
        candidates: state.candidates.clone(),
        interval: config.interval,
    };
    tokio::spawn(worker.run());
}

impl Worker {
    async fn run(self) {
        let mut model: Option<String> = None;
        let mut failures = 0u32;
        let mut gauge_at: Option<Instant> = None;
        loop {
            if gauge_at.is_none_or(|t| t.elapsed() >= GAUGE_EVERY) {
                if let Ok(n) = db::embeddings::pending_jobs(&self.pool).await {
                    metrics::gauge!("embedding_jobs_pending").set(n as f64);
                }
                gauge_at = Some(Instant::now());
            }
            let current = match &model {
                Some(m) => m.clone(),
                None => match self.client.model().await {
                    Ok(m) => {
                        match db::embeddings::enqueue_stale(&self.pool, &m).await {
                            Ok(0) => {}
                            Ok(n) => {
                                tracing::info!(model = %m, rows = n, "re-embedding vectors of another model")
                            }
                            Err(e) => tracing::warn!(error = %e, "stale vectors not enqueued"),
                        }
                        if failures > 0 {
                            tracing::info!(model = %m, "embedder is back");
                        }
                        failures = 0;
                        model = Some(m.clone());
                        m
                    }
                    Err(e) => {
                        failures += 1;
                        if failures == 1 || failures % 20 == 0 {
                            tracing::warn!(error = %e, "embedder unavailable: texts wait in the queue, matching uses keywords");
                        }
                        tokio::time::sleep(retry_delay(failures).min(Duration::from_secs(60)))
                            .await;
                        continue;
                    }
                },
            };
            match self.step(&current).await {
                Ok(step) if step.model.as_deref().is_some_and(|m| m != current) => {
                    tracing::info!(from = %current, to = ?step.model, "embedder model changed");
                    model = None;
                }
                Ok(step) if step.settled == 0 => tokio::time::sleep(self.interval).await,
                Ok(_) => {}
                Err(StepError::Embedder(e)) => {
                    failures += 1;
                    tracing::warn!(error = %e, "embedding batch failed, retried later");
                    model = None;
                    tokio::time::sleep(retry_delay(failures).min(Duration::from_secs(60))).await;
                }
                Err(e) => {
                    tracing::warn!(error = %e, "embedding worker step failed");
                    tokio::time::sleep(self.interval).await;
                }
            }
        }
    }

    async fn items(
        &self,
        kind: EmbeddingKind,
        jobs: &[db::embeddings::Job],
        model: &str,
    ) -> Result<Vec<Item>, sqlx::Error> {
        let ids: Vec<Uuid> = jobs.iter().map(|j| j.entity_id).collect();
        let enqueued = |id: Uuid| {
            jobs.iter()
                .find(|j| j.entity_id == id)
                .map(|j| j.enqueued_at)
        };
        let mut out = Vec::with_capacity(jobs.len());
        match kind {
            EmbeddingKind::Candidate => {
                for r in db::embeddings::candidate_texts(&self.pool, &ids).await? {
                    let text = profile_text(&r.roles, &r.about, &r.experience);
                    let hash = text_hash(&text);
                    let Some(enqueued_at) = enqueued(r.user_id) else {
                        continue;
                    };
                    out.push(Item {
                        id: r.user_id,
                        enqueued_at,
                        company: Uuid::nil(),
                        embed: !text.is_empty()
                            && needs_embedding(r.model.as_deref(), r.text_hash, model, hash),
                        stale: text.is_empty() && r.model.is_some(),
                        text,
                        hash,
                        published: false,
                    });
                }
            }
            EmbeddingKind::Need | EmbeddingKind::Vacancy => {
                let rows = if kind == EmbeddingKind::Need {
                    db::embeddings::need_texts(&self.pool, &ids).await?
                } else {
                    db::embeddings::vacancy_texts(&self.pool, &ids).await?
                };
                for r in rows {
                    let text = semantic::posting_text(&r.title, &r.description);
                    let hash = text_hash(&text);
                    let Some(enqueued_at) = enqueued(r.id) else {
                        continue;
                    };
                    out.push(Item {
                        id: r.id,
                        enqueued_at,
                        company: r.company_id,
                        embed: !text.is_empty()
                            && needs_embedding(r.model.as_deref(), r.text_hash, model, hash),
                        stale: false,
                        text,
                        hash,
                        published: r.published,
                    });
                }
            }
        }
        Ok(out)
    }

    /// One batch; `settled` is 0 when the queue was empty.
    async fn step(&self, model: &str) -> Result<Step, StepError> {
        let jobs = db::embeddings::claim_jobs(&self.pool, BATCH, LEASE_SECS).await?;
        if jobs.is_empty() {
            return Ok(Step {
                settled: 0,
                model: None,
            });
        }
        let mut settled = 0;
        let mut answered: Option<String> = None;
        let mut generation_changed = false;
        const KINDS: [EmbeddingKind; 3] = [
            EmbeddingKind::Candidate,
            EmbeddingKind::Need,
            EmbeddingKind::Vacancy,
        ];
        for (k, kind) in KINDS.into_iter().enumerate() {
            let of_kind: Vec<db::embeddings::Job> =
                jobs.iter().filter(|j| j.kind == kind).cloned().collect();
            if of_kind.is_empty() {
                continue;
            }
            let items = self.items(kind, &of_kind, model).await?;
            let to_embed: Vec<&Item> = items.iter().filter(|i| i.embed).collect();
            let texts: Vec<String> = to_embed.iter().map(|i| i.text.clone()).collect();
            let embedded = if texts.is_empty() {
                None
            } else {
                match self.client.embed(&texts, text_kind(kind)).await {
                    Ok(e) => Some(e),
                    Err(e) => {
                        // This kind and the ones not reached yet go back to the queue with a delay,
                        // instead of staying claimed until the lease ends.
                        for later in &KINDS[k..] {
                            let waiting: Vec<&db::embeddings::Job> =
                                jobs.iter().filter(|j| j.kind == *later).collect();
                            if waiting.is_empty() {
                                continue;
                            }
                            let ids: Vec<Uuid> = waiting.iter().map(|j| j.entity_id).collect();
                            let attempts = waiting.iter().map(|j| j.attempts).max().unwrap_or(1);
                            let delay = retry_delay(u32::try_from(attempts).unwrap_or(1));
                            db::embeddings::retry_jobs(
                                &self.pool,
                                *later,
                                &ids,
                                delay.as_secs_f64(),
                            )
                            .await?;
                            metrics::counter!("embeddings_total", "kind" => kind_label(*later), "result" => "failed")
                                .increment(ids.len() as u64);
                        }
                        return Err(e.into());
                    }
                }
            };
            let mut fresh: Vec<(Uuid, Vector)> = Vec::new();
            let mut tx = self.pool.begin().await?;
            for item in items.iter().filter(|i| i.stale) {
                db::embeddings::delete_candidate_embedding(&mut *tx, item.id).await?;
                generation_changed = true;
            }
            if let Some(e) = &embedded {
                answered = Some(e.model.clone());
                for (item, v) in to_embed.iter().zip(&e.vectors) {
                    let vector = Vector::from(v.clone());
                    match kind {
                        EmbeddingKind::Candidate => {
                            db::embeddings::upsert_candidate_embedding(
                                &mut *tx, item.id, &e.model, item.hash, &vector,
                            )
                            .await?
                        }
                        EmbeddingKind::Need => {
                            db::embeddings::upsert_need_embedding(
                                &mut *tx, item.id, &e.model, item.hash, &vector,
                            )
                            .await?
                        }
                        EmbeddingKind::Vacancy => {
                            db::embeddings::upsert_vacancy_embedding(
                                &mut *tx, item.id, &e.model, item.hash, &vector,
                            )
                            .await?
                        }
                    };
                    fresh.push((item.id, vector));
                }
            }
            let ids: Vec<Uuid> = of_kind.iter().map(|j| j.entity_id).collect();
            let versions: Vec<_> = of_kind.iter().map(|j| j.enqueued_at).collect();
            db::embeddings::finish_jobs(&mut *tx, kind, &ids, &versions).await?;
            tx.commit().await?;
            settled += of_kind.len();
            metrics::counter!("embeddings_total", "kind" => kind_label(kind), "result" => "embedded")
                .increment(fresh.len() as u64);
            metrics::counter!("embeddings_total", "kind" => kind_label(kind), "result" => "unchanged")
                .increment((of_kind.len() - fresh.len()) as u64);
            generation_changed |= kind != EmbeddingKind::Vacancy && !fresh.is_empty();
            if kind == EmbeddingKind::Vacancy {
                let used = embedded.as_ref().map_or(model, |e| e.model.as_str());
                self.check_paraphrases(&items, &fresh, used).await;
            }
        }
        if generation_changed {
            self.candidates.invalidate().await;
        }
        Ok(Step {
            settled,
            model: answered,
        })
    }

    /// The vectors are already stored: a failed check is logged and waits for the vacancy's next
    /// change rather than undoing the batch.
    async fn check_paraphrases(&self, items: &[Item], fresh: &[(Uuid, Vector)], model: &str) {
        for item in items.iter().filter(|i| i.published) {
            let checked = match fresh.iter().find(|(id, _)| *id == item.id) {
                Some((_, v)) => {
                    trust::check_paraphrase(&self.pool, item.id, item.company, v, model).await
                }
                None => match db::embeddings::vacancy_vector(&self.pool, item.id).await {
                    Ok(Some(v)) if v.model == model => {
                        trust::check_paraphrase(
                            &self.pool,
                            item.id,
                            item.company,
                            &v.embedding,
                            model,
                        )
                        .await
                    }
                    Ok(_) => continue,
                    Err(e) => Err(e.into()),
                },
            };
            if let Err(e) = checked {
                tracing::warn!(vacancy = %item.id, error = %e, "paraphrase check failed");
            }
        }
    }
}

fn kind_label(kind: EmbeddingKind) -> &'static str {
    match kind {
        EmbeddingKind::Candidate => "candidate",
        EmbeddingKind::Need => "need",
        EmbeddingKind::Vacancy => "vacancy",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_back_off_and_stay_capped() {
        let delays: Vec<u64> = (0..7).map(|n| retry_delay(n).as_secs()).collect();
        assert_eq!(delays, [5, 5, 30, 120, 600, 600, 600]);
    }

    #[test]
    fn only_a_missing_foreign_or_outdated_vector_is_recomputed() {
        let m = "multilingual-e5-small@614241f";
        assert!(needs_embedding(None, None, m, 1));
        assert!(needs_embedding(Some("rubert-tiny2@e8ed3b0"), Some(1), m, 1));
        assert!(needs_embedding(Some(m), Some(2), m, 1));
        assert!(!needs_embedding(Some(m), Some(1), m, 1));
    }

    #[test]
    fn profiles_are_passages_and_postings_are_queries() {
        assert_eq!(text_kind(EmbeddingKind::Candidate), TextKind::Passage);
        assert_eq!(text_kind(EmbeddingKind::Need), TextKind::Query);
        assert_eq!(text_kind(EmbeddingKind::Vacancy), TextKind::Query);
    }
}
