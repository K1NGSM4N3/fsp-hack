-- Sentence embeddings of profiles, needs and vacancies (docs/solution.md, «Машинное обучение»).
-- The embedder service (crates/embedder, multilingual-e5-small, 384 dimensions) computes them;
-- the API never waits for it: a trigger enqueues the entity in the same transaction as the write
-- of its text, and a background worker embeds the queue. Without the embedder nothing breaks:
-- the queue just waits, and every feature falls back to keywords.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TYPE embedding_kind AS ENUM ('candidate', 'need', 'vacancy');

-- What still has to be embedded. One row per entity: a new write while the row waits only moves
-- `enqueued_at`. A worker claims rows by setting `claimed_until` (FOR UPDATE SKIP LOCKED, so
-- replicas never take the same row) and deletes a row only if `enqueued_at` did not move while it
-- worked; a crashed worker's claim simply expires.
CREATE TABLE embedding_jobs (
    kind           embedding_kind NOT NULL,
    entity_id      uuid           NOT NULL,
    enqueued_at    timestamptz    NOT NULL DEFAULT clock_timestamp(),
    attempts       integer        NOT NULL DEFAULT 0,
    claimed_until  timestamptz,
    PRIMARY KEY (kind, entity_id)
);
CREATE INDEX embedding_jobs_due_idx ON embedding_jobs (enqueued_at);

-- `model` is `name@revision` of the weights; `text_hash` (matching::semantic::text_hash) is the text
-- the vector came from, so an unchanged text is not sent to the model again.
CREATE TABLE candidate_embeddings (
    candidate_id  uuid        PRIMARY KEY REFERENCES candidates (user_id) ON DELETE CASCADE,
    model         text        NOT NULL,
    text_hash     bigint      NOT NULL,
    embedding     vector(384) NOT NULL,
    embedded_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE need_embeddings (
    need_id       uuid        PRIMARY KEY REFERENCES hiring_needs (id) ON DELETE CASCADE,
    model         text        NOT NULL,
    text_hash     bigint      NOT NULL,
    embedding     vector(384) NOT NULL,
    embedded_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE vacancy_embeddings (
    vacancy_id    uuid        PRIMARY KEY REFERENCES vacancies (id) ON DELETE CASCADE,
    model         text        NOT NULL,
    text_hash     bigint      NOT NULL,
    embedding     vector(384) NOT NULL,
    embedded_at   timestamptz NOT NULL DEFAULT now()
);
-- Nearest vacancies: «Похожие вакансии» and the paraphrase check against other companies.
-- Profiles and needs have no index: matching compares a need with the candidates of one
-- specialization already in the API's memory.
CREATE INDEX vacancy_embeddings_hnsw_idx ON vacancy_embeddings USING hnsw (embedding vector_cosine_ops);

CREATE FUNCTION enqueue_candidate_embedding() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO embedding_jobs (kind, entity_id) VALUES ('candidate', NEW.user_id)
    ON CONFLICT (kind, entity_id)
    DO UPDATE SET enqueued_at = clock_timestamp(), attempts = 0, claimed_until = NULL;
    RETURN NULL;
END
$$;

-- Needs and vacancies: TG_ARGV[0] is the kind.
CREATE FUNCTION enqueue_posting_embedding() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO embedding_jobs (kind, entity_id) VALUES (TG_ARGV[0]::embedding_kind, NEW.id)
    ON CONFLICT (kind, entity_id)
    DO UPDATE SET enqueued_at = clock_timestamp(), attempts = 0, claimed_until = NULL;
    RETURN NULL;
END
$$;

CREATE TRIGGER candidates_embedding_insert AFTER INSERT ON candidates
    FOR EACH ROW EXECUTE FUNCTION enqueue_candidate_embedding();
CREATE TRIGGER candidates_embedding_update AFTER UPDATE OF about, roles, experience ON candidates
    FOR EACH ROW
    WHEN (OLD.about IS DISTINCT FROM NEW.about
          OR OLD.roles IS DISTINCT FROM NEW.roles
          OR OLD.experience IS DISTINCT FROM NEW.experience)
    EXECUTE FUNCTION enqueue_candidate_embedding();

CREATE TRIGGER hiring_needs_embedding_insert AFTER INSERT ON hiring_needs
    FOR EACH ROW EXECUTE FUNCTION enqueue_posting_embedding('need');
CREATE TRIGGER hiring_needs_embedding_update AFTER UPDATE OF title, description ON hiring_needs
    FOR EACH ROW
    WHEN (OLD.title IS DISTINCT FROM NEW.title OR OLD.description IS DISTINCT FROM NEW.description)
    EXECUTE FUNCTION enqueue_posting_embedding('need');

-- A vacancy is also re-checked when it is published again: the paraphrase check looks at the
-- other companies' vacancies published by then.
CREATE TRIGGER vacancies_embedding_insert AFTER INSERT ON vacancies
    FOR EACH ROW EXECUTE FUNCTION enqueue_posting_embedding('vacancy');
CREATE TRIGGER vacancies_embedding_update AFTER UPDATE OF title, description, status ON vacancies
    FOR EACH ROW
    WHEN (OLD.title IS DISTINCT FROM NEW.title
          OR OLD.description IS DISTINCT FROM NEW.description
          OR OLD.status IS DISTINCT FROM NEW.status)
    EXECUTE FUNCTION enqueue_posting_embedding('vacancy');

-- Rows written before this migration.
INSERT INTO embedding_jobs (kind, entity_id) SELECT 'candidate', user_id FROM candidates;
INSERT INTO embedding_jobs (kind, entity_id) SELECT 'need', id FROM hiring_needs;
INSERT INTO embedding_jobs (kind, entity_id) SELECT 'vacancy', id FROM vacancies;
