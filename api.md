# API (рабочий контракт)

> Источник правды — OpenAPI, который генерирует бэкенд (utoipa): `GET /api/v1/openapi.json`,
> Swagger UI — `/api/docs`. Этот файл — план и соглашения на время разработки: добавил или
> поменял эндпоинт — обнови таблицу. Статусы: ⬜ план · ✅ реализован.

## Соглашения

- **База:** `/api/v1`. Ломающее изменение → новая версия `/api/v2` рядом со старой; старая отдаёт заголовки `Deprecation` и `Sunset`.
- **Формат:** JSON, `snake_case`, даты — RFC 3339 UTC, деньги — целые рубли (`i64`), id — UUID v7.
- **Аутентификация:** httpOnly cookie `fsp_at` (access JWT, 15 мин) и `fsp_rt` (refresh, путь `/api/v1/auth`). Альтернатива для API-клиентов и Swagger — `Authorization: Bearer <jwt>`.
- **CSRF:** изменяющие запросы с cookie должны иметь `Origin`, совпадающий с `PUBLIC_URL`.
- **Роли:** 🅰 анонимно · 🅲 candidate · 🅴 employer · 🅼 любой залогиненный · admin — администратор (создаётся только в базе, регистрацией не выдаётся).
- **Пагинация:** `?limit=20&offset=0` → `{ "items": [...], "total": 123 }`. Максимум `limit` — 100.
- **Ошибки:** RFC 9457 `application/problem+json`:
  ```json
  { "type": "about:blank", "title": "Смена грейда пока недоступна", "status": 409,
    "code": "grade_change_cooldown", "detail": "Следующая попытка — после 2027-01-03",
    "errors": { "field": ["сообщение"] } }
  ```
  `code` — стабильный машинный код, на него завязывается фронт. `errors` — только для 422. У 409 `consent_required` есть расширения `document_id` и `document_kind` — документ, который нужно принять (`POST /me/consents`). У 409 `guardian_consent_required` те же поля указывают на `guardian_consent`, но принять его может только законный представитель по ссылке из письма.
- **Коды:** 200/201/204; 400 — неверный запрос; 401 — нет или истекла сессия; 403 — нет прав; 404 — не найдено **или чужое** (не раскрываем существование); 409 — конфликт состояния (кулдаун, квота, повтор); 422 — ошибки валидации полей; 429 — лимит запросов (`Retry-After`).
- **Идемпотентность:** `POST /employer/invitations` принимает заголовок `Idempotency-Key`.

## Служебные

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/healthz` | 🅰 | Живость процесса | ✅ |
| GET | `/readyz` | 🅰 | Готовность: БД, Valkey | ✅ |
| GET | `/metrics` | внутр. | Prometheus (не проксируется Caddy наружу) | ✅ |
| GET | `/api/v1/openapi.json` | 🅰 | Спецификация | ✅ |
| GET | `/api/docs` | 🅰 | Swagger UI | ✅ |

## Справочники

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/catalog` | 🅰 | Направления, специализации, грейды, отрасли, дисциплины ФСП | ✅ |
| GET | `/api/v1/catalog/skills?q=` | 🅰 | Автодополнение навыков (с синонимами) | ✅ |
| GET | `/api/v1/catalog/test-plan` | 🅰 | План теста под вакансию и примеры заданий для двух кандидатов (T-4) | ✅ |
| GET | `/api/v1/consents/current` | 🅰 | Актуальные версии документов согласия: `required_for` — обязательны при регистрации, `applies_to` — кто может выдать | ✅ |

