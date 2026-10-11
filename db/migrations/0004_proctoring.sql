-- Proctoring of the adaptive test (architecture.md §5.6, §9.2). Video chunks are stored
-- encrypted on disk under PROCTORING_DIR; Postgres keeps only metadata and integrity events.
-- The summary status (recorded / declined / interrupted) is derived from these rows.

CREATE TYPE integrity_event AS ENUM (
    'tab_hidden', 'window_blur', 'fullscreen_exit', 'paste', 'camera_lost', 'recording_failed'
);

CREATE TABLE proctoring_sessions (
    session_id    uuid PRIMARY KEY REFERENCES assessment_sessions (id) ON DELETE CASCADE,
    candidate_id  uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    -- false: the candidate continued without the camera; events are still logged.
    camera        boolean     NOT NULL,
    -- The proctoring_video consent grant the recording relies on.
    consent_id    uuid REFERENCES user_consents (id) ON DELETE SET NULL,
    -- MediaRecorder container of the first chunk (video/webm in Chromium and Firefox, video/mp4 in Safari).
    mime          text,
    started_at    timestamptz NOT NULL DEFAULT now(),
    -- Video files removed (retention or consent revocation); the summary and events stay.
    purged_at     timestamptz,
    CHECK (camera OR consent_id IS NULL)
);
CREATE INDEX proctoring_sessions_candidate_idx ON proctoring_sessions (candidate_id);
CREATE INDEX proctoring_sessions_retention_idx ON proctoring_sessions (started_at)
    WHERE camera AND purged_at IS NULL;

CREATE TABLE proctoring_chunks (
    session_id   uuid        NOT NULL REFERENCES proctoring_sessions (session_id) ON DELETE CASCADE,
    seq          integer     NOT NULL CHECK (seq >= 0),
    -- Plaintext bytes; the file on disk adds the version prefix, nonce and tag.
    size         integer     NOT NULL CHECK (size > 0),
    received_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, seq)
);

CREATE TABLE proctoring_events (
    session_id   uuid            NOT NULL REFERENCES proctoring_sessions (session_id) ON DELETE CASCADE,
    kind         integrity_event NOT NULL,
    -- Browser clock, clamped by the API to the session's lifetime.
    occurred_at  timestamptz     NOT NULL,
    -- assessment_responses.seq of the item on screen.
    item_seq     smallint,
    -- How long the candidate was away (tab_hidden, window_blur).
    duration_ms  integer CHECK (duration_ms >= 0),
    received_at  timestamptz     NOT NULL DEFAULT now()
);
CREATE INDEX proctoring_events_session_idx ON proctoring_events (session_id, occurred_at);

-- Optional consent: required_for is empty, so registration does not ask for it.
INSERT INTO consent_documents (id, kind, version, title, body, required_for) VALUES
('0191f000-0000-7000-8000-000000000004', 'proctoring_video', 1,
 'Согласие на видеозапись во время теста',
 $$Я даю согласие оператору платформы на видеозапись с камеры моего устройства во время теста на грейд и на обработку этой записи в соответствии с Федеральным законом от 27.07.2006 № 152-ФЗ «О персональных данных». Записывается только изображение с камеры в низком разрешении — без звука и без содержимого экрана — с начала теста до его окончания. Запись дополняет отметки о событиях, которые платформа ведёт во время любого теста: переход на другую вкладку или в другое окно, выход из полноэкранного режима, вставка текста в поле ответа, отключение камеры. Цель обработки — подтвердить, что тест пройден самостоятельно, и разобрать спорный результат. Запись не используется для распознавания лица или установления личности. Запись хранится в зашифрованном виде 90 дней с даты теста и затем удаляется автоматически. Смотреть запись могу я и администратор платформы — только при разборе спорного результата. Работодатели запись не получают: они видят только отметку «Тест с видеозаписью» и число замечаний. Согласие необязательно: тест можно пройти без камеры, тогда у результата будет отметка «без видеозаписи». Согласие можно отозвать в настройках — записи будут удалены сразу; при удалении аккаунта записи удаляются вместе с ним.$$,
 '{}');
