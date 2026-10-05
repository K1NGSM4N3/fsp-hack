-- Integration with employers' ATS (architecture.md §11, docs/solution.md §10): a transactional
-- outbox of integration events, one outbound webhook per company, company API keys and hiring
-- outcomes reported back by the ATS.

-- How contacts left the platform: the employer cabinet, the company's ATS over the API, or a
-- webhook payload after the candidate's acceptance.
ALTER TABLE contact_access_log
    ADD COLUMN channel text NOT NULL DEFAULT 'cabinet' CHECK (channel IN ('cabinet', 'api', 'webhook')),
    ADD COLUMN api_key_id uuid;

CREATE TABLE company_webhooks (
    company_id         uuid PRIMARY KEY REFERENCES companies (id) ON DELETE CASCADE,
    url                text        NOT NULL,
    -- HMAC key, encrypted like personal data: the dispatcher needs it in clear to sign.
    secret_enc         bytea       NOT NULL,
    -- Leading characters of the secret, shown in the cabinet after the one-time reveal.
    secret_hint        text        NOT NULL,
    -- After a rotation the previous secret keeps signing for a day, so the ATS can switch over.
    prev_secret_enc    bytea,
    prev_expires_at    timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    secret_rotated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TYPE integration_event_status AS ENUM ('pending', 'delivered', 'failed');

-- Written in the same transaction as the domain change, delivered by the dispatcher. After
-- acceptance a payload carries contacts, so it is encrypted at rest.
CREATE TABLE integration_events (
    id               uuid PRIMARY KEY,
    company_id       uuid        NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    -- Events with a candidate's data go away together with the candidate's account.
    candidate_id     uuid REFERENCES candidates (user_id) ON DELETE CASCADE,
    event_type       text        NOT NULL CHECK (event_type IN (
        'invitation.sent', 'invitation.viewed', 'invitation.accepted', 'invitation.declined',
        'invitation.expired', 'invitation.withdrawn', 'application.created',
        'application.status_changed', 'contacts.opened', 'integration.test')),
    payload_enc      bytea       NOT NULL,
    status           integration_event_status NOT NULL DEFAULT 'pending',
    -- Failed attempts since creation or the last manual retry; drives the backoff schedule.
    attempts         smallint    NOT NULL DEFAULT 0,
    next_attempt_at  timestamptz NOT NULL DEFAULT now(),
    -- Give up after this moment: 24 hours after creation or after a manual retry.
    deadline_at      timestamptz NOT NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    delivered_at     timestamptz,
    last_error       text
);
CREATE INDEX integration_events_due_idx ON integration_events (next_attempt_at) WHERE status = 'pending';
CREATE INDEX integration_events_company_idx ON integration_events (company_id, created_at DESC);

CREATE TABLE webhook_deliveries (
    id            uuid PRIMARY KEY,
    event_id      uuid        NOT NULL REFERENCES integration_events (id) ON DELETE CASCADE,
    url           text        NOT NULL,
    status_code   smallint,
    latency_ms    integer     NOT NULL CHECK (latency_ms >= 0),
    error         text,
    attempted_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX webhook_deliveries_event_idx ON webhook_deliveries (event_id, attempted_at);

CREATE TABLE company_api_keys (
    id            uuid PRIMARY KEY,
    company_id    uuid        NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    name          text        NOT NULL,
    -- Visible start of the key, e.g. fsp_live_ab12cd34; the key itself is never stored.
    prefix        text        NOT NULL,
    -- SHA-256 of the whole key: 256 random bits need no slow password hash.
    key_hash      bytea       NOT NULL UNIQUE,
    scopes        text[]      NOT NULL CHECK (
        cardinality(scopes) > 0
        AND scopes <@ ARRAY['invitations:read', 'candidates:read', 'outcomes:write', 'needs:write']),
    created_by    uuid REFERENCES users (id) ON DELETE SET NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    last_used_at  timestamptz,
    revoked_at    timestamptz
);
CREATE INDEX company_api_keys_company_idx ON company_api_keys (company_id, created_at DESC);

CREATE TYPE hiring_stage AS ENUM ('interview', 'offer', 'hired', 'rejected');

-- What happened after the contact, as reported by the company's ATS. The candidate sees it in
-- the timeline; for the platform these are real hiring labels for calibrating matching weights.
CREATE TABLE hiring_outcomes (
    id              uuid PRIMARY KEY,
    company_id      uuid        NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    candidate_id    uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    invitation_id   uuid REFERENCES invitations (id) ON DELETE CASCADE,
    application_id  uuid REFERENCES applications (id) ON DELETE CASCADE,
    subject_id      uuid GENERATED ALWAYS AS (coalesce(invitation_id, application_id)) STORED,
    stage           hiring_stage NOT NULL,
    -- Shown to the candidate.
    reason          text,
    occurred_at     timestamptz NOT NULL,
    recorded_at     timestamptz NOT NULL DEFAULT now(),
    api_key_id      uuid REFERENCES company_api_keys (id) ON DELETE SET NULL,
    CHECK (num_nonnulls(invitation_id, application_id) = 1),
    -- Re-sending the same stage updates it instead of duplicating: ATS retries are safe.
    UNIQUE (subject_id, stage)
);
CREATE INDEX hiring_outcomes_candidate_idx ON hiring_outcomes (candidate_id);
CREATE INDEX hiring_outcomes_company_idx ON hiring_outcomes (company_id);
