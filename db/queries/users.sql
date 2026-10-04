--! insert_user
INSERT INTO users (id, role, email_enc, email_bidx, password_hash)
VALUES (:id, :role, :email_enc, :email_bidx, :password_hash);

--! user_by_email_bidx : (email_verified_at?, deleted_at?)
SELECT id, role, email_enc, password_hash, email_verified_at, deleted_at
FROM users
WHERE email_bidx = :email_bidx;

--! user_by_id : (email_verified_at?, deleted_at?)
SELECT id, role, email_enc, password_hash, email_verified_at, deleted_at
FROM users
WHERE id = :id;

--! touch_last_login
UPDATE users SET last_login_at = now() WHERE id = :id;

--! mark_email_verified
UPDATE users SET email_verified_at = now() WHERE id = :id AND email_verified_at IS NULL;

--! update_password_hash
UPDATE users SET password_hash = :password_hash WHERE id = :id;

-- Account deletion (152-ФЗ): wipe personal data, keep an anonymized row for referential history.
--! anonymize_user
UPDATE users
SET email_enc = '\x'::bytea,
    email_bidx = :tombstone_bidx,
    password_hash = '!',
    deleted_at = now()
WHERE id = :id;

--! insert_email_verification
INSERT INTO email_verifications (token_hash, user_id, expires_at)
VALUES (:token_hash, :user_id, :expires_at);

--! consume_email_verification : (user_id)
UPDATE email_verifications
SET used_at = now()
WHERE token_hash = :token_hash AND used_at IS NULL AND expires_at > now()
RETURNING user_id;

--! insert_refresh_token
INSERT INTO refresh_tokens (id, user_id, family_id, token_hash, expires_at)
VALUES (:id, :user_id, :family_id, :token_hash, :expires_at);

--! refresh_token_by_hash : (rotated_at?, revoked_at?)
SELECT id, user_id, family_id, expires_at, rotated_at, revoked_at
FROM refresh_tokens
WHERE token_hash = :token_hash;

-- Atomic rotation: only one concurrent refresh wins.
--! rotate_refresh_token
UPDATE refresh_tokens SET rotated_at = now()
WHERE id = :id AND rotated_at IS NULL AND revoked_at IS NULL;

--! revoke_refresh_family
UPDATE refresh_tokens SET revoked_at = now()
WHERE family_id = :family_id AND revoked_at IS NULL;

--! revoke_user_refresh_tokens
UPDATE refresh_tokens SET revoked_at = now()
WHERE user_id = :user_id AND revoked_at IS NULL;