## Auth и аккаунт

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| POST | `/api/v1/auth/register` | 🅰 | `{email, password, role, consents[], age?, minor?}` → 201, письмо с подтверждением. Кандидат обязан передать `age`: `adult` — больше ничего; `minor` — `minor: {birth_date, guardian: {full_name, email}}`, дата рождения должна давать 14–17 лет, почта представителя ≠ своя (422 `age`, `minor`, `minor.birth_date`, `minor.guardian.*`). Работодатель — 18+ по декларации в форме, `age` не нужен. Возраст пишется в `age_bracket` выданных согласий | ✅ |
| POST | `/api/v1/auth/verify-email` | 🅰 | `{token}` → подтверждает email; у кандидата 14–17 лет — отправляет письмо законному представителю | ✅ |
| POST | `/api/v1/auth/resend-verification` | 🅰 | `{email}` → 204 всегда (не раскрываем наличие) | ✅ |
| POST | `/api/v1/auth/login` | 🅰 | `{email, password}` → cookie + `{access_token, expires_in, user}` | ✅ |
| POST | `/api/v1/auth/refresh` | 🅰 | По cookie `fsp_rt` → новая пара | ✅ |
| POST | `/api/v1/auth/logout` | 🅼 | Отзыв семейства refresh, очистка cookie | ✅ |
| GET | `/api/v1/me` | 🅼 | Пользователь, роль, статус онбординга; у кандидата `age: {bracket, adult_from, guardian}` — возрастная группа на сегодня и состояние согласия представителя | ✅ |
| DELETE | `/api/v1/me` | 🅼 | Удаление аккаунта и ПДн, у 14–17 — вместе с данными законного представителя | ✅ |
| GET | `/api/v1/me/consents` | 🅼 | Выданные согласия, включая отозванные | ✅ |
| POST | `/api/v1/me/consents` | 🅼 | `{document_id}` — дать согласие: только действующая редакция и только документ своей роли (`applies_to`), иначе 422; `guardian_consent` — всегда 422 (его даёт представитель). Повтор при действующей выдаче идемпотентен: 200 и та же выдача; новая — 201. Пишет время, IP, User-Agent | ✅ |
| POST | `/api/v1/me/consents/{document_id}/revoke` | 🅼 | Отзыв согласия (принимает id документа или id выдачи). Отзыв `pd_publication` или `pd_processing` скрывает профиль кандидата; `proctoring_video` — сразу удаляет видеозаписи; `guardian_consent` отзывает только представитель (404) | ✅ |
| GET | `/api/v1/me/export` | 🅼 | Все данные аккаунта JSON-вложением `fsp-talents-<дата>.json` (ст. 14 152-ФЗ): аккаунт, согласия с редакциями, IP, датами и объявленным возрастом; кандидату — возраст (группа, дата рождения у 14–17, запросы к представителям с ответами), профиль, приватность, опросы, тесты с итогами и сводкой прокторинга (без заданий, ответов и видео), история категории, приглашения, отклики, журнал доступа к контактам, ФСП, задания, загрузки резюме; работодателю — компания, потребности, вакансии, приглашения, задания | ✅ |

