--! category_overview : (median_expectation?)
SELECT c.specialization, c.grade,
       count(*) AS candidates,
       count(*) FILTER (WHERE f.has_achievements AND c.show_fsp) AS with_fsp,
       count(*) FILTER (WHERE c.search_status = 'active') AS actively_looking,
       (percentile_cont(0.5) WITHIN GROUP (ORDER BY c.salary_expectation_min))::bigint AS median_expectation
FROM candidates c
LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
WHERE c.visible AND c.search_status <> 'closed' AND c.specialization IS NOT NULL AND c.grade IS NOT NULL
GROUP BY c.specialization, c.grade;

--! candidate_market : (median_salary_min?, median_salary_max?)
SELECT
    (SELECT count(DISTINCT pv.company_id) FROM profile_views pv
      WHERE pv.candidate_id = :candidate_id AND pv.viewed_on >= current_date - 30) AS companies_viewed_you,
    (SELECT count(*) FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
      WHERE ic.specialization = :specialization AND ic.grade = :grade AND i.created_at >= now() - interval '30 days') AS invitations_in_category,
    (SELECT count(*) FROM hiring_needs n
      WHERE n.status = 'active' AND n.specialization = :specialization AND :grade = ANY (n.grades)) AS open_needs,
    (SELECT count(*) FROM candidates oc
      WHERE oc.specialization = :specialization AND oc.grade = :grade AND oc.visible AND oc.search_status <> 'closed') AS peers,
    (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_min))::bigint FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
      WHERE ic.specialization = :specialization AND ic.grade = :grade AND i.created_at >= now() - interval '90 days') AS median_salary_min,
    (SELECT (percentile_cont(0.5) WITHIN GROUP (ORDER BY i.salary_max))::bigint FROM invitations i JOIN candidates ic ON ic.user_id = i.candidate_id
      WHERE ic.specialization = :specialization AND ic.grade = :grade AND i.created_at >= now() - interval '90 days') AS median_salary_max;

--! record_profile_view
INSERT INTO profile_views (company_id, candidate_id) VALUES (:company_id, :candidate_id) ON CONFLICT DO NOTHING;
