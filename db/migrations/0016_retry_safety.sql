-- Retries and double clicks create nothing twice (api.md, «Идемпотентность»): a vacancy per
-- `Idempotency-Key`, a need imported from an ATS per its `external_id`, one open complaint per
-- person and target.

ALTER TABLE vacancies ADD COLUMN idempotency_key text
    CHECK (length(idempotency_key) BETWEEN 1 AND 100);
CREATE UNIQUE INDEX vacancies_idempotency_idx ON vacancies (company_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

-- The vacancy id in the company's ATS.
ALTER TABLE hiring_needs ADD COLUMN external_id text
    CHECK (length(external_id) BETWEEN 1 AND 200);
CREATE UNIQUE INDEX hiring_needs_external_idx ON hiring_needs (company_id, external_id)
    WHERE external_id IS NOT NULL;

-- Repeats of an open complaint by the same person fold into the first one: it keeps every
-- comment, so nothing the person wrote is lost, and the company's quota counts it once.
WITH ranked AS (
    SELECT id, comment, created_at,
           first_value(id) OVER w AS keep_id,
           count(*) OVER (PARTITION BY reporter_user_id, target_kind, target_id) AS copies
    FROM reports
    WHERE resolved_at IS NULL
    WINDOW w AS (PARTITION BY reporter_user_id, target_kind, target_id ORDER BY created_at, id)
), merged AS (
    SELECT keep_id,
           string_agg(comment, E'\n\n' ORDER BY created_at, id) FILTER (WHERE comment IS NOT NULL)
               AS comments
    FROM ranked
    WHERE copies > 1
    GROUP BY keep_id
)
UPDATE reports r SET comment = m.comments FROM merged m WHERE r.id = m.keep_id;

DELETE FROM reports r
USING (
    SELECT id, row_number() OVER (PARTITION BY reporter_user_id, target_kind, target_id
                                  ORDER BY created_at, id) AS n
    FROM reports
    WHERE resolved_at IS NULL
) d
WHERE r.id = d.id AND d.n > 1;

CREATE UNIQUE INDEX reports_one_open_idx ON reports (reporter_user_id, target_kind, target_id)
    WHERE resolved_at IS NULL;
