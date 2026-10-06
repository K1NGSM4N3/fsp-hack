-- Lookups by candidate on tables keyed by something else (docs/solution.md §12.4).

-- The employer card counts the candidate's submitted and reviewed tasks; the candidate's task list,
-- expiry and the next-task pick filter by candidate too. Without it each card scanned the table:
-- 30 cards with 4 400 assignments took 7.2 ms instead of 1.7 ms.
CREATE INDEX task_assignments_candidate_idx ON task_assignments (candidate_id);

-- «Applied to this company» on every card shown to an employer, and the candidate's own
-- applications. The unique (vacancy_id, candidate_id) index serves only lookups by vacancy.
CREATE INDEX applications_candidate_idx ON applications (candidate_id);
