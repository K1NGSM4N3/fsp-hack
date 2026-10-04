--! insert_report (comment?)
INSERT INTO reports (id, reporter_user_id, target_kind, target_id, reason, comment)
VALUES (:id, :reporter_user_id, :target_kind, :target_id, :reason, :comment);

--! recent_reports_by : (count)
SELECT count(*) AS count FROM reports WHERE reporter_user_id = :reporter_user_id AND created_at > now() - interval '1 day';
