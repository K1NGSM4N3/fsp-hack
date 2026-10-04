--! demo_seeded : (count)
SELECT count(*) AS count FROM companies;

--! seed_verified_user
INSERT INTO users (id, role, email_enc, email_bidx, password_hash, email_verified_at, created_at)
VALUES (:id, :role, :email_enc, :email_bidx, :password_hash, :created_at, :created_at);

--! seed_candidate (salary_expectation_min?, experience_years?, city?)
INSERT INTO candidates (user_id, public_code, full_name_enc, contacts_enc, city, relocation, work_formats,
                        salary_expectation_min, about, experience_years, roles, soft_skills, industries, experience,
                        search_status, show_name, last_activity_at, created_at)
VALUES (:user_id, :public_code, :full_name_enc, :contacts_enc, :city, :relocation, :work_formats,
        :salary_expectation_min, :about, :experience_years, :roles, :soft_skills, :industries, :experience,
        :search_status, :show_name, :last_activity_at, :created_at);

--! seed_company
INSERT INTO companies (id, owner_user_id, name, description, industry, city, size, contact_email, contact_telegram)
VALUES (:id, :owner_user_id, :name, :description, :industry, :city, :size, :contact_email, :contact_telegram);

--! seed_fsp_link
INSERT INTO fsp_links (candidate_id, fsp_id, linked_via, last_sync_at, achievements)
VALUES (:candidate_id, :fsp_id, 'seed', now(), :achievements);

--! seed_session (measured_grade?)
INSERT INTO assessment_sessions (id, candidate_id, specialization, claimed_grade, attempt_kind, status, state,
                                 theta, se, measured_grade, confident_above_next, person_fit_lz, aberrant,
                                 items_count, started_at, finished_at)
VALUES (:id, :candidate_id, :specialization, :claimed_grade, :attempt_kind, :status, :state,
        :theta, :se, :measured_grade, :confident_above_next, :person_fit_lz, :aberrant,
        :items_count, :started_at, :finished_at);
