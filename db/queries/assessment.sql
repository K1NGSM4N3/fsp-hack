--! expire_abandoned_sessions
UPDATE assessment_sessions SET status = 'expired', finished_at = now()
WHERE candidate_id = :candidate_id AND status = 'in_progress'
  AND coalesce(current_item_deadline, started_at) < now() - interval '30 minutes';

--! active_session
SELECT id FROM assessment_sessions WHERE candidate_id = :candidate_id AND status = 'in_progress';

--! insert_session
INSERT INTO assessment_sessions (id, candidate_id, specialization, claimed_grade, attempt_kind, state,
                                 current_item, current_item_deadline, current_item_shown_at)
VALUES (:id, :candidate_id, :specialization, :claimed_grade, :attempt_kind, :state,
        :current_item, :deadline, :shown_at);

--! session_for_update : (current_item?, current_item_deadline?, current_item_shown_at?)
SELECT id, specialization, claimed_grade, attempt_kind, status, state, current_item, current_item_deadline,
       current_item_shown_at, items_count
FROM assessment_sessions
WHERE id = :id AND candidate_id = :candidate_id
FOR UPDATE;

--! update_session_progress
UPDATE assessment_sessions
SET state = :state, current_item = :current_item, current_item_deadline = :deadline,
    current_item_shown_at = :shown_at, items_count = :items_count
WHERE id = :id;

--! finish_session (measured_grade?)
UPDATE assessment_sessions
SET status = :status, state = :state, current_item = NULL, current_item_deadline = NULL,
    theta = :theta, se = :se, measured_grade = :measured_grade, confident_above_next = :confident_above_next,
    person_fit_lz = :person_fit_lz, aberrant = :aberrant, items_count = :items_count, finished_at = :finished_at
WHERE id = :id;

--! insert_response (answer?)
INSERT INTO assessment_responses (session_id, seq, template_id, level, seed, answer, correct, timed_out,
                                  elapsed_ms, theta_after, se_after, answered_at)
VALUES (:session_id, :seq, :template_id, :level, :seed, :answer, :correct, :timed_out, :elapsed_ms,
        :theta_after, :se_after, :answered_at);

--! candidate_sessions : (theta?, se?, measured_grade?, confident_above_next?, person_fit_lz?, aberrant?, finished_at?)
SELECT s.id, s.specialization, s.claimed_grade, s.attempt_kind, s.status, s.theta, s.se, s.measured_grade,
       s.confident_above_next, s.person_fit_lz, s.aberrant, s.items_count, s.started_at, s.finished_at,
       EXISTS (SELECT 1 FROM grade_assignments g WHERE g.session_id = s.id) AS assigned
FROM assessment_sessions s
WHERE s.candidate_id = :candidate_id
ORDER BY s.started_at DESC;

--! session_state : (theta?, se?, measured_grade?, confident_above_next?, person_fit_lz?, aberrant?, finished_at?, current_item?, current_item_deadline?)
SELECT s.id, s.specialization, s.claimed_grade, s.attempt_kind, s.status, s.state, s.current_item,
       s.current_item_deadline, s.theta, s.se, s.measured_grade, s.confident_above_next, s.person_fit_lz,
       s.aberrant, s.items_count, s.started_at, s.finished_at,
       EXISTS (SELECT 1 FROM grade_assignments g WHERE g.session_id = s.id) AS assigned
FROM assessment_sessions s
WHERE s.id = :id AND s.candidate_id = :candidate_id;
