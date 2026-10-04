--! insert_task
INSERT INTO tasks (id, company_id, specialization, grades, skills, title, body, kind)
VALUES (:id, :company_id, :specialization, :grades, :skills, :title, :body, :kind);

--! company_tasks
SELECT t.id, t.specialization, t.grades, t.skills, t.title, t.body, t.kind, t.active, t.created_at,
       count(a.id) FILTER (WHERE a.status IN ('submitted', 'reviewed')) AS submissions,
       count(a.id) FILTER (WHERE a.status = 'reviewed') AS reviewed
FROM tasks t
LEFT JOIN task_assignments a ON a.task_id = t.id
WHERE t.company_id = :company_id
GROUP BY t.id
ORDER BY t.created_at DESC;

--! set_task_active
UPDATE tasks SET active = :active WHERE id = :id AND company_id = :company_id;

--! task_submissions : (answer?, rating?, review_comment?, submitted_at?, specialization?, grade?)
SELECT a.id, a.candidate_id, c.public_code, c.specialization, c.grade, a.status, a.answer, a.rating,
       a.review_comment, a.submitted_at
FROM task_assignments a
JOIN tasks t ON t.id = a.task_id
JOIN candidates c ON c.user_id = a.candidate_id
WHERE t.id = :task_id AND t.company_id = :company_id AND a.status IN ('submitted', 'reviewed')
ORDER BY a.submitted_at DESC;

--! review_submission (review_comment?)
UPDATE task_assignments a
SET status = 'reviewed', rating = :rating, review_comment = :review_comment, reviewed_at = now()
FROM tasks t
WHERE a.id = :id AND a.task_id = t.id AND t.id = :task_id AND t.company_id = :company_id
  AND a.status IN ('submitted', 'reviewed');

--! expire_assignments
UPDATE task_assignments SET status = 'expired'
WHERE candidate_id = :candidate_id AND status = 'assigned' AND due_at < now();

--! last_assignment_at : (assigned_at?)
SELECT max(assigned_at) AS assigned_at FROM task_assignments WHERE candidate_id = :candidate_id;

--! has_open_assignment
SELECT count(*) AS open FROM task_assignments WHERE candidate_id = :candidate_id AND status = 'assigned';

-- Oldest active task for the candidate's category that they have not seen yet; skills overlap first.
--! pick_task_for
SELECT t.id
FROM tasks t, candidates c
WHERE c.user_id = :candidate_id
  AND t.active
  AND t.specialization = c.specialization
  AND c.grade = ANY (t.grades)
  AND NOT EXISTS (SELECT 1 FROM task_assignments a WHERE a.task_id = t.id AND a.candidate_id = c.user_id)
ORDER BY (SELECT count(*) FROM candidate_skills s WHERE s.candidate_id = c.user_id AND s.skill = ANY (t.skills)) DESC,
         t.created_at
LIMIT 1;

--! insert_assignment
INSERT INTO task_assignments (id, task_id, candidate_id, due_at) VALUES (:id, :task_id, :candidate_id, :due_at);

--! candidate_assignments : (company_name?, answer?, rating?, review_comment?, submitted_at?, reviewed_at?)
SELECT a.id, t.title, t.body, t.kind, t.skills, co.name AS company_name, a.status, a.answer, a.rating,
       a.review_comment, a.assigned_at, a.due_at, a.submitted_at, a.reviewed_at
FROM task_assignments a
JOIN tasks t ON t.id = a.task_id
LEFT JOIN companies co ON co.id = t.company_id
WHERE a.candidate_id = :candidate_id
ORDER BY a.assigned_at DESC;

--! submit_assignment
UPDATE task_assignments SET status = 'submitted', answer = :answer, submitted_at = now()
WHERE id = :id AND candidate_id = :candidate_id AND status = 'assigned' AND due_at >= now();

--! skip_assignment
UPDATE task_assignments SET status = 'skipped'
WHERE id = :id AND candidate_id = :candidate_id AND status = 'assigned';
