-- Initial schema. See architecture.md §10 for the overview.
-- Personal data columns end with `_enc` (XChaCha20-Poly1305, see crates/api/src/crypto.rs).
-- Slugs (specialization, grade, skills, formats, industries) are validated in the app
-- against the catalog crate, so they are plain text here.

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TYPE user_role AS ENUM ('candidate', 'employer', 'admin');

CREATE TABLE users (
    id                 uuid PRIMARY KEY,
    role               user_role   NOT NULL,
    email_enc          bytea       NOT NULL,
    -- HMAC-SHA256 of the normalized email: lookup without storing the email in clear.
    email_bidx         bytea       NOT NULL UNIQUE,
    password_hash      text        NOT NULL,
    email_verified_at  timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    last_login_at      timestamptz,
    deleted_at         timestamptz
);

CREATE TABLE email_verifications (
    token_hash  bytea PRIMARY KEY,
    user_id     uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    expires_at  timestamptz NOT NULL,
    used_at     timestamptz,
    created_at  timestamptz NOT NULL DEFAULT now()
);

-- Refresh tokens rotate on every use. Reuse of a rotated token revokes the whole family.
CREATE TABLE refresh_tokens (
    id          uuid PRIMARY KEY,
    user_id     uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    family_id   uuid        NOT NULL,
    token_hash  bytea       NOT NULL UNIQUE,
    expires_at  timestamptz NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    rotated_at  timestamptz,
    revoked_at  timestamptz
);
CREATE INDEX refresh_tokens_family_idx ON refresh_tokens (family_id);

-- ---------------------------------------------------------------------------
-- Consents (152-ФЗ): versioned documents and the fact of granting / revoking.
-- ---------------------------------------------------------------------------

CREATE TABLE consent_documents (
    id            uuid PRIMARY KEY,
    kind          text        NOT NULL,          -- pd_processing | pd_publication | terms
    version       integer     NOT NULL,
    title         text        NOT NULL,
    body          text        NOT NULL,
    required_for  user_role[] NOT NULL,
    published_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (kind, version)
);

CREATE TABLE user_consents (
    id           uuid PRIMARY KEY,
    user_id      uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    document_id  uuid        NOT NULL REFERENCES consent_documents (id),
    granted_at   timestamptz NOT NULL DEFAULT now(),
    ip           text,
    user_agent   text,
    revoked_at   timestamptz
);
CREATE INDEX user_consents_user_idx ON user_consents (user_id);

-- ---------------------------------------------------------------------------
-- Candidates
-- ---------------------------------------------------------------------------

CREATE TYPE search_status AS ENUM ('active', 'open', 'closed');

CREATE TABLE candidates (
    user_id                 uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    -- Anonymous handle shown to employers before contacts are granted, e.g. "K-7F3A2C".
    public_code             text        NOT NULL UNIQUE,
    full_name_enc           bytea,
    -- Encrypted JSON {phone, telegram, email}.
    contacts_enc            bytea,
    city                    text,
    relocation              boolean     NOT NULL DEFAULT false,
    work_formats            text[]      NOT NULL DEFAULT '{}',
    salary_expectation_min  bigint CHECK (salary_expectation_min IS NULL OR salary_expectation_min > 0),
    about                   text        NOT NULL DEFAULT '',
    experience_years        real,
    roles                   text[]      NOT NULL DEFAULT '{}',
    soft_skills             text[]      NOT NULL DEFAULT '{}',
    industries              text[]      NOT NULL DEFAULT '{}',
    -- [{company, position, from, to, description}]; company names hidden unless show_experience.
    experience              jsonb       NOT NULL DEFAULT '[]',
    search_status           search_status NOT NULL DEFAULT 'open',
    -- Privacy settings (C-7).
    visible                 boolean     NOT NULL DEFAULT true,
    show_name               boolean     NOT NULL DEFAULT false,
    show_fsp                boolean     NOT NULL DEFAULT true,
    show_experience         boolean     NOT NULL DEFAULT true,
    -- Current category, denormalized from the latest grade_assignments row for fast search.
    specialization          text,
    grade                   text,
    theta                   double precision,
    category_assigned_at    timestamptz,
    last_activity_at        timestamptz NOT NULL DEFAULT now(),
    created_at              timestamptz NOT NULL DEFAULT now(),
    updated_at              timestamptz NOT NULL DEFAULT now(),
    about_tsv               tsvector GENERATED ALWAYS AS (to_tsvector('russian', about)) STORED
);
CREATE INDEX candidates_category_idx ON candidates (specialization, grade) WHERE visible AND search_status <> 'closed';
CREATE INDEX candidates_about_tsv_idx ON candidates USING gin (about_tsv);

