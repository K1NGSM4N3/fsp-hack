-- Indexes justified by EXPLAIN (ANALYZE) on 10 000 candidates, 300 companies, 45 000 invitations,
-- 90 000 profile views (docs/solution.md, «Производительность и кеширование»).

-- Semi-joins "has an active grant of document kind X" (search pool, category cards, salary
-- check): an index-only scan of the document's holders instead of a pass over every grant.
CREATE INDEX user_consents_document_idx ON user_consents (document_id, user_id) WHERE revoked_at IS NULL;

-- Invitation counter on every need read (shortlist, search by need, the needs list).
CREATE INDEX invitations_need_idx ON invitations (need_id) WHERE need_id IS NOT NULL;

-- Lazy expiry runs before every invitation read: only live invitations past their deadline.
CREATE INDEX invitations_expiry_idx ON invitations (expires_at) WHERE status IN ('sent', 'viewed');

-- "Companies that viewed you" in the candidate's market stats; the primary key leads with company.
CREATE INDEX profile_views_candidate_idx ON profile_views (candidate_id, viewed_on);