## Кандидат

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET / PUT | `/api/v1/candidate/profile` | 🅲 | Профиль: имя, контакты, город, формат, ожидания, о себе, стаж, навыки, отрасли | ✅ |
| POST | `/api/v1/candidate/resume` | 🅲 | multipart PDF → распознанные поля (черновик, не сохраняется в профиль сам) | ✅ |
| GET | `/api/v1/candidate/profile.pdf` | 🅲 | Стандартизированный PDF-профиль | ✅ |
| GET / PUT | `/api/v1/candidate/privacy` | 🅲 | Что видно работодателю, статус поиска. `visible: true` без действующих `pd_processing` и `pd_publication` → 409 `consent_required`, в problem — `document_id` и `document_kind` текущей редакции, которую нужно принять. До 18 лет без действующего `guardian_consent` → 409 `guardian_consent_required` | ✅ |
| GET | `/api/v1/candidate/guardian` | 🅲 | Согласие законного представителя: `required` (младше 18 сегодня), `adult_from`, `state` (`not_sent`, `pending`, `expired`, `granted`, `declined`, `revoked`), ФИО и email представителя, даты письма, ответа, отзыва. У взрослых — `required: false` | ✅ |
| POST | `/api/v1/candidate/guardian/resend` | 🅲 | Письмо представителю ещё раз с новыми ссылками (старые гаснут); после отказа или отзыва — новый запрос тому же человеку. 409 `guardian_not_needed`, `guardian_already_confirmed`; 429 — 3 письма в час, 5 в сутки на адрес | ✅ |
| PUT | `/api/v1/candidate/guardian` | 🅲 | `{full_name, email}` — другой представитель или адрес, сразу письмо. 422 (в т. ч. своя почта), 409, 429 — как выше | ✅ |
| GET | `/api/v1/candidate/contact-access` | 🅲 | «Кто открывал ваши контакты»: компания (`company_name: null`, если она удалила аккаунт), время, основание `invitation` / `application`; новые сверху. Сотрудник, IP и браузер в ответ не входят | ✅ |
| GET | `/api/v1/me` → `candidate.onboarding_step` | 🅲 | Шаг пути: профиль → опрос → тест → категория (отдельного эндпоинта нет) | ✅ |
| POST | `/api/v1/candidate/survey` | 🅲 | Опрос: направление, специализация, стек, отрасли, самооценка тем | ✅ |
| GET | `/api/v1/candidate/survey/history` | 🅲 | История опросов | ✅ |
| GET | `/api/v1/candidate/category` | 🅲 | Текущая категория, история присвоений, когда доступна смена грейда | ✅ |
| POST | `/api/v1/candidate/assessments` | 🅲 | `{specialization, grade, proctoring?: {camera, consent_document_id}}` → новая сессия (проверка политики, 409 `grade_change_cooldown`). С `camera: true` нужен id текущего документа `proctoring_video`, иначе 422; без поля `proctoring` — тест без видеозаписи | ✅ |
| GET | `/api/v1/candidate/assessments` | 🅲 | История сессий | ✅ |
| GET | `/api/v1/candidate/assessments/{id}` | 🅲 | Состояние или результат сессии; `proctoring` — сводка видеозаписи, `result.needs_review` — «требует подтверждения» | ✅ |
| GET | `/api/v1/candidate/assessments/{id}/current-item` | 🅲 | Текущее задание (без правильного ответа), дедлайн | ✅ |
| POST | `/api/v1/candidate/assessments/{id}/answer` | 🅲 | `{seq, answer}` → следующее задание или результат | ✅ |
| GET | `/api/v1/candidate/invitations` | 🅲 | Входящие приглашения | ✅ |
| GET | `/api/v1/candidate/invitations/{id}` | 🅲 | Приглашение; первое открытие → `viewed` | ✅ |
| POST | `/api/v1/candidate/invitations/{id}/accept` | 🅲 | → `accepted`, контакты раскрываются компании. До 18 лет без действующего согласия представителя (например, его отозвали после приглашения) → 409 `guardian_consent_required` | ✅ |
| POST | `/api/v1/candidate/invitations/{id}/decline` | 🅲 | `{reason, comment?}` → `declined` | ✅ |
| GET | `/api/v1/candidate/market` | 🅲 | Анонимный спрос в своей категории | ✅ |
| GET | `/api/v1/candidate/applications` | 🅲 | Свои отклики | ✅ |
| POST | `/api/v1/candidate/applications/{id}/withdraw` | 🅲 | Отозвать отклик | ✅ |
| GET | `/api/v1/candidate/tasks` | 🅲 | Регулярные задания | ✅ |
| POST | `/api/v1/candidate/tasks/{id}/submit` | 🅲 | Решение или подход | ✅ |
| POST | `/api/v1/candidate/tasks/{id}/skip` | 🅲 | Пропустить задание | ✅ |

## Прокторинг теста

