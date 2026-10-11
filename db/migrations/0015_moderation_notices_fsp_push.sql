-- Moderation decisions and ФСП registry pushes reach the people they concern (architecture.md
-- §13, §7.1): notification kinds for test reviews, anti-fake actions and new achievements, and
-- the registry events already applied.

ALTER TABLE notifications DROP CONSTRAINT notifications_kind_check;
ALTER TABLE notifications ADD CONSTRAINT notifications_kind_check CHECK (kind IN (
    'invitation_received', 'invitation_expiring', 'invitation_withdrawn', 'hiring_stage',
    'application_status', 'assessment_result', 'guardian_decision',
    'invitation_accepted', 'invitation_declined', 'application_received', 'webhook_failing',
    'trust_changed',
    'result_confirmed', 'result_annulled', 'retest_ordered',
    'vacancy_unpublished', 'invitations_frozen', 'sanction_lifted', 'complaint_resolved',
    'fsp_achievements'));

-- Registry pushes already applied: a redelivered `event_id` changes nothing. Only events about a
-- linked participant are kept — an unknown `fsp_id` leaves no trace — and they go away with the
-- link (and so with the account), or after 30 days, long past any redelivery.
CREATE TABLE fsp_registry_events (
    event_id      text        PRIMARY KEY CHECK (length(event_id) BETWEEN 1 AND 128),
    candidate_id  uuid        NOT NULL REFERENCES fsp_links (candidate_id) ON DELETE CASCADE,
    occurred_at   timestamptz NOT NULL,
    received_at   timestamptz NOT NULL DEFAULT now(),
    -- The sync changed the snapshot, and the candidate got a notification.
    changed       boolean     NOT NULL
);
CREATE INDEX fsp_registry_events_received_idx ON fsp_registry_events (received_at);
CREATE INDEX fsp_registry_events_candidate_idx ON fsp_registry_events (candidate_id);
