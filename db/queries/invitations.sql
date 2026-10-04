--! expire_stale_invitations
UPDATE invitations SET status = 'expired'
WHERE status IN ('sent', 'viewed') AND expires_at < now();

--! company_invitations_since : (count)
SELECT count(*) AS count FROM invitations WHERE company_id = :company_id AND created_at >= :since;

--! invitation_by_idempotency_key
SELECT id FROM invitations WHERE company_id = :company_id AND idempotency_key = :idempotency_key;

--! insert_invitation (need_id?, vacancy_id?, work_format?, idempotency_key?)
INSERT INTO invitations (id, company_id, candidate_id, need_id, vacancy_id, title, message, salary_min,
                         salary_max, work_format, contact, idempotency_key, expires_at)
VALUES (:id, :company_id, :candidate_id, :need_id, :vacancy_id, :title, :message, :salary_min,
        :salary_max, :work_format, :contact, :idempotency_key, :expires_at);

--! insert_invitation_event (actor_user_id?)
INSERT INTO invitation_events (id, invitation_id, status, actor_user_id)
VALUES (:id, :invitation_id, :status, :actor_user_id);

--! invitation_events
SELECT status, at FROM invitation_events WHERE invitation_id = :invitation_id ORDER BY at;

--! employer_invitations : (need_id?, vacancy_id?, work_format?, decline_reason?, decline_comment?, viewed_at?, responded_at?, specialization?, grade?)
SELECT i.id, i.candidate_id, c.public_code, c.specialization, c.grade, i.need_id, i.vacancy_id, i.title,
       i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status, i.decline_reason,
       i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at
FROM invitations i
JOIN candidates c ON c.user_id = i.candidate_id
WHERE i.company_id = :company_id
ORDER BY i.created_at DESC;

--! employer_invitation : (need_id?, vacancy_id?, work_format?, decline_reason?, decline_comment?, viewed_at?, responded_at?, specialization?, grade?)
SELECT i.id, i.candidate_id, c.public_code, c.specialization, c.grade, i.need_id, i.vacancy_id, i.title,
       i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status, i.decline_reason,
       i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at
FROM invitations i
JOIN candidates c ON c.user_id = i.candidate_id
WHERE i.company_id = :company_id AND i.id = :id;

--! candidate_invitations : (vacancy_id?, work_format?, decline_reason?, decline_comment?, viewed_at?, responded_at?, company_industry?, company_city?, company_website?)
SELECT i.id, i.company_id, co.name AS company_name, co.description AS company_description,
       co.industry AS company_industry, co.city AS company_city, co.website AS company_website,
       i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status,
       i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at
FROM invitations i
JOIN companies co ON co.id = i.company_id
WHERE i.candidate_id = :candidate_id
ORDER BY i.created_at DESC;

--! candidate_invitation : (vacancy_id?, work_format?, decline_reason?, decline_comment?, viewed_at?, responded_at?, company_industry?, company_city?, company_website?)
SELECT i.id, i.company_id, co.name AS company_name, co.description AS company_description,
       co.industry AS company_industry, co.city AS company_city, co.website AS company_website,
       i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format, i.contact, i.status,
       i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at
FROM invitations i
JOIN companies co ON co.id = i.company_id
WHERE i.candidate_id = :candidate_id AND i.id = :id;

--! mark_invitation_viewed
UPDATE invitations SET status = 'viewed', viewed_at = now()
WHERE id = :id AND candidate_id = :candidate_id AND status = 'sent';

--! respond_invitation (decline_reason?, decline_comment?)
UPDATE invitations
SET status = :status, responded_at = now(), decline_reason = :decline_reason, decline_comment = :decline_comment
WHERE id = :id AND candidate_id = :candidate_id AND status IN ('sent', 'viewed') AND expires_at >= now();

--! withdraw_invitation
UPDATE invitations SET status = 'withdrawn', responded_at = now()
WHERE id = :id AND company_id = :company_id AND status IN ('sent', 'viewed');

--! company_invitation_status_counts : (count)
SELECT status, count(*) AS count FROM invitations WHERE company_id = :company_id GROUP BY status;

--! company_decline_reasons : (reason?, count)
SELECT decline_reason AS reason, count(*) AS count
FROM invitations
WHERE company_id = :company_id AND status = 'declined'
GROUP BY decline_reason
ORDER BY count DESC;

--! company_response_rate : (answered, total)
SELECT count(*) FILTER (WHERE status IN ('accepted', 'declined')) AS answered, count(*) AS total
FROM invitations
WHERE company_id = :company_id AND status <> 'withdrawn';
