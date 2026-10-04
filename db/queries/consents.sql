--! current_consent_documents
SELECT DISTINCT ON (kind) id, kind, version, title, body, required_for, published_at
FROM consent_documents
ORDER BY kind, version DESC;

--! insert_user_consent (ip?, user_agent?)
INSERT INTO user_consents (id, user_id, document_id, ip, user_agent)
VALUES (:id, :user_id, :document_id, :ip, :user_agent);

--! user_consents : (revoked_at?)
SELECT uc.id, uc.document_id, d.kind, d.version, d.title, uc.granted_at, uc.revoked_at
FROM user_consents uc
JOIN consent_documents d ON d.id = uc.document_id
WHERE uc.user_id = :user_id
ORDER BY uc.granted_at DESC;

--! revoke_user_consent : (kind)
UPDATE user_consents uc
SET revoked_at = now()
FROM consent_documents d
WHERE uc.id = :id AND uc.user_id = :user_id AND uc.revoked_at IS NULL AND d.id = uc.document_id
RETURNING d.kind;
