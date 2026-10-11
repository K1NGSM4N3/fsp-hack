-- One definition of a published candidate profile: visible, with an unrevoked publication
-- consent and, while under 18, an unrevoked consent of the legal representative
-- (architecture.md §9.2, §9.5). The search pool, the category cards, the demand statistics,
-- the anonymous card and the market salary band read this view instead of repeating the rule.
-- A plain filtering view is inlined by the planner, so the consent checks stay semi-joins on
-- user_consents_document_idx as when they were written out in every query. `c.*` is expanded
-- when the view is created: a migration that adds a column queries need here recreates the view.
CREATE VIEW published_candidates AS
SELECT c.*
FROM candidates c
WHERE c.visible
  AND EXISTS (SELECT 1 FROM user_consents uc
               JOIN consent_documents d ON d.id = uc.document_id AND d.kind = 'pd_publication'
               WHERE uc.user_id = c.user_id AND uc.revoked_at IS NULL)
  AND (c.birth_date IS NULL OR c.birth_date + interval '18 years' <= current_date
       OR EXISTS (SELECT 1 FROM user_consents gc
                   JOIN consent_documents gd ON gd.id = gc.document_id AND gd.kind = 'guardian_consent'
                   WHERE gc.user_id = c.user_id AND gc.revoked_at IS NULL));
