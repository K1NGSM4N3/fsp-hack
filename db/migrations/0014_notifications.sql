-- Notifications (architecture.md §13): an in-app feed written in the same transaction as the
-- change it reports, and email delivery that batches the feed into letters with a rate cap.

CREATE TYPE notification_email_status AS ENUM ('pending', 'sent', 'failed', 'skipped');

-- One letter: the items it carries point at it. The id is also the Message-ID, so a retried send
-- is the same letter for the recipient's mail client.
CREATE TABLE notification_emails (
    id               uuid PRIMARY KEY,
    user_id          uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    status           notification_email_status NOT NULL DEFAULT 'pending',
    attempts         smallint    NOT NULL DEFAULT 0,
    next_attempt_at  timestamptz NOT NULL DEFAULT now(),
    created_at       timestamptz NOT NULL DEFAULT now(),
    sent_at          timestamptz,
    last_error       text
);
CREATE INDEX notification_emails_due_idx ON notification_emails (next_attempt_at) WHERE status = 'pending';
-- The rate cap counts a user's letters of the last 24 hours.
CREATE INDEX notification_emails_user_idx ON notification_emails (user_id, created_at DESC);

CREATE TABLE notifications (
    -- UUIDv7: the feed is ordered and paged by id.
    id            uuid PRIMARY KEY,
    user_id       uuid        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind          text        NOT NULL CHECK (kind IN (
        'invitation_received', 'invitation_expiring', 'invitation_withdrawn', 'hiring_stage',
        'application_status', 'assessment_result', 'guardian_decision',
        'invitation_accepted', 'invitation_declined', 'application_received', 'webhook_failing',
        'trust_changed')),
    -- What the text is rendered from: ids, company name, title, salary, the candidate's anonymous
    -- code. Never a name or contacts. Written and read whole, hence json.
    payload       json        NOT NULL,
    -- The invitation, application or test the item is about: opening it marks the item read.
    subject_id    uuid,
    -- Items about a candidate or a company go away with that account, like the ATS outbox.
    candidate_id  uuid REFERENCES candidates (user_id) ON DELETE CASCADE,
    company_id    uuid REFERENCES companies (id) ON DELETE CASCADE,
    -- One expiry reminder per invitation, one integration alert per company and day.
    dedup_key     text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    read_at       timestamptz,
    -- When the item may leave by email; NULL — in-app only, or already handed to a letter.
    email_due_at  timestamptz,
    email_id      uuid REFERENCES notification_emails (id) ON DELETE SET NULL,
    UNIQUE (user_id, dedup_key)
);
CREATE INDEX notifications_feed_idx ON notifications (user_id, id DESC);
-- The unread badge is polled: a count over this index only.
CREATE INDEX notifications_unread_idx ON notifications (user_id) WHERE read_at IS NULL;
CREATE INDEX notifications_email_due_idx ON notifications (email_due_at) WHERE email_due_at IS NOT NULL;
CREATE INDEX notifications_email_idx ON notifications (email_id) WHERE email_id IS NOT NULL;
CREATE INDEX notifications_candidate_idx ON notifications (candidate_id) WHERE candidate_id IS NOT NULL;
CREATE INDEX notifications_company_idx ON notifications (company_id) WHERE company_id IS NOT NULL;

-- In-app is always on; email can be turned off per kind. Bit n stands for the n-th kind of
-- `api::notify::NotificationKind` — bits are never renumbered. No row: every email on.
CREATE TABLE notification_settings (
    user_id     uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    email_off   integer     NOT NULL DEFAULT 0,
    updated_at  timestamptz NOT NULL DEFAULT now()
);

-- The trust level and quota the company was last told about. The sweeper compares the current
-- assessment with it and writes `trust_changed` when either moves; the first look only records.
CREATE TABLE company_trust_seen (
    company_id  uuid PRIMARY KEY REFERENCES companies (id) ON DELETE CASCADE,
    level       text        NOT NULL,
    quota       smallint    NOT NULL,
    updated_at  timestamptz NOT NULL DEFAULT now()
);
