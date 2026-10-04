--! candidate_onboarding : (specialization?, grade?, active_session_id?)
SELECT c.public_code,
       c.full_name_enc IS NOT NULL                                            AS has_profile,
       EXISTS (SELECT 1 FROM survey_responses s WHERE s.candidate_id = c.user_id) AS has_survey,
       c.specialization,
       c.grade,
       (SELECT a.id FROM assessment_sessions a
         WHERE a.candidate_id = c.user_id AND a.status = 'in_progress')      AS active_session_id
FROM candidates c
WHERE c.user_id = :user_id;

--! company_by_owner
SELECT id, name FROM companies WHERE owner_user_id = :owner_user_id;

--! delete_candidate
DELETE FROM candidates WHERE user_id = :user_id;

--! delete_company_of
DELETE FROM companies WHERE owner_user_id = :owner_user_id;

--! hide_candidate
UPDATE candidates SET visible = false, updated_at = now() WHERE user_id = :user_id;
