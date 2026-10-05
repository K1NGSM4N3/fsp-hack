-- When the shortlist was last (re)built: candidates first seen after the previous build are "new".
ALTER TABLE hiring_needs ADD COLUMN shortlist_generated_at timestamptz;