Видеозапись и журнал событий теста (`architecture.md` §5.6). При `PROCTORING=off` загрузка и события отвечают 409 `proctoring_off`. Работодателю ни один из этих путей не доступен (403 `wrong_role`); в его DTO есть только `CandidateCard.proctoring` — отметка без id сессии и ссылок.

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/candidate/proctoring` | 🅲 | Шаг перед тестом: режим, текст согласия `proctoring_video`, выдано ли оно, срок хранения, лимиты | ✅ |
| PUT | `/api/v1/candidate/assessments/{id}/proctoring/chunks/{seq}` | 🅲 | Очередной фрагмент видео (сырое тело `video/webm` или `video/mp4`, ≤ 1 МиБ). Строго по порядку с 0; повтор того же `seq` → 204 без перезаписи. Только своя сессия с выбранной камерой и действующим согласием, пока тест идёт и 60 с после. 400 `empty_chunk`; 404; 409 `proctoring_not_recording`, `assessment_finished`, `chunk_out_of_order`; 413 `chunk_too_large`, `recording_too_large` (64 МиБ / 720 фрагментов) | ✅ |
| POST | `/api/v1/candidate/assessments/{id}/proctoring/events` | 🅲 | `{events: [{kind, at, item_seq?, duration_ms?}]}` — до 50 за запрос, до 500 на сессию (лишние отбрасываются); `kind`: `tab_hidden`, `window_blur`, `fullscreen_exit`, `paste`, `camera_lost`, `recording_failed`. Пишутся и без камеры. 404; 409 `assessment_finished`; 422 | ✅ |
| GET | `/api/v1/candidate/assessments/{id}/proctoring/recording` | 🅲 | Своя запись: фрагменты расшифровываются и отдаются потоком по порядку, `Content-Length` известен. 404 `recording_unavailable` — без камеры, не началась или удалена | ✅ |
| GET | `/api/v1/admin/proctoring/{session_id}` | admin | Сводка, флаг person-fit, «требует подтверждения» и журнал событий любой сессии — для разбора спора | ✅ |
| GET | `/api/v1/admin/proctoring/{session_id}/recording` | admin | Запись любой сессии; доступ пишется в лог | ✅ |

## ФСП

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/fsp/link/start` | 🅲 | 302 на Keycloak (state + PKCE) | ✅ |
| GET | `/api/v1/fsp/link/callback` | 🅲 | Обмен code, привязка `fsp_id`, синхронизация → 302 в кабинет | ✅ |
| POST | `/api/v1/fsp/link/dev` | 🅲 | `{fsp_id}` — только при `FSP_DEV_LINK=true` | ✅ |
| GET | `/api/v1/fsp/achievements` | 🅲 | Свой снимок достижений и ФСП-скор | ✅ |
| POST | `/api/v1/fsp/sync` | 🅲 | Пересинхронизация | ✅ |
| DELETE | `/api/v1/fsp/link` | 🅲 | Отвязка | ✅ |

