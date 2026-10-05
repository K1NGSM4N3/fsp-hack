-- Trust signals against fake vacancies and dishonest employers (X-1, docs/solution.md §10).
-- Flags never block publishing: they warn the employer, lower the company's trust level and
-- weekly invitation quota, and fill the moderation list (GET /api/v1/admin/trust).

-- Registrable domain shared by the owner's verified login email and the company website;
-- NULL until they match. Recomputed when the profile is saved and when the trust page is opened.
ALTER TABLE companies ADD COLUMN verified_domain text;

-- MinHash signature of "title\ndescription" (matching::trust::similarity): 128 × u32 little-endian.
-- NULL for texts too short to compare; rows written before this migration are backfilled at start.
ALTER TABLE vacancies ADD COLUMN text_signature bytea
    CHECK (text_signature IS NULL OR octet_length(text_signature) = 512);
ALTER TABLE vacancies ADD COLUMN signed_at timestamptz;
-- matching::trust::TrustFlag[] as of the latest publish: [{"kind": "salary_above_market", ...}].
ALTER TABLE vacancies ADD COLUMN trust_flags jsonb NOT NULL DEFAULT '[]'
    CHECK (jsonb_typeof(trust_flags) = 'array');
-- When the vacancy was first flagged; flags older than 90 days stop counting against the company.
ALTER TABLE vacancies ADD COLUMN flagged_at timestamptz;

ALTER TABLE invitations ADD COLUMN trust_flags jsonb NOT NULL DEFAULT '[]'
    CHECK (jsonb_typeof(trust_flags) = 'array');

CREATE INDEX vacancies_flagged_idx ON vacancies (flagged_at DESC) WHERE flagged_at IS NOT NULL;
CREATE INDEX invitations_flagged_idx ON invitations (created_at DESC) WHERE trust_flags <> '[]';
CREATE INDEX vacancies_unsigned_idx ON vacancies (id) WHERE signed_at IS NULL;
-- Complaints are counted per company through their target.
CREATE INDEX reports_target_idx ON reports (target_kind, target_id) WHERE resolved_at IS NULL;
