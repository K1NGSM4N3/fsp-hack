--! insert_oidc_state
INSERT INTO oidc_states (state, user_id, pkce_verifier, nonce) VALUES (:state, :user_id, :pkce_verifier, :nonce);

--! take_oidc_state
DELETE FROM oidc_states WHERE state = :state RETURNING user_id, pkce_verifier, nonce, created_at;

--! purge_oidc_states
DELETE FROM oidc_states WHERE created_at < now() - interval '15 minutes';

--! fsp_link : (last_sync_at?)
SELECT candidate_id, fsp_id, linked_via, linked_at, last_sync_at, achievements, has_achievements
FROM fsp_links
WHERE candidate_id = :candidate_id;

--! fsp_link_owner
SELECT candidate_id FROM fsp_links WHERE fsp_id = :fsp_id;

--! upsert_fsp_link
INSERT INTO fsp_links (candidate_id, fsp_id, linked_via)
VALUES (:candidate_id, :fsp_id, :linked_via)
ON CONFLICT (candidate_id) DO UPDATE
SET fsp_id = EXCLUDED.fsp_id, linked_via = EXCLUDED.linked_via, linked_at = now(),
    achievements = '[]', last_sync_at = NULL;

--! store_fsp_achievements
UPDATE fsp_links SET achievements = :achievements, last_sync_at = now() WHERE candidate_id = :candidate_id;

--! delete_fsp_link
DELETE FROM fsp_links WHERE candidate_id = :candidate_id;
