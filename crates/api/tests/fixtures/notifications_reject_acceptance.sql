-- The feed refuses `invitation_accepted`: proves the acceptance rolls back together with its item.
CREATE FUNCTION reject_accepted_notification() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.kind = 'invitation_accepted' THEN
        RAISE EXCEPTION 'notifications unavailable';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER reject_accepted_notification BEFORE INSERT ON notifications
    FOR EACH ROW EXECUTE FUNCTION reject_accepted_notification();
