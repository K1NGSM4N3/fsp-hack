--! insert_candidate
INSERT INTO candidates (user_id, public_code) VALUES (:user_id, :public_code);

--! candidate_profile : (full_name_enc?, contacts_enc?, city?, salary_expectation_min?, experience_years?, specialization?, grade?, theta?, category_assigned_at?)
SELECT user_id, public_code, full_name_enc, contacts_enc, city, relocation, work_formats,
       salary_expectation_min, about, experience_years, roles, soft_skills, industries, experience,
       search_status, visible, show_name, show_fsp, show_experience,
       specialization, grade, theta, category_assigned_at, last_activity_at, updated_at
FROM candidates
WHERE user_id = :user_id;

--! update_candidate_profile (full_name_enc?, contacts_enc?, city?, salary_expectation_min?, experience_years?)
UPDATE candidates
SET full_name_enc = :full_name_enc, contacts_enc = :contacts_enc, city = :city, relocation = :relocation,
    work_formats = :work_formats, salary_expectation_min = :salary_expectation_min, about = :about,
    experience_years = :experience_years, roles = :roles, soft_skills = :soft_skills,
    industries = :industries, experience = :experience, updated_at = now(), last_activity_at = now()
WHERE user_id = :user_id;

--! update_candidate_privacy
UPDATE candidates
SET visible = :visible, show_name = :show_name, show_fsp = :show_fsp, show_experience = :show_experience,
    search_status = :search_status, updated_at = now()
WHERE user_id = :user_id;

--! candidate_skills
SELECT skill, source FROM candidate_skills WHERE candidate_id = :candidate_id ORDER BY skill, source;

--! delete_candidate_skills_by_source
DELETE FROM candidate_skills WHERE candidate_id = :candidate_id AND source = :source;

--! insert_candidate_skill
INSERT INTO candidate_skills (candidate_id, skill, source) VALUES (:candidate_id, :skill, :source)
ON CONFLICT DO NOTHING;

--! insert_resume_upload
INSERT INTO resume_uploads (id, candidate_id, filename, size_bytes, recognized_fields)
VALUES (:id, :candidate_id, :filename, :size_bytes, :recognized_fields);

--! touch_candidate_activity
UPDATE candidates SET last_activity_at = now() WHERE user_id = :user_id;

--! candidate_invitable : (specialization?, grade?)
SELECT visible, search_status, specialization, grade FROM candidates WHERE user_id = :user_id;

--! set_search_status
UPDATE candidates SET search_status = :search_status, updated_at = now() WHERE user_id = :user_id;
