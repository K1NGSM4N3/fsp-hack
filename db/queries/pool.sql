-- Searchable candidates of one specialization with everything the matching engine needs.
-- `skills`/`skill_sources` are parallel arrays; `text_rank` scores the "about" text against an
-- OR-query built from the need description.
--! candidate_pool (grade?, text_query?) : (theta?, salary_expectation_min?, city?, achievements?)
SELECT c.user_id, c.public_code, c.specialization, c.grade, c.theta, c.salary_expectation_min,
       c.work_formats, c.city, c.relocation, c.last_activity_at, c.industries, c.show_fsp,
       f.achievements,
       coalesce(array_agg(s.skill ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skills,
       coalesce(array_agg(s.source ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skill_sources,
       CASE WHEN :text_query::text IS NULL THEN 0::real
            ELSE ts_rank_cd(c.about_tsv, to_tsquery('russian', :text_query))::real END AS text_rank
FROM candidates c
JOIN users u ON u.id = c.user_id AND u.deleted_at IS NULL
LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
LEFT JOIN candidate_skills s ON s.candidate_id = c.user_id
WHERE c.visible
  AND c.search_status <> 'closed'
  AND c.specialization = :specialization
  AND c.grade IS NOT NULL
  AND (:grade::text IS NULL OR c.grade = :grade)
GROUP BY c.user_id, f.achievements;

--! candidate_cards : (full_name_enc?, city?, salary_expectation_min?, experience_years?, specialization?, grade?, theta?, category_assigned_at?, achievements?, task_rating?)
SELECT c.user_id, c.public_code, c.full_name_enc, c.show_name, c.show_fsp, c.show_experience, c.city, c.relocation,
       c.work_formats, c.salary_expectation_min, c.about, c.experience_years, c.roles, c.soft_skills, c.industries,
       c.experience, c.search_status, c.specialization, c.grade, c.theta, c.category_assigned_at, c.last_activity_at,
       c.visible, f.achievements,
       coalesce(array_agg(s.skill ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skills,
       coalesce(array_agg(s.source ORDER BY s.skill, s.source) FILTER (WHERE s.skill IS NOT NULL), '{}') AS skill_sources,
       (SELECT avg(a.rating)::real FROM task_assignments a WHERE a.candidate_id = c.user_id AND a.status = 'reviewed') AS task_rating,
       (SELECT count(*) FROM task_assignments a WHERE a.candidate_id = c.user_id AND a.status IN ('submitted', 'reviewed')) AS tasks_done
FROM candidates c
LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
LEFT JOIN candidate_skills s ON s.candidate_id = c.user_id
WHERE c.user_id = ANY (:ids)
GROUP BY c.user_id, f.achievements;

--! company_relations : (invitation_status?)
SELECT c.user_id AS candidate_id,
       (SELECT i.status FROM invitations i WHERE i.company_id = :company_id AND i.candidate_id = c.user_id
         ORDER BY i.created_at DESC LIMIT 1) AS invitation_status,
       EXISTS (SELECT 1 FROM applications a JOIN vacancies v ON v.id = a.vacancy_id
                WHERE v.company_id = :company_id AND a.candidate_id = c.user_id AND a.status <> 'withdrawn') AS applied
FROM candidates c
WHERE c.user_id = ANY (:ids);
