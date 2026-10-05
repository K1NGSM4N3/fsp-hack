-- Consent gating (152-ФЗ): publishing a profile needs active pd_processing and pd_publication
-- grants, and a document can be granted again from the cabinet.

-- required_for lists what registration insists on; applies_to says who may grant the document at
-- all. The video consent is optional (not required) but still only meaningful for candidates.
ALTER TABLE consent_documents ADD COLUMN applies_to user_role[];
UPDATE consent_documents
SET applies_to = CASE WHEN kind = 'proctoring_video' THEN '{candidate}'::user_role[] ELSE required_for END;
ALTER TABLE consent_documents ALTER COLUMN applies_to SET NOT NULL;
ALTER TABLE consent_documents ADD CHECK (required_for <@ applies_to);

-- At most one unrevoked grant per document: granting again is idempotent (ON CONFLICT), and the
-- search pool's "has an active publication grant" semi-join reads this index.
CREATE UNIQUE INDEX user_consents_active_idx
    ON user_consents (user_id, document_id) WHERE revoked_at IS NULL;