## Работодатель

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET / PUT | `/api/v1/employer/company` | 🅴 | Профиль компании; `verified_domain` — общий домен почты входа и сайта, пересчитывается при сохранении | ✅ |
| GET / POST | `/api/v1/employer/needs` | 🅴 | Потребности: список / создать | ✅ |
| GET / PUT / DELETE | `/api/v1/employer/needs/{id}` | 🅴 | Потребность | ✅ |
| POST | `/api/v1/employer/needs/parse` | 🅴 | Свободный текст → специализация, грейд, навыки (черновик) | ✅ |
| GET | `/api/v1/employer/needs/{id}/categories` | 🅴 | Рекомендованные категории со статистикой | ✅ |
| GET | `/api/v1/employer/needs/{id}/test-plan` | 🅴 | Каким будет тест под эту потребность (T-4) | ✅ |
| POST | `/api/v1/employer/needs/{id}/shortlist` | 🅴 | Сформировать или уточнить подборку `{filters}` (в т. ч. `adults_only`); закреплённые сохраняются | ✅ |
| GET | `/api/v1/employer/needs/{id}/shortlist` | 🅴 | Подборка с объяснениями | ✅ |
| PATCH | `/api/v1/employer/needs/{id}/shortlist/{candidate_id}` | 🅴 | `{pinned?, dismissed?}` | ✅ |
| GET | `/api/v1/employer/categories` | 🅴 | Все категории: число кандидатов, с ФСП, медианные ожидания | ✅ |
| GET | `/api/v1/employer/candidates` | 🅴 | Поиск по банку: `specialization, grade, skills, fsp_only, adults_only, work_format, city, salary_max, q, sort`. `adults_only` («Только 18+») отсекает кандидатов 14–17 лет — фильтр, не штраф | ✅ |
| GET | `/api/v1/employer/candidates/{id}` | 🅴 | Анонимный профиль; `?need_id=` → объяснение под потребность. `candidate.minor` — отметка «14–17 лет», даты рождения нет | ✅ |
| GET | `/api/v1/employer/candidates/{id}/contacts` | 🅴 | Контакты — только при принятом приглашении или отклике (иначе 403 `contacts_not_granted`); пишет журнал | ✅ |
| GET / POST | `/api/v1/employer/invitations` | 🅴 | Список / отправить `{candidate_id, need_id?, vacancy_id?, title, message, salary: {min, max}, work_format?, contact}`; 409 `invitation_quota_exceeded` (лимит — по уровню доверия, 5–30 за 7 дней), `invitation_already_active`. Вилка сверяется с рынком категории кандидата, отметка пишется в приглашение (не блокирует) | ✅ |
| GET | `/api/v1/employer/invitations/{id}` | 🅴 | Приглашение со статусом и историей событий | ✅ |
| POST | `/api/v1/employer/invitations/{id}/withdraw` | 🅴 | Отозвать | ✅ |
| GET | `/api/v1/employer/invitations/stats` | 🅴 | Квота (`quota_limit` по уровню доверия, `quota_max`, `trust_level`), отклик кандидатов, причины отказов | ✅ |
| GET / POST | `/api/v1/employer/vacancies` | 🅴 | Вакансии | ✅ |
| GET / PUT | `/api/v1/employer/vacancies/{id}` | 🅴 | Вакансия; статус `draft` / `published` / `closed`. Каждое сохранение подписывает текст; публикация пишет отметки антифейка (вилка, дубль своей, копия чужой) — не блокирует | ✅ |
| GET | `/api/v1/employer/vacancies/{id}/applications` | 🅴 | Отклики (с контактами — кандидат откликнулся сам) | ✅ |
| POST | `/api/v1/employer/applications/{id}/status` | 🅴 | `viewed` / `accepted` / `rejected` | ✅ |
| GET / POST | `/api/v1/employer/tasks` | 🅴 | Регулярные задания | ✅ |
| POST | `/api/v1/employer/tasks/{id}/active` | 🅴 | Приостановить или возобновить раздачу | ✅ |
| GET | `/api/v1/employer/tasks/{id}/submissions` | 🅴 | Решения | ✅ |
| POST | `/api/v1/employer/tasks/{id}/submissions/{sid}/review` | 🅴 | Оценка решения | ✅ |

## Доверие и антифейк

Сигналы не блокируют действия: предупреждают работодателя, меняют уровень доверия и квоту, попадают в модерацию (`architecture.md` §11.1).

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/employer/trust` | 🅴 | Уровень доверия (`low` / `new` / `standard` / `high`), квота за 7 дней = база + бонусы + штрафы, причины с вкладом, советы, статус домена с подсказкой, правило словами. Перепроверяет домен | ✅ |
| GET | `/api/v1/employer/salary-check?specialization&grade&min&max` | 🅴 | Рынок категории (медиана и 90-й перцентиль ожиданий, при ≥ 5) и мягкие предупреждения `salary_above_market`, `salary_wide_range`; 422 при неверной вилке | ✅ |
| POST | `/api/v1/employer/vacancies/similar` | 🅴 | `{title, description, exclude_id?}` → свои черновики и опубликованные вакансии со сходством ≥ 0,6 (чужие не показываются) | ✅ |
| GET | `/api/v1/admin/trust` | admin | Очередь модерации: компании с отрицательными сигналами (от низкой квоты), отмеченные вакансии и приглашения с причинами словами | ✅ |

## Публичное

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/vacancies` | 🅰 | Опубликованные вакансии с фильтрами; `company_verified_domain` → отметка «Домен подтверждён» (также в `GET /candidate/invitations`) | ✅ |
| GET | `/api/v1/vacancies/{id}` | 🅰 | Вакансия | ✅ |
| POST | `/api/v1/vacancies/{id}/applications` | 🅲 | Откликнуться `{cover_letter?}`. До 18 лет — только с действующим согласием представителя, иначе 409 `guardian_consent_required`: отклик открывает компании контакты | ✅ |
| POST | `/api/v1/reports` | 🅼 | Жалоба на вакансию, приглашение или компанию (до 20 в сутки) | ✅ |

