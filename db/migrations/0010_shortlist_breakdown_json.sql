-- The score breakdown of a shortlist item is written and read whole and never queried inside.
-- As `json` it keeps the text the API wrote: a read is a copy instead of a jsonb-to-text
-- conversion, and the API embeds it in the response without parsing (docs/solution.md §12.4).
ALTER TABLE shortlist_items ALTER COLUMN breakdown TYPE json USING breakdown::json;
