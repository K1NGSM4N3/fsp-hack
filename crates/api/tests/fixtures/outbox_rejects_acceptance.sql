-- The outbox refuses `invitation.accepted`: proves the acceptance rolls back together with it.
CREATE FUNCTION reject_accepted_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.event_type = 'invitation.accepted' THEN
        RAISE EXCEPTION 'outbox unavailable';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER reject_accepted_event BEFORE INSERT ON integration_events
    FOR EACH ROW EXECUTE FUNCTION reject_accepted_event();
