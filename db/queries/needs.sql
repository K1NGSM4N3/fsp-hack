--! insert_need (city?)
INSERT INTO hiring_needs (id, company_id, title, description, specialization, grades, allow_adjacent_grades,
                          must_skills, nice_skills, salary_min, salary_max, work_formats, city, industries)
VALUES (:id, :company_id, :title, :description, :specialization, :grades, :allow_adjacent_grades,
        :must_skills, :nice_skills, :salary_min, :salary_max, :work_formats, :city, :industries);

--! update_need (city?)
UPDATE hiring_needs
SET title = :title, description = :description, specialization = :specialization, grades = :grades,
    allow_adjacent_grades = :allow_adjacent_grades, must_skills = :must_skills, nice_skills = :nice_skills,
    salary_min = :salary_min, salary_max = :salary_max, work_formats = :work_formats, city = :city,
    industries = :industries, status = :status, updated_at = now()
WHERE id = :id AND company_id = :company_id;

--! company_needs : (city?, shortlist_generated_at?)
SELECT n.id, n.title, n.description, n.specialization, n.grades, n.allow_adjacent_grades, n.must_skills,
       n.nice_skills, n.salary_min, n.salary_max, n.work_formats, n.city, n.industries, n.filters, n.status,
       n.created_at, n.updated_at, n.shortlist_generated_at,
       (SELECT count(*) FROM shortlist_items s WHERE s.need_id = n.id AND s.in_current AND NOT s.dismissed) AS shortlisted,
       (SELECT count(*) FROM invitations i WHERE i.need_id = n.id) AS invited
FROM hiring_needs n
WHERE n.company_id = :company_id
ORDER BY n.created_at DESC;

--! company_need : (city?, shortlist_generated_at?)
SELECT n.id, n.title, n.description, n.specialization, n.grades, n.allow_adjacent_grades, n.must_skills,
       n.nice_skills, n.salary_min, n.salary_max, n.work_formats, n.city, n.industries, n.filters, n.status,
       n.created_at, n.updated_at, n.shortlist_generated_at,
       (SELECT count(*) FROM shortlist_items s WHERE s.need_id = n.id AND s.in_current AND NOT s.dismissed) AS shortlisted,
       (SELECT count(*) FROM invitations i WHERE i.need_id = n.id) AS invited
FROM hiring_needs n
WHERE n.company_id = :company_id AND n.id = :id;

--! delete_need
DELETE FROM hiring_needs WHERE id = :id AND company_id = :company_id;

--! start_shortlist_generation : (previous?)
UPDATE hiring_needs n
SET filters = :filters, shortlist_generated_at = now()
FROM (SELECT shortlist_generated_at AS previous FROM hiring_needs WHERE id = :id FOR UPDATE) prev
WHERE n.id = :id
RETURNING prev.previous, n.shortlist_generated_at AS generated_at;

--! mark_shortlist_stale
UPDATE shortlist_items SET in_current = false, updated_at = now()
WHERE need_id = :need_id AND NOT (candidate_id = ANY (:keep));

--! upsert_shortlist_item
INSERT INTO shortlist_items (need_id, candidate_id, score, rank, breakdown, in_current)
VALUES (:need_id, :candidate_id, :score, :rank, :breakdown, true)
ON CONFLICT (need_id, candidate_id) DO UPDATE
SET score = EXCLUDED.score, rank = EXCLUDED.rank, breakdown = EXCLUDED.breakdown, in_current = true, updated_at = now();

--! shortlist_items
SELECT candidate_id, score, rank, breakdown, in_current, pinned, dismissed, first_seen_at
FROM shortlist_items
WHERE need_id = :need_id
ORDER BY in_current DESC, rank;

--! patch_shortlist_item (pinned?, dismissed?)
UPDATE shortlist_items s
SET pinned = coalesce(:pinned, s.pinned), dismissed = coalesce(:dismissed, s.dismissed), updated_at = now()
FROM hiring_needs n
WHERE s.need_id = n.id AND n.company_id = :company_id AND s.need_id = :need_id AND s.candidate_id = :candidate_id;
