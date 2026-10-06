-- The feed refuses the annulment and take-down notices: proves the decision and the action roll
-- back together with their notification.
CREATE FUNCTION reject_moderation_notice() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.kind IN ('result_annulled', 'vacancy_unpublished') THEN
        RAISE EXCEPTION 'notifications unavailable';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER reject_moderation_notice BEFORE INSERT ON notifications
    FOR EACH ROW EXECUTE FUNCTION reject_moderation_notice();