CREATE TABLE candidate_skills (
    candidate_id  uuid NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    skill         text NOT NULL,
    source        text NOT NULL CHECK (source IN ('declared', 'resume', 'test', 'fsp')),
    PRIMARY KEY (candidate_id, skill, source)
);
CREATE INDEX candidate_skills_skill_idx ON candidate_skills (skill);

CREATE TABLE resume_uploads (
    id                 uuid PRIMARY KEY,
    candidate_id       uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    filename           text        NOT NULL,
    size_bytes         integer     NOT NULL,
    recognized_fields  text[]      NOT NULL DEFAULT '{}',
    created_at         timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE survey_responses (
    id                uuid PRIMARY KEY,
    candidate_id      uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    direction         text        NOT NULL,
    specialization    text        NOT NULL,
    claimed_grade     text        NOT NULL,
    skills            text[]      NOT NULL DEFAULT '{}',
    industries        text[]      NOT NULL DEFAULT '{}',
    experience_years  real,
    self_assessment   jsonb       NOT NULL DEFAULT '{}',
    created_at        timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX survey_responses_candidate_idx ON survey_responses (candidate_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- Assessment (architecture.md §5)
-- ---------------------------------------------------------------------------

CREATE TYPE assessment_status AS ENUM ('in_progress', 'passed', 'failed', 'expired');

CREATE TABLE assessment_sessions (
    id                     uuid PRIMARY KEY,
    candidate_id           uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    specialization         text        NOT NULL,
    claimed_grade          text        NOT NULL,
    attempt_kind           text        NOT NULL,
    status                 assessment_status NOT NULL DEFAULT 'in_progress',
    -- Serialized assessment::SessionState; the item sequence is reproducible from it.
    state                  jsonb       NOT NULL,
    -- ItemRef currently shown to the candidate and its deadline (timeouts count as wrong).
    current_item           jsonb,
    current_item_deadline  timestamptz,
    current_item_shown_at  timestamptz,
    theta                  double precision,
    se                     double precision,
    measured_grade         text,
    confident_above_next   boolean,
    person_fit_lz          double precision,
    aberrant               boolean,
    items_count            integer     NOT NULL DEFAULT 0,
    started_at             timestamptz NOT NULL DEFAULT now(),
    finished_at            timestamptz
);
CREATE UNIQUE INDEX assessment_sessions_one_active_idx
    ON assessment_sessions (candidate_id) WHERE status = 'in_progress';
CREATE INDEX assessment_sessions_candidate_idx ON assessment_sessions (candidate_id, started_at DESC);

CREATE TABLE assessment_responses (
    session_id   uuid        NOT NULL REFERENCES assessment_sessions (id) ON DELETE CASCADE,
    seq          integer     NOT NULL,
    template_id  text        NOT NULL,
    level        smallint    NOT NULL,
    seed         bigint      NOT NULL,
    answer       jsonb,
    correct      boolean     NOT NULL,
    timed_out    boolean     NOT NULL DEFAULT false,
    elapsed_ms   integer     NOT NULL,
    theta_after  double precision NOT NULL,
    se_after     double precision NOT NULL,
    answered_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, seq)
);

CREATE TABLE grade_assignments (
    id              uuid PRIMARY KEY,
    candidate_id    uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    specialization  text        NOT NULL,
    grade           text        NOT NULL,
    theta           double precision NOT NULL,
    se              double precision NOT NULL,
    session_id      uuid REFERENCES assessment_sessions (id) ON DELETE SET NULL,
    reason          text        NOT NULL,   -- placement | upgrade | downgrade
    assigned_at     timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX grade_assignments_candidate_idx ON grade_assignments (candidate_id, assigned_at DESC);

-- ---------------------------------------------------------------------------
-- FSP (architecture.md §7)
-- ---------------------------------------------------------------------------

CREATE TABLE fsp_links (
    candidate_id      uuid PRIMARY KEY REFERENCES candidates (user_id) ON DELETE CASCADE,
    fsp_id            text        NOT NULL UNIQUE,
    linked_via        text        NOT NULL CHECK (linked_via IN ('oidc', 'dev', 'seed')),
    linked_at         timestamptz NOT NULL DEFAULT now(),
    last_sync_at      timestamptz,
    -- Snapshot of the registry response (matching::fsp::Achievement[]).
    achievements      jsonb       NOT NULL DEFAULT '[]',
    has_achievements  boolean GENERATED ALWAYS AS (jsonb_array_length(achievements) > 0) STORED
);

-- OIDC authorization requests in flight (state → PKCE verifier), short-lived.
CREATE TABLE oidc_states (
    state          text PRIMARY KEY,
    user_id        uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    pkce_verifier  text        NOT NULL,
    nonce          text        NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- Employers
-- ---------------------------------------------------------------------------

CREATE TABLE companies (
    id                uuid PRIMARY KEY,
    owner_user_id     uuid        NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    name              text        NOT NULL,
    description       text        NOT NULL DEFAULT '',
    industry          text,
    website           text,
    city              text,
    size              text,
    -- Public company contacts, shown to candidates in invitations.
    contact_email     text,
    contact_phone     text,
    contact_telegram  text,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE hiring_needs (
    id                     uuid PRIMARY KEY,
    company_id             uuid        NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    title                  text        NOT NULL,
    description            text        NOT NULL DEFAULT '',
    specialization         text        NOT NULL,
    grades                 text[]      NOT NULL,
    allow_adjacent_grades  boolean     NOT NULL DEFAULT false,
    must_skills            text[]      NOT NULL DEFAULT '{}',
    nice_skills            text[]      NOT NULL DEFAULT '{}',
    salary_min             bigint      NOT NULL,
    salary_max             bigint      NOT NULL,
    work_formats           text[]      NOT NULL DEFAULT '{}',
    city                   text,
    industries             text[]      NOT NULL DEFAULT '{}',
    -- Current refinement filters on top of the need (M-4).
    filters                jsonb       NOT NULL DEFAULT '{}',
    status                 text        NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'closed')),
    created_at             timestamptz NOT NULL DEFAULT now(),
    updated_at             timestamptz NOT NULL DEFAULT now(),
    CHECK (salary_min > 0 AND salary_min <= salary_max)
);
CREATE INDEX hiring_needs_company_idx ON hiring_needs (company_id);

-- Persisted shortlist: refining the query re-scores rows but keeps pinned/dismissed flags
-- and remembers when each candidate first appeared ("без потери подборки").
CREATE TABLE shortlist_items (
    need_id        uuid        NOT NULL REFERENCES hiring_needs (id) ON DELETE CASCADE,
    candidate_id   uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    score          double precision NOT NULL,
    rank           integer     NOT NULL,
    breakdown      jsonb       NOT NULL,
    in_current     boolean     NOT NULL DEFAULT true,
    pinned         boolean     NOT NULL DEFAULT false,
    dismissed      boolean     NOT NULL DEFAULT false,
    first_seen_at  timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (need_id, candidate_id)
);

CREATE TABLE profile_views (
    company_id    uuid NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    candidate_id  uuid NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    viewed_on     date NOT NULL DEFAULT current_date,
    PRIMARY KEY (company_id, candidate_id, viewed_on)
);

-- ---------------------------------------------------------------------------
-- Vacancies and applications (optional flow, C-6 / E-6)
-- ---------------------------------------------------------------------------

CREATE TYPE vacancy_status AS ENUM ('draft', 'published', 'closed');

CREATE TABLE vacancies (
    id              uuid PRIMARY KEY,
    company_id      uuid        NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    title           text        NOT NULL,
    description     text        NOT NULL DEFAULT '',
    specialization  text        NOT NULL,
    grade           text        NOT NULL,
    skills          text[]      NOT NULL DEFAULT '{}',
    salary_min      bigint      NOT NULL,
    salary_max      bigint      NOT NULL,
    work_format     text        NOT NULL,
    city            text,
    status          vacancy_status NOT NULL DEFAULT 'draft',
    published_at    timestamptz,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    search_tsv      tsvector GENERATED ALWAYS AS (
        to_tsvector('russian', title || ' ' || description)) STORED,
    CHECK (salary_min > 0 AND salary_min <= salary_max)
);
CREATE INDEX vacancies_published_idx ON vacancies (published_at DESC) WHERE status = 'published';
CREATE INDEX vacancies_tsv_idx ON vacancies USING gin (search_tsv);

CREATE TYPE application_status AS ENUM ('sent', 'viewed', 'accepted', 'rejected', 'withdrawn');

CREATE TABLE applications (
    id            uuid PRIMARY KEY,
    vacancy_id    uuid        NOT NULL REFERENCES vacancies (id) ON DELETE CASCADE,
    candidate_id  uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    cover_letter  text        NOT NULL DEFAULT '',
    status        application_status NOT NULL DEFAULT 'sent',
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    UNIQUE (vacancy_id, candidate_id)
);

-- ---------------------------------------------------------------------------
-- Invitations — the core contact mechanic (architecture.md §6)
-- ---------------------------------------------------------------------------

CREATE TYPE invitation_status AS ENUM ('sent', 'viewed', 'accepted', 'declined', 'expired', 'withdrawn');

CREATE TABLE invitations (
    id               uuid PRIMARY KEY,
    company_id       uuid        NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    candidate_id     uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    need_id          uuid REFERENCES hiring_needs (id) ON DELETE SET NULL,
    vacancy_id       uuid REFERENCES vacancies (id) ON DELETE SET NULL,
    title            text        NOT NULL,
    message          text        NOT NULL,
    salary_min       bigint      NOT NULL,
    salary_max       bigint      NOT NULL,
    work_format      text,
    -- How the candidate reaches the company (V-2).
    contact          text        NOT NULL,
    status           invitation_status NOT NULL DEFAULT 'sent',
    decline_reason   text,
    decline_comment  text,
    idempotency_key  text,
    created_at       timestamptz NOT NULL DEFAULT now(),
    viewed_at        timestamptz,
    responded_at     timestamptz,
    expires_at       timestamptz NOT NULL,
    CHECK (salary_min > 0 AND salary_min <= salary_max),
    UNIQUE (company_id, idempotency_key)
);
-- One live invitation per company–candidate pair: no spamming the same person.
CREATE UNIQUE INDEX invitations_one_active_idx
    ON invitations (company_id, candidate_id) WHERE status IN ('sent', 'viewed');
CREATE INDEX invitations_candidate_idx ON invitations (candidate_id, created_at DESC);
CREATE INDEX invitations_company_idx ON invitations (company_id, created_at DESC);

CREATE TABLE invitation_events (
    id             uuid PRIMARY KEY,
    invitation_id  uuid        NOT NULL REFERENCES invitations (id) ON DELETE CASCADE,
    status         invitation_status NOT NULL,
    actor_user_id  uuid,
    at             timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX invitation_events_invitation_idx ON invitation_events (invitation_id, at);

-- Who saw whose contacts, when and why (152-ФЗ audit). No FKs: the log outlives accounts.
CREATE TABLE contact_access_log (
    id                uuid PRIMARY KEY,
    employer_user_id  uuid        NOT NULL,
    company_id        uuid        NOT NULL,
    candidate_id      uuid        NOT NULL,
    basis             text        NOT NULL CHECK (basis IN ('invitation', 'application')),
    basis_id          uuid        NOT NULL,
    ip                text,
    user_agent        text,
    accessed_at       timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX contact_access_log_candidate_idx ON contact_access_log (candidate_id, accessed_at DESC);

-- ---------------------------------------------------------------------------
-- Regular short tasks (T-8)
-- ---------------------------------------------------------------------------

CREATE TABLE tasks (
    id              uuid PRIMARY KEY,
    company_id      uuid REFERENCES companies (id) ON DELETE CASCADE,
    specialization  text        NOT NULL,
    grades          text[]      NOT NULL,
    skills          text[]      NOT NULL DEFAULT '{}',
    title           text        NOT NULL,
    body            text        NOT NULL,
    kind            text        NOT NULL CHECK (kind IN ('solution', 'approach')),
    active          boolean     NOT NULL DEFAULT true,
    created_at      timestamptz NOT NULL DEFAULT now()
);

CREATE TYPE task_assignment_status AS ENUM ('assigned', 'submitted', 'reviewed', 'skipped', 'expired');

CREATE TABLE task_assignments (
    id              uuid PRIMARY KEY,
    task_id         uuid        NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    candidate_id    uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    status          task_assignment_status NOT NULL DEFAULT 'assigned',
    answer          text,
    rating          smallint CHECK (rating BETWEEN 1 AND 5),
    review_comment  text,
    assigned_at     timestamptz NOT NULL DEFAULT now(),
    due_at          timestamptz NOT NULL,
    submitted_at    timestamptz,
    reviewed_at     timestamptz,
    UNIQUE (task_id, candidate_id)
);

-- ---------------------------------------------------------------------------
-- Reports (anti-fake, X-1)
-- ---------------------------------------------------------------------------

CREATE TABLE reports (
    id                uuid PRIMARY KEY,
    reporter_user_id  uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    target_kind       text        NOT NULL CHECK (target_kind IN ('vacancy', 'invitation', 'company')),
    target_id         uuid        NOT NULL,
    reason            text        NOT NULL,
    comment           text,
    created_at        timestamptz NOT NULL DEFAULT now(),
    resolved_at       timestamptz
);
