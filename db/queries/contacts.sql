--! accepted_invitation_basis
SELECT id FROM invitations
WHERE company_id = :company_id AND candidate_id = :candidate_id AND status = 'accepted'
ORDER BY responded_at DESC
LIMIT 1;

--! application_basis
SELECT a.id FROM applications a
JOIN vacancies v ON v.id = a.vacancy_id
WHERE v.company_id = :company_id AND a.candidate_id = :candidate_id AND a.status <> 'withdrawn'
ORDER BY a.created_at DESC
LIMIT 1;

--! candidate_secrets : (full_name_enc?, contacts_enc?)
SELECT full_name_enc, contacts_enc FROM candidates WHERE user_id = :user_id;

--! insert_contact_access (ip?, user_agent?)
INSERT INTO contact_access_log (id, employer_user_id, company_id, candidate_id, basis, basis_id, ip, user_agent)
VALUES (:id, :employer_user_id, :company_id, :candidate_id, :basis, :basis_id, :ip, :user_agent);
