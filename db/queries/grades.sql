--! insert_grade_assignment (session_id?)
INSERT INTO grade_assignments (id, candidate_id, specialization, grade, theta, se, session_id, reason, assigned_at)
VALUES (:id, :candidate_id, :specialization, :grade, :theta, :se, :session_id, :reason, :assigned_at);

--! set_candidate_category
UPDATE candidates
SET specialization = :specialization, grade = :grade, theta = :theta, category_assigned_at = :assigned_at,
    last_activity_at = greatest(last_activity_at, :assigned_at), updated_at = now()
WHERE user_id = :candidate_id;

--! grade_history : (session_id?)
SELECT id, specialization, grade, theta, se, session_id, reason, assigned_at
FROM grade_assignments
WHERE candidate_id = :candidate_id
ORDER BY assigned_at DESC;
