--! insert_survey (experience_years?)
INSERT INTO survey_responses (id, candidate_id, direction, specialization, claimed_grade, skills, industries,
                              experience_years, self_assessment)
VALUES (:id, :candidate_id, :direction, :specialization, :claimed_grade, :skills, :industries,
        :experience_years, :self_assessment);

--! survey_history : (experience_years?)
SELECT id, direction, specialization, claimed_grade, skills, industries, experience_years, self_assessment, created_at
FROM survey_responses
WHERE candidate_id = :candidate_id
ORDER BY created_at DESC;

--! latest_survey : (experience_years?)
SELECT id, direction, specialization, claimed_grade, skills, industries, experience_years, self_assessment, created_at
FROM survey_responses
WHERE candidate_id = :candidate_id
ORDER BY created_at DESC
LIMIT 1;
