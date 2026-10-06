-- Moderation (architecture.md §5.7, §11.1, §9.3): a moderator's decisions on disputed test
-- results, anti-fake actions on vacancies, invitations, companies and complaints, and the audit
-- log of every action and every view of a test recording.

CREATE TYPE review_decision AS ENUM ('confirm', 'annul', 'retest');

-- One final decision per finished session. `confirm` clears «требует подтверждения»; `annul`
-- voids the grade assignment the session made; `retest` voids it too and requires the camera for
-- the next test at that grade or above. The grade policy reads both voiding decisions as an
-- unsuccessful attempt at the moment of the decision (assessment::policy, Outcome::Annulled).
CREATE TABLE session_reviews (
    session_id    uuid PRIMARY KEY REFERENCES assessment_sessions (id) ON DELETE CASCADE,
    candidate_id  uuid            NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    decision      review_decision NOT NULL,
    -- Written for the candidate, who reads it in the test history.
    comment       text            NOT NULL CHECK (length(btrim(comment)) > 0),
    -- No FK: the decision outlives the moderator's account.
    moderator_id  uuid            NOT NULL,
    decided_at    timestamptz     NOT NULL DEFAULT now()
);
CREATE INDEX session_reviews_candidate_idx ON session_reviews (candidate_id);

-- A voided assignment stays in the history; the current category is the latest one without it.
ALTER TABLE grade_assignments ADD COLUMN annulled_at timestamptz;

-- The proctoring queue walks finished sessions oldest first.
CREATE INDEX assessment_sessions_finished_idx ON assessment_sessions (finished_at, id)
    WHERE status IN ('passed', 'failed');

-- A reviewed flag leaves the queue. `dismissed`: a false alarm, it stops counting against the
-- company; `upheld`: the moderator acted on it (took the vacancy down, froze the company), it
-- keeps counting for its 90 days. Flags that change on the next publish go back to the queue.
CREATE TYPE flag_review AS ENUM ('dismissed', 'upheld');
ALTER TABLE vacancies ADD COLUMN trust_review flag_review;
ALTER TABLE vacancies ADD COLUMN trust_reviewed_at timestamptz;
ALTER TABLE vacancies ADD CHECK ((trust_review IS NULL) = (trust_reviewed_at IS NULL));
-- Taken down by a moderator: closed, and publishing is refused until a moderator restores it.
ALTER TABLE vacancies ADD COLUMN unpublished_at timestamptz;
ALTER TABLE invitations ADD COLUMN trust_review flag_review;
ALTER TABLE invitations ADD COLUMN trust_reviewed_at timestamptz;
ALTER TABLE invitations ADD CHECK ((trust_review IS NULL) = (trust_reviewed_at IS NULL));
-- The company left the queue at this moment; a newer complaint, flag or bad decline brings it back.
ALTER TABLE companies ADD COLUMN trust_reviewed_at timestamptz;
-- Invitations frozen by a moderator: the weekly quota is 0 until `invitations_frozen_until`.
ALTER TABLE companies ADD COLUMN invitations_frozen_at timestamptz;
ALTER TABLE companies ADD COLUMN invitations_frozen_until timestamptz;

-- Complaint status: open while `resolved_at` is NULL. A dismissed complaint stops lowering the
-- company's quota; an upheld one (the moderator took its target down or froze the company) keeps
-- counting for its 90 days.
ALTER TABLE reports ADD COLUMN resolution text
    CHECK (resolution IN ('dismissed', 'vacancy_unpublished', 'invitations_frozen'));
ALTER TABLE reports ADD COLUMN resolution_comment text;
ALTER TABLE reports ADD COLUMN resolved_by uuid;
ALTER TABLE reports ADD CHECK ((resolved_at IS NULL) = (resolution IS NULL));
CREATE INDEX reports_open_idx ON reports (created_at, id) WHERE resolved_at IS NULL;

CREATE TYPE moderation_action AS ENUM (
    'confirm_result', 'annul_result', 'order_retest', 'view_recording',
    'dismiss', 'unpublish_vacancy', 'freeze_invitations', 'restore'
);
CREATE TYPE moderation_target AS ENUM ('session', 'vacancy', 'invitation', 'company', 'complaint');

-- Who did what to which object, when and why. No FKs, like contact_access_log: the log outlives
-- accounts. `candidate_id` and `company_id` name whom the action concerns: the candidate sees
-- decisions and recording views in the test history, the company sees outcomes on its trust page.
CREATE TABLE moderation_actions (
    id            uuid PRIMARY KEY,
    moderator_id  uuid              NOT NULL,
    action        moderation_action NOT NULL,
    target_kind   moderation_target NOT NULL,
    target_id     uuid              NOT NULL,
    candidate_id  uuid,
    company_id    uuid,
    comment       text,
    -- Specifics: freeze days and end, the grade voided, the complaint an action came from.
    details       jsonb             NOT NULL DEFAULT '{}' CHECK (jsonb_typeof(details) = 'object'),
    at            timestamptz       NOT NULL DEFAULT now(),
    CHECK (action = 'view_recording' OR length(btrim(coalesce(comment, ''))) > 0)
);
CREATE INDEX moderation_actions_at_idx ON moderation_actions (at DESC, id DESC);
CREATE INDEX moderation_actions_target_idx ON moderation_actions (target_id, at);
CREATE INDEX moderation_actions_candidate_idx ON moderation_actions (candidate_id, at)
    WHERE candidate_id IS NOT NULL;
CREATE INDEX moderation_actions_company_idx ON moderation_actions (company_id, at DESC)
    WHERE company_id IS NOT NULL;