## Согласие законного представителя

Публичная страница `/guardian/confirm?token=…` — из письма представителю соискателя 14–17 лет. Токен — 32 случайных байта, в БД только SHA-256. В письме две ссылки: **ответ** (одноразовая, 7 дней) и **отзыв** (пока согласие действует).

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/guardian/consent?token=` | 🅰 | Что за ссылка (`link`: `confirm` / `revoke`), состояние, доступные действия (`actions`), замаскированная почта участника, ФИО представителя, действующий текст `guardian_consent`. `participant_adult: true` — участнику уже 18, действий нет. 400 `invalid_token` — неизвестная ссылка или ссылка из старого письма | ✅ |
| POST | `/api/v1/guardian/consent` | 🅰 | `{token, action}`: `grant` / `decline` — по ссылке ответа, пока она жива; `revoke` — по ссылке отзыва (до ответа она работает как `decline`). `grant` пишет выдачу `guardian_consent` на участника с IP, User-Agent и временем представителя; `decline` и `revoke` скрывают профиль. 400 `invalid_token`, `token_expired`; 409 `token_used`, `action_unavailable`, `guardian_not_needed` | ✅ |

## Валидация

Проверка подбора на своём наборе «вакансия – кандидат» (`docs/solution.md` §6.4). Та же логика, что у `validate pairs`: `matching::pairs`. Базу и данные пользователей не трогает, ничего не сохраняет.

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| POST | `/api/v1/eval/pairs` | 🅰 | `{vacancies: [{id, text, …}], candidates: [{id, resume_text, …}], labels?: [{vacancy_id, candidate_id, relevance 0–3}], k?, now?}` → как распознана каждая вакансия, топ-k нашего подбора с объяснениями и топ-k базовой линии «ключевые слова + стаж», Precision@5/@k, nDCG@k, MAP, MRR по вакансиям и в среднем, покрытие. Тело ≤ 8 МиБ (413 `payload_too_large`), ≤ 200 вакансий, 5 000 кандидатов, 100 000 оценок, тексты ≤ 20 000 символов (422 `validation_failed`, поле в `errors`); JSON не по схеме — 400/422 `invalid_json`; 10 запросов за 10 минут с IP (429) | ✅ |

## Статусы

**Приглашение:** `sent` → `viewed` → `accepted` | `declined`; из `sent`/`viewed` → `expired` (14 дней) | `withdrawn`.

**Отклик:** `sent` → `viewed` → `accepted` | `rejected`; кандидат может `withdrawn` до решения.

**Сессия теста:** `in_progress` → `passed` | `failed` | `expired` (брошена > 1 часа).

**Согласие представителя** (`state`): `not_sent` (подросток ещё не подтвердил почту) → `pending` → `granted` | `declined`; из `pending` → `expired` (ссылка ответа старше 7 дней, письмо можно повторить); `granted` → `revoked`.

**Видеозапись теста** (`proctoring.status`): `recording` пока тест идёт → `recorded` | `interrupted`; без камеры — `declined`. После удаления файлов (срок хранения, отзыв согласия) статус сохраняется, `purged: true`.

## Заглушка реестра ФСП (`fsp-mock`)

Внутренний сервис, не проксируется наружу.

| Метод | Путь | Описание |
|---|---|---|
| GET | `/participants/{fsp_id}` | Карточка участника: ФИО, регион, разряды |
| GET | `/participants/{fsp_id}/achievements` | Соревнования и разряды (модель — `architecture.md` §7.2) |

Доступ — `Authorization: Bearer` с client credentials токеном realm `fsp` (в dev — статический токен `FSP_REGISTRY_TOKEN`).
