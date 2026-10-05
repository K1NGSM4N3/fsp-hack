-- Age gate (architecture.md §9.2): candidates are 18+ by declaration, or 14–17 with the consent of a
-- legal representative. Adults give no birth date; minors give one so the restriction lifts by itself
-- on the 18th birthday (derived at query time as birth_date + interval '18 years' <= current_date).

CREATE TYPE age_bracket AS ENUM ('adult', 'minor');
CREATE TYPE guardian_decision AS ENUM ('granted', 'declined');

-- The age declaration travels with the grants made at registration: for adults it is the only record.
ALTER TABLE user_consents ADD COLUMN age_bracket age_bracket;

-- Only for candidates who registered as 14–17. Cleared 30 days after the 18th birthday, together with
-- the representative's data; from then on the account looks like any adult one.
ALTER TABLE candidates ADD COLUMN birth_date date;
CREATE INDEX candidates_birth_date_idx ON candidates (birth_date) WHERE birth_date IS NOT NULL;

-- One row per representative asked. Resending to the same, still silent, address rotates the link
-- hashes in place, so only the latest letter works; asking again after an answer adds a row and keeps
-- the earlier answer on record. The grant itself is an ordinary user_consents row of the minor.
CREATE TABLE guardian_requests (
    id                  uuid PRIMARY KEY,
    candidate_id        uuid        NOT NULL REFERENCES candidates (user_id) ON DELETE CASCADE,
    -- Encrypted like other PII; NULL once erased.
    full_name_enc       bytea,
    email_enc           bytea,
    -- HMAC of the normalized email: per-recipient rate limit and "not your own address" check.
    email_bidx          bytea,
    -- SHA-256 of the one-time answer link and of the revocation link of the latest letter.
    confirm_hash        bytea UNIQUE,
    revoke_hash         bytea UNIQUE,
    confirm_expires_at  timestamptz,
    sent_at             timestamptz,
    decision            guardian_decision,
    answered_at         timestamptz,
    consent_id          uuid REFERENCES user_consents (id),
    erased_at           timestamptz,
    created_at          timestamptz NOT NULL DEFAULT now(),
    CHECK ((decision IS NOT DISTINCT FROM 'granted') = (consent_id IS NOT NULL)),
    CHECK ((decision IS NULL) = (answered_at IS NULL))
);
CREATE INDEX guardian_requests_candidate_idx ON guardian_requests (candidate_id, created_at DESC);

-- Granted by the representative on the minor's behalf (applies_to = candidate), never at registration
-- and never through POST /me/consents.
INSERT INTO consent_documents (id, kind, version, title, body, required_for, applies_to) VALUES
('0191f000-0000-7000-8000-000000000a01', 'guardian_consent', 1,
 'Согласие законного представителя несовершеннолетнего',
 $$Я — законный представитель (родитель, усыновитель или попечитель) пользователя платформы «ФСП Таланты», которому от 14 до 18 лет, и даю согласие оператору платформы на обработку его персональных данных в соответствии с Федеральным законом от 27.07.2006 № 152-ФЗ «О персональных данных».

«ФСП Таланты» — платформа подбора ИТ-специалистов Федерации спортивного программирования России. Участник проходит тест по своему направлению и получает подтверждённую категорию, а компании сами предлагают ему работу, стажировку или подработку и сразу называют условия.

Обрабатываются: email, дата рождения, ФИО и контакты, которые участник укажет сам, город, навыки, опыт, ожидания по зарплате, результаты тестов, достижения ФСП (если он привяжет ФСП ID), приглашения и отклики. Цель — подбор работы и связь с работодателями. Обработка включает сбор, запись, хранение, уточнение, использование, обезличивание и удаление.

Компании видят только анонимную карточку — без имени и контактов, с отметкой «14–17 лет». Контакты получает только компания, чьё приглашение участник принял сам, или компания, на вакансию которой он сам откликнулся. Видеозапись во время теста ведётся, только если участник отдельно на неё согласится; без неё тест проходится так же.

Пока согласия нет, участник может пользоваться кабинетом и проходить тест, но работодатели его не видят. Согласие действует до отзыва или до 18-летия участника. Отозвать его можно в любой момент по ссылке из письма — профиль сразу скроется из поиска. Вместе с согласием сохраняются мои ФИО и email, указанные участником, время подтверждения, IP-адрес и браузер; они удаляются через 30 дней после 18-летия участника или сразу при удалении аккаунта.$$,
 '{}', '{candidate}');

-- New wording of the processing consent: it now names the age data. Version 1 grants stay valid.
INSERT INTO consent_documents (id, kind, version, title, body, required_for, applies_to) VALUES
('0191f000-0000-7000-8000-000000000a02', 'pd_processing', 2,
 'Согласие на обработку персональных данных',
 $$Я даю согласие оператору платформы на обработку моих персональных данных (ФИО, контактные данные, сведения о возрасте — есть ли мне 18 лет, а если нет, то дата рождения, сведения об образовании, опыте работы, навыках, результатах тестирования и спортивных достижениях) в целях подбора работы и взаимодействия с работодателями в соответствии с Федеральным законом от 27.07.2006 № 152-ФЗ «О персональных данных». Обработка включает сбор, запись, систематизацию, хранение, уточнение, использование, обезличивание и удаление. Контактные данные передаются работодателю только после того, как я приму его приглашение или сам откликнусь на вакансию. Если мне от 14 до 18 лет, профиль показывается работодателям только после того, как согласие подтвердит мой законный представитель; дата рождения нужна, чтобы снять это ограничение в день 18-летия, и удаляется через 30 дней после него. Согласие действует до его отзыва; отозвать его можно в настройках профиля.$$,
 '{candidate,employer}', '{candidate,employer}');
