--! insert_vacancy (city?, published_at?)
INSERT INTO vacancies (id, company_id, title, description, specialization, grade, skills, salary_min, salary_max,
                       work_format, city, status, published_at)
VALUES (:id, :company_id, :title, :description, :specialization, :grade, :skills, :salary_min, :salary_max,
        :work_format, :city, :status, :published_at);

--! update_vacancy (city?)
UPDATE vacancies
SET title = :title, description = :description, specialization = :specialization, grade = :grade,
    skills = :skills, salary_min = :salary_min, salary_max = :salary_max, work_format = :work_format,
    city = :city, status = :status,
    published_at = CASE WHEN :status = 'published'::vacancy_status AND published_at IS NULL THEN now() ELSE published_at END,
    updated_at = now()
WHERE id = :id AND company_id = :company_id;

--! company_vacancies : (city?, published_at?)
SELECT v.id, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max,
       v.work_format, v.city, v.status, v.published_at, v.created_at, v.updated_at,
       (SELECT count(*) FROM applications a WHERE a.vacancy_id = v.id AND a.status <> 'withdrawn') AS applications
FROM vacancies v
WHERE v.company_id = :company_id
ORDER BY v.created_at DESC;

--! company_vacancy : (city?, published_at?)
SELECT v.id, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max,
       v.work_format, v.city, v.status, v.published_at, v.created_at, v.updated_at,
       (SELECT count(*) FROM applications a WHERE a.vacancy_id = v.id AND a.status <> 'withdrawn') AS applications
FROM vacancies v
WHERE v.company_id = :company_id AND v.id = :id;

--! published_vacancies (specialization?, grade?, work_format?, q?, salary_from?) : (city?, published_at?, company_city?)
SELECT v.id, v.company_id, co.name AS company_name, co.city AS company_city, v.title, v.description,
       v.specialization, v.grade, v.skills, v.salary_min, v.salary_max, v.work_format, v.city, v.published_at,
       count(*) OVER () AS total
FROM vacancies v
JOIN companies co ON co.id = v.company_id
WHERE v.status = 'published'
  AND (:specialization::text IS NULL OR v.specialization = :specialization)
  AND (:grade::text IS NULL OR v.grade = :grade)
  AND (:work_format::text IS NULL OR v.work_format = :work_format)
  AND (:salary_from::bigint IS NULL OR v.salary_max >= :salary_from)
  AND (:q::text IS NULL OR v.search_tsv @@ websearch_to_tsquery('russian', :q))
ORDER BY v.published_at DESC
LIMIT :limit OFFSET :offset;

--! published_vacancy : (city?, published_at?, company_city?)
SELECT v.id, v.company_id, co.name AS company_name, co.city AS company_city, co.description AS company_description,
       v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max, v.work_format,
       v.city, v.published_at
FROM vacancies v
JOIN companies co ON co.id = v.company_id
WHERE v.id = :id AND v.status = 'published';

--! insert_application
INSERT INTO applications (id, vacancy_id, candidate_id, cover_letter)
VALUES (:id, :vacancy_id, :candidate_id, :cover_letter);

--! reopen_application
UPDATE applications SET status = 'sent', cover_letter = :cover_letter, updated_at = now()
WHERE vacancy_id = :vacancy_id AND candidate_id = :candidate_id AND status = 'withdrawn'
RETURNING id;

--! candidate_applications
SELECT a.id, a.vacancy_id, v.title AS vacancy_title, co.name AS company_name, v.salary_min, v.salary_max,
       a.cover_letter, a.status, a.created_at, a.updated_at
FROM applications a
JOIN vacancies v ON v.id = a.vacancy_id
JOIN companies co ON co.id = v.company_id
WHERE a.candidate_id = :candidate_id
ORDER BY a.created_at DESC;

--! withdraw_application
UPDATE applications SET status = 'withdrawn', updated_at = now()
WHERE id = :id AND candidate_id = :candidate_id AND status IN ('sent', 'viewed');

--! vacancy_applications : (specialization?, grade?, theta?)
SELECT a.id, a.candidate_id, c.public_code, c.specialization, c.grade, c.theta, a.cover_letter, a.status,
       a.created_at, a.updated_at
FROM applications a
JOIN vacancies v ON v.id = a.vacancy_id
JOIN candidates c ON c.user_id = a.candidate_id
WHERE v.id = :vacancy_id AND v.company_id = :company_id AND a.status <> 'withdrawn'
ORDER BY a.created_at DESC;

--! set_application_status
UPDATE applications a SET status = :status, updated_at = now()
FROM vacancies v
WHERE a.id = :id AND v.id = a.vacancy_id AND v.company_id = :company_id AND a.status NOT IN ('withdrawn');
