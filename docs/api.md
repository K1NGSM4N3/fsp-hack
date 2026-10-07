# API (рабочий контракт)

> Источник правды — OpenAPI, который генерирует бэкенд (utoipa): `GET /api/v1/openapi.json`,
> Swagger UI — `/api/docs`; 131 операция на 111 путях. Этот файл — соглашения и сводка на время
> разработки: добавил или поменял эндпоинт — обнови таблицу. Статусы: ⬜ план · ✅ реализован.

## Соглашения

- **База:** `/api/v1`. Ломающее изменение → новая версия `/api/v2` рядом со старой; старая отдаёт заголовки `Deprecation` и `Sunset`.
- **Формат:** JSON, `snake_case`, даты — RFC 3339 UTC, деньги — целые рубли (`i64`), id — UUID v7.
- **Аутентификация:** httpOnly cookie `fsp_at` (access JWT, 15 мин, путь `/`, `SameSite=Lax`), `fsp_rt` (refresh, 30 дней, путь `/api/v1/auth`, `SameSite=Strict`) и `fsp_session` (без данных, 30 дней — по нему Next только решает, отправлять ли на `/login`), а также `fsp_at_exp` — без `HttpOnly`, 30 дней: только срок действия access-токена (unix-секунды), самого токена в нём нет. По нему страница обновляет сессию до запроса, а не после его 401; выход и удаление аккаунта стирают и его. Альтернатива для API-клиентов и Swagger — `Authorization: Bearer <jwt>`. Дерево `/api/v1/integrations/…` — только API-ключ компании `Authorization: Bearer fsp_live_…`: cookie и JWT туда не доходят, а ключ не открывает остальные пути.
- **CSRF:** изменяющий запрос с cookie и без `Authorization` должен прийти с `Origin` (при его отсутствии — `Referer`), равным `PUBLIC_URL` или начинающимся с `PUBLIC_URL/`; иначе 403 `csrf`.
- **Роли:** 🅰 анонимно · 🅲 candidate · 🅴 employer · 🅼 любой залогиненный · admin — модератор (в интерфейсе «Модератор»; регистрацией не выдаётся, в демо — `moderator@demo.fsp`) · 🔑 API-ключ компании со скоупом.
- **Пагинация:** `GET /vacancies` и `GET /employer/candidates` — `?limit=20&offset=0` → `{ "items": [...], "total": 123 }`, максимум `limit` — 100. Подборка — `limit` в фильтрах (по умолчанию 30, до 100). Списки, которые растут, — непрозрачный курсор `cursor` / `next_cursor` (keyset по времени и id, новые сверху): приглашения обеих сторон, отклики на вакансию, ответы на задание (`limit` 1–100, по умолчанию 30), журнал доставок ATS (по умолчанию 50, до 200), очереди и журнал модерации, лента уведомлений (там курсор — id). `next_cursor: null` — страница последняя; курсор не из ответа — 400 `invalid_query`. Опрос приглашений ATS — тоже `cursor` / `next_cursor`, но по возрастанию и с задержкой в несколько секунд (раздел «Интеграции ATS»). Остальные списки отдаются массивом целиком.
- **Ошибки:** RFC 9457 `application/problem+json`:
  ```json
  { "type": "about:blank", "title": "Смена грейда пока недоступна", "status": 409,
    "code": "grade_change_cooldown", "detail": "Следующая попытка — после 03.01.2027",
    "errors": { "field": ["сообщение"] } }
  ```
  `code` — стабильный машинный код, на него завязывается фронт; общие — `unauthorized`, `wrong_role`, `not_found`, `validation_failed`, `rate_limited`, `csrf`, `internal`. `errors` — только для 422. Запрос, который не удалось разобрать, — тоже problem+json со своим `code` (extractor'ы `crates/api/src/extract.rs`): `invalid_json` — 400, если тело не JSON, и 422, если JSON не по схеме (неизвестное значение enum, не тот тип, нет поля; что именно — в `detail`); `unsupported_media_type` — 415, не тот `Content-Type`; `invalid_path` — 400, параметр пути не разбирается (например, не UUID); `invalid_query` — 400, нет обязательного query-параметра или он не разбирается; `payload_too_large` — 413; `invalid_body` — 400, тело не удалось дочитать. В OpenAPI эти ответы добавлены каждому методу, где они возможны, вместе с 401 `unauthorized` у методов с сессией, 403 `wrong_role` у методов кабинета одной роли (`/candidate`, `/employer`, `/admin`, `/fsp`, отклик на вакансию) и 403 `csrf` у изменяющих методов с сессией. У 409 `consent_required` есть расширения `document_id` и `document_kind` — документ, который нужно принять (`POST /me/consents`). У 409 `guardian_consent_required` те же поля указывают на `guardian_consent`, но принять его может только законный представитель по ссылке из письма.
- **Коды:** 200/201/204, 202 — принято к обработке (событие реестра ФСП), 303 — редиректы ФСП ID; 400 — неверный запрос; 401 — нет или истекла сессия; 403 — нет прав; 404 — не найдено **или чужое** (не раскрываем существование); 408 — запрос дольше 30 с; 409 — конфликт состояния (кулдаун, квота, повтор); 413 — тело больше лимита; 422 — ошибки валидации полей; 429 — лимит запросов (`Retry-After` — сколько секунд осталось до сброса окна лимита, который отказал; то же словами в `detail`); 502/503 — реестр или ФСП ID недоступны или не настроены.
- **Размер тела:** по умолчанию до 256 КиБ, резюме — до 10 МиБ, фрагмент видео — до 1 МиБ, `POST /eval/pairs` — до 8 МиБ; сверх 10 МиБ (по `Content-Length`) запрос отсекается до хендлера — тоже 413 `payload_too_large`.
- **Создание:** 201 у вакансии, потребности, приглашения и сессии теста приходит с заголовком `Location` — путь, по которому созданное читается (`/api/v1/employer/vacancies/{id}` и т. п.).
- **Идемпотентность:** `POST /employer/invitations` и `POST /employer/vacancies` принимают заголовок `Idempotency-Key` (до 100 символов, в пределах компании): повтор — и одновременная отправка с тем же ключом — возвращает 200 и уже созданное; ключ длиннее 100 символов — 422 (`idempotency_key`), у приглашения — и ключ, уже использованный для другого кандидата. Кабинет делает один ключ на открытую форму, поэтому двойной клик не создаёт дубль вакансии и не снижает квоту отметкой «дубль своей». `POST /integrations/needs` с `external_id` (id вакансии в ATS, уникален в компании) при повторе возвращает 200 и уже импортированную потребность. Жалоба (`POST /reports`) — одна открытая на человека и объект: повтор возвращает её же с 200 и `already_open: true`. Этапы найма от ATS идемпотентны по паре «приглашение или отклик + этап». События реестра ФСП (`POST /fsp/registry/events`) применяются один раз на `event_id`. Двойной клик «Начать тест» — 409 `assessment_in_progress`, а не 500.
- **Длина текста** считается в символах, не в байтах: лимит «до 4000 символов» одинаков для кириллицы и латиницы.
- **Кеширование:** по умолчанию `Cache-Control: private, no-store` — всё, что за cookie, токеном или API-ключом, и всё с ПДн. Публично: `GET /catalog`, `/catalog/skills`, `/consents/current` — `public, max-age=300, stale-while-revalidate=86400`; `GET /vacancies`, `/vacancies/{id}`, `/vacancies/{id}/similar` — `public, max-age=30, stale-while-revalidate=120`. У публичных ответов есть `ETag`, `If-None-Match` → 304. `/guardian/consent`, `POST /eval/pairs`, `GET /catalog/test-plan` — `no-store`. Ошибки и ответы с `Set-Cookie` в публичных группах — `no-store`; 401 — `private, no-cache` (данных в нём нет, docs/architecture.md §12.1). Тело ответа, которое клиенту не нужно, всё равно прочитать или отменить (`res.body?.cancel()`): Chrome не завершает непрочитанный ответ с `no-store` и держит соединение.

## Служебные

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/healthz` | 🅰 | Живость процесса | ✅ |
| GET | `/readyz` | 🅰 | Готовность: 200 или 503 с телом `db=<bool> kv=<bool>`; без `VALKEY_URL` проверка kv всегда проходит (хранилище в памяти) | ✅ |
| GET | `/metrics` | внутр. | Prometheus (не проксируется Caddy наружу) | ✅ |
| GET | `/api/v1/openapi.json` | 🅰 | Спецификация | ✅ |
| GET | `/api/docs` | 🅰 | Swagger UI | ✅ |

## Справочники

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/catalog` | 🅰 | Направления, специализации, грейды, отрасли, дисциплины ФСП | ✅ |
| GET | `/api/v1/catalog/skills?q=&specialization=` | 🅰 | Автодополнение навыков с синонимами, до 20; без `q` — типичный стек специализации | ✅ |
| GET | `/api/v1/catalog/test-plan` | 🅰 | План теста под вакансию и примеры заданий для двух кандидатов (T-4) | ✅ |
| GET | `/api/v1/consents/current` | 🅰 | Актуальные версии документов согласия: `required_for` — обязательны при регистрации, `applies_to` — кто может выдать | ✅ |

## Auth и аккаунт

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| POST | `/api/v1/auth/register` | 🅰 | `{email, password, role, consents[], company_name?, age?, minor?}` → 201, письмо с подтверждением (ссылка живёт 24 ч). Все обязательные для роли документы — в `consents`, иначе 422 `consents`; занятый email — 409 `email_taken`; 20 регистраций в час с IP. Работодатель обязан передать `company_name`. Кандидат обязан передать `age`: `adult` — больше ничего; `minor` — `minor: {birth_date, guardian: {full_name, email}}`, дата рождения должна давать 14–17 лет, почта представителя ≠ своя (422 `age`, `minor`, `minor.birth_date`, `minor.guardian.*`). Работодатель — 18+ по декларации в форме, `age` не нужен. Возраст пишется в `age_bracket` выданных согласий | ✅ |
| POST | `/api/v1/auth/verify-email` | 🅰 | `{token}` → подтверждает email; у кандидата 14–17 лет — отправляет письмо законному представителю. 400 `invalid_token` — ссылка недействительна, устарела или уже использована | ✅ |
| POST | `/api/v1/auth/resend-verification` | 🅰 | `{email}` → 204 всегда (не раскрываем наличие); 3 письма в час на адрес | ✅ |
| POST | `/api/v1/auth/login` | 🅰 | `{email, password}` → cookie + `{access_token, expires_in, user}`. 401 `invalid_credentials`, 403 `email_not_verified`; 10 попыток за 15 минут на пару IP + email | ✅ |
| POST | `/api/v1/auth/refresh` | 🅰 | По cookie `fsp_rt` → новая пара | ✅ |
| POST | `/api/v1/auth/logout` | 🅰 | Отзыв семейства refresh, access-токен — в deny-list, очистка cookie; без сессии — тоже 204 | ✅ |
| GET | `/api/v1/me` | 🅼 | Пользователь, роль, статус онбординга; у кандидата `age: {bracket, adult_from, guardian}` — возрастная группа на сегодня и состояние согласия представителя | ✅ |
| DELETE | `/api/v1/me` | 🅼 | Удаление аккаунта и ПДн, у 14–17 — вместе с данными законного представителя; уведомления, письма и их настройки удаляются тоже | ✅ |
| GET | `/api/v1/me/consents` | 🅼 | Выданные согласия, включая отозванные | ✅ |
| POST | `/api/v1/me/consents` | 🅼 | `{document_id}` — дать согласие: только действующая редакция и только документ своей роли (`applies_to`), иначе 422; `guardian_consent` — всегда 422 (его даёт представитель). Повтор при действующей выдаче идемпотентен: 200 и та же выдача; новая — 201. Пишет время, IP, User-Agent | ✅ |
| POST | `/api/v1/me/consents/{id}/revoke` | 🅼 | Отзыв согласия (`id` — документа или выдачи). Отзыв `pd_publication` или `pd_processing` скрывает профиль кандидата; `proctoring_video` — сразу удаляет видеозаписи; `guardian_consent` отзывает только представитель (404) | ✅ |
| GET | `/api/v1/me/export` | 🅼 | Все данные аккаунта JSON-вложением `fsp-talents-<дата>.json` (ст. 14 152-ФЗ): аккаунт, согласия с редакциями, IP, датами и объявленным возрастом; кандидату — возраст (группа, дата рождения у 14–17, запросы к представителям с ответами), профиль, приватность, опросы, тесты с итогами и сводкой прокторинга (без заданий, ответов и видео), история категории, приглашения, отклики, журнал доступа к контактам, ФСП, задания, загрузки резюме; работодателю — компания, потребности, вакансии, приглашения, задания; обоим — `notifications`: лента уведомлений, настройки писем по видам и список отправленных писем | ✅ |

## Уведомления

Тег Swagger «Уведомления» (`docs/architecture.md` §13). Лента — то, что видит колокольчик в кабинете; письма собираются из той же ленты. Ответы с сессией — `private, no-store`, как всё за cookie.

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/me/notifications?unread=&cursor=&limit=` | 🅼 | Новые сверху: `{items: [{id, kind, title, body, link, created_at, read_at}], next_cursor}`. Пагинация по ключу: `cursor` — `next_cursor` прошлой страницы (id, UUIDv7), `limit` — 1–50, по умолчанию 20; `unread=true` — только непрочитанные. Текст собирается из полезной нагрузки при чтении; `link` — путь в кабинете. 400 `invalid_query` — курсор не UUID | ✅ |
| POST | `/api/v1/me/notifications/read` | 🅼 | `{ids: [...]}` (до 100) или `{all: true}` → `{count}` — сколько непрочитанных осталось. 422 — ни того ни другого или больше 100 id. Открытие самого приглашения (`GET /candidate/invitations/{id}`, `/employer/invitations/{id}`) отмечает прочитанными уведомления о нём | ✅ |
| GET | `/api/v1/me/notifications/unread-count` | 🅼 | `{count}` до 100 (значок показывает «99+»): один подсчёт по частичному индексу, рассчитан на опрос раз в 30 с | ✅ |
| GET | `/api/v1/me/notification-settings` | 🅼 | Виды своей роли: `{items: [{kind, title, description, in_app: true, email}]}` — в кабинете всё приходит всегда, письма включаются по видам | ✅ |
| PUT | `/api/v1/me/notification-settings` | 🅼 | `{email: [{kind, enabled}]}` — только меняемые виды, остальные не трогаются; ответ — новые настройки. 422 `validation_failed` (`email`) — вид другой роли или один вид дважды. Письмо, уже стоящее в очереди, при отправке пропускает выключенные виды | ✅ |
| POST | `/api/v1/notifications/unsubscribe?token=` | 🅰 | Отписка в один клик (RFC 8058): почтовый клиент шлёт сюда `List-Unsubscribe=One-Click`, кнопка на странице `/notifications/unsubscribe` — тот же POST; тело не читается, сессия не нужна. Токен подписан (HMAC-SHA256, ключ выводится из `BIDX_KEY`) и называет аккаунт и вид письма (или все). → `{kind, title, settings_path}`; повтор ничего не меняет. 400 `invalid_token` — подпись не сходится | ✅ |
| GET | `/api/v1/notifications/unsubscribe?token=` | 🅰 | Ссылку открыли в браузере: GET ничего не отключает (её открывают и сканеры почты), 303 на страницу `/notifications/unsubscribe?token=…` с кнопкой. 400 `invalid_token` | ✅ |

Виды (`kind`) — соискателю: `invitation_received`, `invitation_expiring` (за 48 ч до истечения, один раз), `invitation_withdrawn`, `hiring_stage`, `application_status` («просмотрено» — только в кабинете), `assessment_result`, `guardian_decision`, решения модератора по тесту `result_confirmed`, `result_annulled`, `retest_ordered` (с комментарием модератора; модератор не назван), `fsp_achievements` (реестр ФСП прислал изменения достижений); компании: `invitation_accepted`, `invitation_declined`, `application_received`, `webhook_failing`, `trust_changed`, решения модератора `vacancy_unpublished`, `invitations_frozen` (с датой), `sanction_lifted` (вакансия возвращена или заморозка снята досрочно), `complaint_resolved` (жалоба не подтвердилась; кто жаловался — не сообщается, приглашение из жалобы не называется). Все виды по умолчанию приходят и письмом, письма отключаются по видам. В письмах компании нет ФИО и контактов кандидата — только анонимный код; комментарий кандидата к отказу виден только в кабинете.

## Кандидат

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET / PUT | `/api/v1/candidate/profile` | 🅲 | Профиль: имя, контакты, город, формат, ожидания, о себе, стаж, навыки, отрасли. 422: `contacts` — ни одного способа связи; `contacts.phone` — не 10–15 цифр (допустимы `+` в начале, пробелы, скобки, дефис); `contacts.telegram` — не ник из 5–32 латинских букв, цифр и `_` (можно с `@` или ссылкой `t.me/…`) | ✅ |
| POST | `/api/v1/candidate/resume` | 🅲 | multipart PDF до 10 МиБ → распознанные поля: черновик для формы, в профиль сами не попадают. Сразу сохраняются только навыки с источником `resume` (заменяют прежние) и запись о загрузке. 422 `not_pdf`, `pdf_unreadable`, `pdf_no_text`, `pdf_too_large` | ✅ |
| GET | `/api/v1/candidate/profile.pdf` | 🅲 | Стандартизированный PDF-профиль | ✅ |
| GET / PUT | `/api/v1/candidate/privacy` | 🅲 | Что видно работодателю, статус поиска. `visible: true` без действующих `pd_processing` и `pd_publication` → 409 `consent_required`, в problem — `document_id` и `document_kind` текущей редакции, которую нужно принять. До 18 лет без действующего `guardian_consent` → 409 `guardian_consent_required` | ✅ |
| GET | `/api/v1/candidate/guardian` | 🅲 | Согласие законного представителя: `required` (младше 18 сегодня), `adult_from`, `state` (`pending`, `expired`, `granted`, `declined`, `revoked`; `null`, пока запроса нет — в `/me` это `not_sent`), ФИО и email представителя, даты письма, ответа, отзыва. У взрослых — `required: false` | ✅ |
| POST | `/api/v1/candidate/guardian/resend` | 🅲 | Письмо представителю ещё раз с новыми ссылками (старые гаснут); после отказа или отзыва — новый запрос тому же человеку. 409 `guardian_not_needed`, `guardian_already_confirmed`, `guardian_missing`, `guardian_already_answered` (представитель ответил, пока готовилось письмо); 429 — больше 3 писем в час от подростка или 5 в сутки на один адрес | ✅ |
| PUT | `/api/v1/candidate/guardian` | 🅲 | `{full_name, email}` — другой представитель или адрес, сразу письмо. 422 (в т. ч. своя почта), 409, 429 — как выше | ✅ |
| GET | `/api/v1/candidate/contact-access` | 🅲 | «Кто открывал ваши контакты»: компания (`company_name: null`, если она удалила аккаунт), время, основание `invitation` / `application`, канал `channel`: `cabinet` / `api` / `webhook` (передано в ATS компании); новые сверху, до 500 записей. Сотрудник, IP и браузер в ответ не входят | ✅ |
| GET | `/api/v1/me` → `candidate.onboarding_step` | 🅲 | Шаг пути: профиль → опрос → тест → категория (отдельного эндпоинта нет) | ✅ |
| POST | `/api/v1/candidate/survey` | 🅲 | Опрос: направление, специализация, стек, отрасли, самооценка тем | ✅ |
| GET | `/api/v1/candidate/survey/history` | 🅲 | История опросов | ✅ |
| GET | `/api/v1/candidate/category` | 🅲 | Текущая категория, история присвоений (у аннулированных модератором — `annulled_at`); `available` — тесты, доступные сейчас (`ordered_retest` — пересдача, назначенная модератором, `camera_required` — только с камерой); `locked` — тесты с датой открытия `available_at` и причиной `reason`: `retry_blocked` (30 дней после неудачной или брошенной попытки на этот грейд или неудачного повышения) или `grade_change_cooldown` (90 дней от присвоения); например, дотест на грейд, проваленный меньше 30 дней назад; `next_change_at` — когда станет возможна смена грейда, если сейчас нельзя ни вверх, ни вниз | ✅ |
| POST | `/api/v1/candidate/assessments` | 🅲 | `{specialization, grade, proctoring?: {camera, consent_document_id}}` → новая сессия. Политика грейда (docs/architecture.md §5.4): 409 `grade_change_cooldown`, `retry_blocked` (30 дней после неудачной или брошенной попытки на этот грейд, в т. ч. для дотеста после уверенной сдачи грейдом ниже, и 30 дней после решения модератора аннулировать результат на этом грейде), `attempt_not_allowed`, `already_assigned`; в `detail` — дата; ещё 409 `assessment_in_progress`, `survey_required`, `camera_required` — модератор назначил пересдачу с видеозаписью, тест на этот грейд и выше начинается только с `camera: true` (docs/architecture.md §5.7); 422 `specialization` — не та, что в опросе или категории. С `camera: true` нужен id текущего документа `proctoring_video` (выдача согласия пишется тут же), иначе 422; без поля `proctoring` — тест без видеозаписи | ✅ |
| GET | `/api/v1/candidate/assessments` | 🅲 | История сессий | ✅ |
| GET | `/api/v1/candidate/assessments/{id}` | 🅲 | Состояние или результат сессии; `proctoring` — сводка видеозаписи, `result.needs_review` — «требует подтверждения» (пока нет решения модератора), `review` — решение модератора с комментарием, `recording_views` — когда модератор смотрел запись, `result.offers` / `result.locked` — что доступно сейчас и что откроется позже (как в `/candidate/category`) | ✅ |
| GET | `/api/v1/candidate/assessments/{id}/current-item` | 🅲 | Текущее задание (без правильного ответа), дедлайн. Просроченное задание засчитывается неверным и выдаётся следующее; если 30 минут после дедлайна прошли без ответа — сессия закрывается как `expired` | ✅ |
| POST | `/api/v1/candidate/assessments/{id}/answer` | 🅲 | `{seq, answer}` → следующее задание или результат; ответ после дедлайна засчитывается неверным. 409 `assessment_finished` (в т. ч. когда 30 минут после дедлайна прошли без ответа: сессия только что закрыта как брошенная), `stale_item` (ответ не на текущее задание) | ✅ |
| GET | `/api/v1/candidate/invitations?status=&cursor=&limit=` | 🅲 | Входящие приглашения, новые сверху: `{items, next_cursor, by_status}`. `status` — статусы через запятую (`sent,viewed` — «Ждут ответа»), `by_status` — сколько приглашений каждого статуса без учёта фильтра (счётчики вкладок); `limit` 1–100, по умолчанию 30. `outcomes` — этапы найма, которые отметила ATS компании («Компания отметила: собеседование»). 400 `invalid_query` — неизвестный статус или курсор не из ответа | ✅ |
| GET | `/api/v1/candidate/invitations/{id}` | 🅲 | Приглашение; первое открытие → `viewed` | ✅ |
| POST | `/api/v1/candidate/invitations/{id}/accept` | 🅲 | → `accepted`, контакты раскрываются компании. 409 `invitation_closed` — приглашение уже не ждёт ответа. До 18 лет без действующего согласия представителя (например, его отозвали после приглашения) → 409 `guardian_consent_required` | ✅ |
| POST | `/api/v1/candidate/invitations/{id}/decline` | 🅲 | `{reason, comment?}` → `declined`; `reason`: `salary`, `stack`, `format`, `company`, `not_looking`, `other`, комментарий до 1000 символов (длиннее — 422). `not_looking` закрывает статус поиска. 409 `invitation_closed` | ✅ |
| GET | `/api/v1/candidate/market` | 🅲 | Анонимный спрос в своей категории; сколько компаний открывали профиль за 30 дней. 409 `no_category` — категории ещё нет | ✅ |
| GET | `/api/v1/candidate/applications` | 🅲 | Свои отклики; `outcomes` — этапы найма от ATS компании | ✅ |
| POST | `/api/v1/candidate/applications/{id}/withdraw` | 🅲 | Отозвать отклик; 404 — не свой; 409 `application_closed` — отклик уже рассмотрен или отозван | ✅ |
| GET | `/api/v1/candidate/tasks` | 🅲 | Регулярные задания | ✅ |
| POST | `/api/v1/candidate/tasks/{id}/submit` | 🅲 | Решение или подход; 404 — не своё; 409 `task_closed` — задание закрыто или просрочено | ✅ |
| POST | `/api/v1/candidate/tasks/{id}/skip` | 🅲 | Пропустить задание; 404 — не своё; 409 `task_closed` | ✅ |

## Прокторинг теста

Видеозапись и журнал событий теста (`docs/architecture.md` §5.6). При `PROCTORING=off` загрузка и события отвечают 409 `proctoring_off`. Работодателю ни один из этих путей не доступен (403 `wrong_role`); в его DTO есть только `CandidateCard.proctoring` — отметка без id сессии и ссылок.

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/candidate/proctoring` | 🅲 | Шаг перед тестом: режим, текст согласия `proctoring_video`, выдано ли оно, срок хранения, лимиты | ✅ |
| PUT | `/api/v1/candidate/assessments/{id}/proctoring/chunks/{seq}` | 🅲 | Очередной фрагмент видео (сырое тело, ≤ 1 МиБ; `Content-Type` первого фрагмента — `video/webm` или `video/mp4` — запоминается для воспроизведения, иное считается webm). Строго по порядку с 0; повтор того же `seq` → 204 без перезаписи. Только своя сессия с выбранной камерой и действующим согласием, пока тест идёт и 60 с после. 400 `empty_chunk`; 404; 409 `proctoring_not_recording`, `assessment_finished`, `chunk_out_of_order`; 413 `chunk_too_large`, `recording_too_large` (64 МиБ / 720 фрагментов) | ✅ |
| POST | `/api/v1/candidate/assessments/{id}/proctoring/events` | 🅲 | `{events: [{kind, at, item_seq?, duration_ms?}]}` — до 50 за запрос, до 500 на сессию (лишние отбрасываются); `kind`: `tab_hidden`, `window_blur`, `fullscreen_exit`, `paste`, `camera_lost`, `recording_failed`. Пишутся и без камеры; принимаются, пока тест идёт и 60 с после. 404; 409 `assessment_finished`; 422 | ✅ |
| GET | `/api/v1/candidate/assessments/{id}/proctoring/recording` | 🅲 | Своя запись: фрагменты расшифровываются и отдаются потоком по порядку, `Content-Length` известен. 404 `recording_unavailable` — без камеры, не началась или удалена | ✅ |
| GET | `/api/v1/admin/proctoring/{session_id}` | admin | Карточка теста для разбора спора — раздел «Модерация» ниже | ✅ |
| GET | `/api/v1/admin/proctoring/{session_id}/recording` | admin | Запись любой сессии; каждый просмотр — строка журнала модерации (раздел «Модерация») | ✅ |

## ФСП

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/fsp/link/start` | 🅲 | 303 на Keycloak: `state`, PKCE S256 и `nonce` хранятся в `oidc_states`. 503 `fsp_oidc_disabled` — ФСП ID не настроен, 502 `fsp_unavailable` | ✅ |
| GET | `/api/v1/fsp/link/callback` | 🅰 | Пользователь — по одноразовой строке `state` (живёт 10 минут). Обмен code, проверка id_token, привязка `fsp_id`, синхронизация → 303 на `/candidate/settings?fsp=linked` или `?fsp=error&reason=…` (`cancelled`, `expired`, `no_fsp_id`, `verification_failed`, `already_linked`, …) | ✅ |
| POST | `/api/v1/fsp/link/dev` | 🅲 | `{fsp_id}` — только при `FSP_DEV_LINK=true` (иначе 404); 409 `fsp_already_linked`, 422 — пустой `fsp_id`. Как и вход через ФСП ID, синхронизирует достижения и пишет навыки, которые они подтверждают (источник `fsp`, docs/architecture.md §7.4) | ✅ |
| GET | `/api/v1/fsp/achievements` | 🅲 | Свой снимок достижений и ФСП-скор | ✅ |
| POST | `/api/v1/fsp/sync` | 🅲 | Пересинхронизация: новый снимок и навыки из ФСП по нему; 404 — ФСП ID не привязан, 502 `fsp_unavailable`, 503 `fsp_registry_disabled` — реестр не настроен. Участника нет в реестре — пустой список достижений | ✅ |
| DELETE | `/api/v1/fsp/link` | 🅲 | Отвязка: удаляются снимок достижений, навыки из ФСП и принятые события реестра | ✅ |
| POST | `/api/v1/fsp/registry/events` | реестр ФСП | Push вместо опроса: `{event_id, type: "achievements.changed", fsp_id, occurred_at}` с подписью `FSP-Signature: t=<unix>,v1=hex(HMAC-SHA256(FSP_REGISTRY_WEBHOOK_SECRET, t + "." + тело))` — как у исходящих вебхуков ATS; `t` дальше 5 минут — 401 `signature_expired`, нет подписи или не совпала — 401 `signature_invalid`. Для привязанного участника — та же синхронизация, что `POST /fsp/sync` (снимок, навыки из ФСП, сброс кеша пула), и, если что-то изменилось, уведомление `fsp_achievements`. Один раз на `event_id`. Ответ 202 без тела одинаков для применённого, неизменившегося, повторного события и незнакомого `fsp_id` — о нём ничего не сохраняется; другие `type` принимаются и игнорируются. 422 — `event_id` (1–128 видимых ASCII), `fsp_id`; 413 — тело больше 16 КиБ; 429 — больше 300 запросов в минуту с адреса; 502 `fsp_unavailable` — реестр не отдал достижения (доставьте позже); 503 `fsp_webhook_disabled` — секрет не задан | ✅ |

## Работодатель

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET / PUT | `/api/v1/employer/company` | 🅴 | Профиль компании; `verified_domain` — общий домен почты входа и сайта, пересчитывается при сохранении | ✅ |
| GET / POST | `/api/v1/employer/needs` | 🅴 | Потребности: список / создать | ✅ |
| GET / PUT / DELETE | `/api/v1/employer/needs/{id}` | 🅴 | Потребность | ✅ |
| POST | `/api/v1/employer/needs/parse` | 🅴 | Свободный текст → специализация, грейд, навыки (черновик) | ✅ |
| GET | `/api/v1/employer/needs/{id}/categories` | 🅴 | Рекомендованные категории со статистикой | ✅ |
| GET | `/api/v1/employer/needs/{id}/test-plan` | 🅴 | Каким будет тест под эту потребность (T-4) | ✅ |
| POST | `/api/v1/employer/needs/{id}/shortlist` | 🅴 | Сформировать или уточнить подборку; тело — фильтры `{grades, allow_adjacent_grades, must_skills, nice_skills, work_formats, city, fsp_only, adults_only, max_expectation, limit}`; закреплённые сохраняются. Признак `text` в объяснении — «Близость описаний»: по смыслу (косинус эмбеддингов потребности и профиля), если векторы у обоих посчитаны, иначе по ключевым словам; `explanation_ru` говорит, как именно | ✅ |
| GET | `/api/v1/employer/needs/{id}/shortlist` | 🅴 | Подборка с объяснениями | ✅ |
| PATCH | `/api/v1/employer/needs/{id}/shortlist/{candidate_id}` | 🅴 | `{pinned?, dismissed?}` | ✅ |
| GET | `/api/v1/employer/categories` | 🅴 | Все категории: число кандидатов, с ФСП, медианные ожидания | ✅ |
| GET | `/api/v1/employer/candidates` | 🅴 | Поиск по банку: `specialization` (обязательно), `grade, skills, fsp_only, adults_only, work_format, city, max_expectation, q, need_id, limit, offset`. Без `need_id` — порядок по силе профиля и `city` не учитывается, с `need_id` — по соответствию потребности. `adults_only` («Только 18+») отсекает кандидатов 14–17 лет — фильтр, не штраф | ✅ |
| GET | `/api/v1/employer/candidates/{id}` | 🅴 | Анонимный профиль; `?need_id=` → объяснение под потребность. `candidate.minor` — отметка «14–17 лет», даты рождения нет; `candidate.result_confirmed` — модератор подтвердил тест, давший категорию («Результат подтверждён»). Скрытый кандидат — 404, если у компании с ним нет приглашения или отклика. Просмотр учитывается в статистике спроса кандидата | ✅ |
| GET | `/api/v1/employer/candidates/{id}/contacts` | 🅴 | Контакты — только при принятом приглашении или отклике (иначе 403 `contacts_not_granted`); пишет журнал | ✅ |
| GET / POST | `/api/v1/employer/invitations` | 🅴 | Список `?status=&cursor=&limit=` — как у кандидата: новые сверху, `{items, next_cursor, by_status}` / отправить `{candidate_id, need_id?, vacancy_id?, title, message, salary: {min, max}, work_format?, contact}`; 409 `invitation_quota_exceeded` (лимит — по уровню доверия, 5–30 приглашений, созданных за 7 дней, включая отозванные), `invitations_frozen` (модератор приостановил приглашения, в заголовке — дата), `invitation_already_active`, `candidate_not_available` (нет категории или поиск закрыт); скрытый кандидат — 404. Вилка сверяется с рынком категории кандидата, отметка пишется в приглашение (не блокирует) | ✅ |
| GET | `/api/v1/employer/invitations/{id}` | 🅴 | Приглашение со статусом и историей событий | ✅ |
| POST | `/api/v1/employer/invitations/{id}/withdraw` | 🅴 | Отозвать; 404 — не приглашение этой компании; 409 `invitation_closed` — уже не ждёт ответа | ✅ |
| GET | `/api/v1/employer/invitations/stats` | 🅴 | Квота (`quota_limit` по уровню доверия, `quota_max`, `trust_level`), отклик кандидатов, причины отказов; `hiring` — сколько приглашений и откликов дошли до собеседования, оффера, найма, отказа (по данным ATS) | ✅ |
| GET / POST | `/api/v1/employer/vacancies` | 🅴 | Вакансии / создать; с `Idempotency-Key` повтор и двойной клик возвращают 200 и уже созданную вакансию (422 `idempotency_key` — ключ длиннее 100 символов) | ✅ |
| GET / PUT | `/api/v1/employer/vacancies/{id}` | 🅴 | Вакансия; статус `draft` / `published` / `closed`. Каждое сохранение подписывает текст; публикация пишет отметки антифейка (вилка, дубль своей, копия чужой) — не блокирует. `warnings` — отметки вакансии словами (`TrustWarning`: `kind`, `text`); `paraphrase_of_other` («по смыслу похоже на вакансию другой компании», без её названия) появляется через несколько секунд после публикации, когда посчитан вектор текста, и квоту не снижает. `unpublished_by_moderator_at` — вакансию снял модератор: править можно, опубликовать — 409 `vacancy_unpublished` | ✅ |
| GET | `/api/v1/employer/vacancies/{id}/applications?cursor=&limit=` | 🅴 | Отклики, новые сверху, `{items, next_cursor}`: код и категория кандидата, сопроводительное письмо, статус. Контакты — через `/employer/candidates/{id}/contacts` (отклик — основание). 404 — не вакансия этой компании | ✅ |
| POST | `/api/v1/employer/applications/{id}/status` | 🅴 | `viewed` / `accepted` / `rejected` в любом порядке, пока кандидат не отозвал отклик | ✅ |
| GET / POST | `/api/v1/employer/tasks` | 🅴 | Регулярные задания | ✅ |
| POST | `/api/v1/employer/tasks/{id}/active` | 🅴 | Приостановить или возобновить раздачу | ✅ |
| GET | `/api/v1/employer/tasks/{id}/submissions?cursor=&limit=` | 🅴 | Решения, новые сверху, `{items, next_cursor}`; 404 — не задание этой компании | ✅ |
| POST | `/api/v1/employer/tasks/{id}/submissions/{sid}/review` | 🅴 | Оценка решения | ✅ |

## Доверие и антифейк

Сигналы не блокируют действия: предупреждают работодателя, меняют уровень доверия и квоту, попадают в модерацию (`docs/architecture.md` §11.1).

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/employer/trust` | 🅴 | Уровень доверия (`frozen` / `low` / `new` / `standard` / `high`), квота за 7 дней = база + бонусы + штрафы (при заморозке — 0, `frozen_until`), причины с вкладом, советы, статус домена с подсказкой, правило словами; `moderation` — решения модератора о компании, её вакансиях, приглашениях и жалобах на них: что произошло и комментарий, без автора жалобы. Перепроверяет домен | ✅ |
| GET | `/api/v1/employer/salary-check?specialization&grade&min&max` | 🅴 | Рынок категории (медиана и 90-й перцентиль ожиданий, при ≥ 5) — по тем же кандидатам, что карточки категорий: опубликованные (14–17 лет — с согласием представителя), поиск не закрыт; мягкие предупреждения `salary_above_market`, `salary_wide_range`; 422 при неверной вилке | ✅ |
| POST | `/api/v1/employer/vacancies/similar` | 🅴 | `{title, description, exclude_id?}` → свои черновики и опубликованные вакансии со сходством ≥ 0,6 (чужие не показываются) | ✅ |

Очередь модерации с действиями — раздел «Модерация» ниже.

## Модерация

Кабинет модератора (`docs/architecture.md` §5.7, §11.1). Тег Swagger «Модерация», роль `admin` — в интерфейсе «Модератор»; чужая роль — 403 `wrong_role`, без сессии — 401. Списки — курсор: ответ `{items, next_cursor}`, следующая страница — `?cursor=<next_cursor>`, `limit` 1–100 (по умолчанию 30); курсор не из ответа — 400 `invalid_query`. Комментарий к решению или действию обязателен, до 2000 символов (422 `validation_failed`, поле `comment`). Каждое решение, действие и просмотр записи пишется в журнал `moderation_actions` в той же транзакции.

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/admin/overview` | admin | Счётчики: тесты, требующие подтверждения, с замечаниями, с видео, решения за неделю; жалобы, вакансии и приглашения с отметками, компании с замечаниями, снятые вакансии, замороженные компании; действия и просмотры записей за неделю | ✅ |
| GET | `/api/v1/admin/proctoring?filter=&cursor=&limit=` | admin | Очередь завершённых тестов, старые сверху. `filter`: `needs_review` (по умолчанию: «требует подтверждения» и без решения), `remarks` (уходы, вставки, выход из полноэкранного режима, без решения), `video` (все с камерой). У строки — анонимный код, категория, `l_z`, замечания, состояние видео (`off` / `declined` / `kept` / `purged`), действует ли грейд теста, решение | ✅ |
| GET | `/api/v1/admin/proctoring/{session_id}` | admin | Карточка теста для разбора: результат и порог, что тест делает с категорией сейчас, person-fit с разбором, ответы по порядку со сложностью `b`, временем показа и ответа, события браузера с заданием на экране, видео (состояние, срок удаления, даты просмотров), решение и `allowed` — какие решения возможны. Без ФИО, контактов и email кандидата | ✅ |
| GET | `/api/v1/admin/proctoring/{session_id}/recording` | admin | Запись любой сессии потоком. Строка `view_recording` в журнале пишется до первого байта; `HEAD` только проверяет, что файлы есть, и просмотром не считается. 404 `recording_unavailable` — без камеры, удалена по сроку или файлов нет | ✅ |
| POST | `/api/v1/admin/proctoring/{session_id}/decision` | admin | `{decision: confirm\|annul\|retest, comment}` → карточка теста. `confirm` снимает «требует подтверждения», работодатель видит `result_confirmed`; `annul` аннулирует присвоение теста: категория — по последнему действующему присвоению или никакая, навыки только из этого теста теряют источник `test`, грейд закрыт на 30 дней от решения; `retest` — то же плюс немедленная пересдача той же попытки, только с камерой. 409 `already_decided`, `session_not_finished`, `nothing_to_annul`, `retest_not_applicable` (результат уже заменён более поздним), `proctoring_off`. В той же транзакции соискатель получает уведомление `result_confirmed` / `result_annulled` / `retest_ordered` с комментарием | ✅ |
| GET | `/api/v1/admin/antifake?view=&kind=&cursor=&limit=` | admin | Очередь антифейка: жалобы, вакансии и приглашения с отметками, компании с отрицательными сигналами — по времени попадания в очередь (кабинет группирует загруженное по компаниям: первой идёт та, чей сигнал ждёт дольше). `view`: `open` (по умолчанию) или `sanctions` (снятые вакансии и замороженные компании); `kind`: `company` / `vacancy` / `invitation` / `complaint`. У элемента — причины словами, компания с уровнем доверия и квотой, текст жалобы без автора, `actions` — что можно сделать | ✅ |
| POST | `/api/v1/admin/antifake/actions` | admin | `{kind, id, action: dismiss\|unpublish\|freeze\|restore, comment, days?}` → `{company_id, complaints_resolved, frozen_until}`. `dismiss` — отметка перестаёт снижать квоту, жалоба закрыта как необоснованная, компания уходит из очереди до нового сигнала; `unpublish` — опубликованная вакансия (или вакансия из жалобы) закрыта без права публикации, жалобы на неё подтверждены; `freeze` — `days` 1–90, квота компании 0, жалобы на компанию и её приглашения подтверждены; `restore` — вакансия снова опубликована (её отметки сняты) или заморозка снята. В той же транзакции компания получает уведомление: `vacancy_unpublished`, `invitations_frozen`, `sanction_lifted` или (отклонённая жалоба) `complaint_resolved`, а запомненные уровень доверия и квота обновляются — повторного `trust_changed` о том же не будет. 404; 409 `action_not_applicable`; 422 `comment`, `days` | ✅ |
| GET | `/api/v1/admin/log?action=&target_kind=&moderator_id=&subject=&cursor=&limit=` | admin | Журнал модерации, новые сверху: кто (email модератора), действие, объект с названием на момент действия, код кандидата или компания, комментарий, детали словами (срок заморозки, закрыто жалоб, категория после аннулирования). `subject` — id объекта, кандидата или компании либо код кандидата «K-XXXXXX» (регистр, пробелы и дефис не важны); незнакомый код — пустой журнал; ни id, ни код — 400 `invalid_query` | ✅ |
| GET | `/api/v1/admin/trust` | admin | Прежний список отмеченного: компании с отрицательными сигналами (от низкой квоты), вакансии и приглашения с непросмотренными отметками (до 200 каждого) с причинами словами | ✅ |

## Интеграции: кабинет работодателя

Настройка ATS компании (`docs/architecture.md` §11.2). Без профиля компании — 409 `company_profile_required`.

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/employer/integrations` | 🅴 | Вебхук (адрес, начало секрета, когда выпущен, до когда действует прежний), счётчики доставок, число действующих ключей, адрес API для ATS, каталог событий и скоупов | ✅ |
| PUT | `/api/v1/employer/integrations/webhook` | 🅴 | `{url}`. Первое сохранение — 201 и `secret` (показывается один раз); дальше — 200, меняется только адрес. 422: не https (http — только хостам из `WEBHOOK_DEV_HOSTS`), логин в URL, адрес во внутренней или служебной сети, хост не находится в DNS. 409 `webhook_changed` — вебхук изменили в другой вкладке | ✅ |
| DELETE | `/api/v1/employer/integrations/webhook` | 🅴 | Отключить: адрес и секрет стираются, события из очереди отменяются. 404 — вебхука нет | ✅ |
| POST | `/api/v1/employer/integrations/webhook/rotate` | 🅴 | Новый секрет (один раз); сутки события подписаны и новым, и прежним | ✅ |
| POST | `/api/v1/employer/integrations/webhook/test` | 🅴 | Отправить `integration.test` сразу и вернуть ответ ATS: `{event_id, delivered, status_code, latency_ms, error}`. 404 — нет вебхука; 429 — больше 10 в минуту | ✅ |
| GET | `/api/v1/employer/integrations/deliveries?status=&cursor=&limit=` | 🅴 | События, новые сверху, `{items, next_cursor}` (по умолчанию 50, до 200), со статусом `pending` / `delivered` / `failed` (`status` — только этот), следующей попыткой и журналом попыток: код, задержка, ошибка. Завершённые события хранятся 30 дней | ✅ |
| POST | `/api/v1/employer/integrations/deliveries/{id}/retry` | 🅴 | «Отправить ещё раз» (в любом статусе): событие снова в очереди с новым суточным окном → 202. 404 — нет события, 409 `webhook_missing` | ✅ |
| GET / POST | `/api/v1/employer/integrations/keys` | 🅴 | Ключи (действующие сверху: название, префикс `fsp_live_ab12cd34`, скоупы, создан, последний запрос, отозван) / создать `{name, scopes[]}` → 201 и `secret` один раз. 409 `api_key_limit` — больше 10 действующих | ✅ |
| DELETE | `/api/v1/employer/integrations/keys/{id}` | 🅴 | Отозвать ключ — перестаёт работать сразу | ✅ |

## Интеграции ATS (API-ключ)

Тег Swagger «Интеграции ATS». Лимит — 120 запросов в минуту на ключ (429). Ошибки ключа: 401 `api_key_required` (нет ключа или это не `fsp_live_…`), 401 `api_key_invalid` (не найден или отозван), 403 `insufficient_scope`.

| Метод | Путь | Скоуп | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/integrations/whoami` | любой | Компания и скоупы ключа — проверка подключения; 404 — компании больше нет | ✅ |
| GET | `/api/v1/integrations/invitations?status=&cursor=&limit=` | `invitations:read` | Опрос вместо вебхука: приглашения, чей статус изменился после курсора, по возрастанию, `{items, next_cursor, next_since}`; `limit` — по умолчанию 50, до 200. `cursor` — непрозрачная позиция «время изменения + id»: следующий вызов передаёт `next_cursor`; пустая страница возвращает тот же курсор, полная (`limit` строк) значит, что ждут ещё. Изменения последних секунд (`ATS_POLL_LAG_SECS`, по умолчанию 5) и транзакций, которые ещё пишут, придут следующим вызовом — изменение, которое сохраняется прямо во время опроса, не перепрыгивается (docs/architecture.md §11.2). Прежний `since` (RFC 3339) работает как курсор «после этого момента», `next_since` — время последней строки; новым клиентам — `cursor`. До принятия кандидат — только код и категория; после — ещё `id` и `contacts_available: true`. С `outcomes`. 400 `invalid_query` — курсор не из ответа | ✅ |
| GET | `/api/v1/integrations/candidates/{id}` | `candidates:read` | Полный профиль с ФИО и контактами — только при принятом приглашении или отклике кандидата этой компании, иначе 404. Каждый вызов — в журнал доступа (`channel: api`) | ✅ |
| GET | `/api/v1/integrations/candidates/{id}/profile.pdf` | `candidates:read` | Стандартизированный PDF-профиль на тех же условиях | ✅ |
| POST | `/api/v1/integrations/outcomes` | `outcomes:write` | `{invitation_id \| application_id, stage: interview\|offer\|hired\|rejected, reason?, at}` → 201; тот же этап повторно → 200 (обновляет причину и время). 404 — чужое; 409 `outcome_not_allowed` — приглашение не принято или отклик отозван; 422 — оба id или ни одного (поле `invitation_id`), `at` позже чем через 5 минут или старше года, причина длиннее 500 | ✅ |
| POST | `/api/v1/integrations/needs` | `needs:write` | `{external_id?, title?, text, salary?}` → 201 `{need_id, external_id, shortlist_url, …, notes}` и `Location`: текст вакансии из ATS разбирается в потребность; подборка строится при открытии `shortlist_url`. `external_id` — id вакансии в ATS (до 200 символов, уникален в компании): повтор с ним — и одновременная отправка — ничего не создаёт и возвращает 200 и потребность, импортированную первой. 422 — пустой текст или длиннее 20 000 символов, не распознана специализация, нет вилки или она неверна, `external_id` длиннее 200 символов | ✅ |

## Вебхуки (исходящие)

`POST` на адрес компании, тело — схема `WebhookEvent` в OpenAPI: `{id, type, created_at, company_id, data}`. Заголовки: `FSP-Event-Id` (= `id`, для идемпотентности: доставка «хотя бы один раз»), `FSP-Event-Type`, `FSP-Signature: t=<unix>,v1=<hex>[,v1=<hex>]`, где `v1 = HMAC-SHA256(secret, t + "." + тело)`; второй `v1` — прежним секретом в первые сутки после смены. Получатель отклоняет `t`, отличающееся от его часов больше чем на 5 минут. Также `Content-Type: application/json`, `User-Agent: FSP-Talents-Webhooks/1.0`; таймаут соединения 5 с, ответа — 10 с. Ответ 2xx — доставлено; иначе повтор через 1 мин, 5 мин, 30 мин, 2 ч, 12 ч, пока очередной укладывается в 24 ч от создания, затем `failed` (у неотвечающего адреса — после шестой попытки, ≈ 14,6 ч). Редиректы не выполняются. Событие пишется, только если у компании в этот момент есть вебхук; позже ничего не досылается. `invitation.expired` появляется лениво — при следующем чтении приглашений.

| Событие | Когда | Кандидат в `data.candidate` |
|---|---|---|
| `invitation.sent` / `.viewed` / `.declined` / `.expired` / `.withdrawn` | смена статуса приглашения | код и категория; у `declined` — `invitation.decline_reason` и комментарий |
| `invitation.accepted` | кандидат принял | + `id` и `disclosed`: ФИО, телефон, Telegram, email, основание, ссылки на профиль и PDF |
| `application.created` / `application.status_changed` | отклик создан или сменил статус | + `id` и `disclosed` (кроме отозванного) |
| `contacts.opened` | сотрудник открыл контакты в кабинете | код, категория, `id`; основание и время — полями `data` (`basis`, `basis_id`, `opened_at`), без контактов |
| `integration.test` | кнопка «Тестовое событие» | — |

## Публичное

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/vacancies` | 🅰 | Опубликованные вакансии с фильтрами; `company_verified_domain` → отметка «Домен подтверждён» (также в `GET /candidate/invitations`) | ✅ |
| GET | `/api/v1/vacancies/{id}` | 🅰 | Вакансия | ✅ |
| GET | `/api/v1/vacancies/{id}/similar?limit=` | 🅰 | «Похожие вакансии»: `{method, items: [{vacancy, similarity}]}`, `limit` 1–12 (по умолчанию 4). `method: meaning` — ближайшие опубликованные по косинусу эмбеддингов, сначала той же специализации; `skills` — эмбеддингов нет (выключены или вектор ещё не посчитан): той же специализации по доле общих навыков. Сама вакансия в список не входит; 404 — не опубликована | ✅ |
| POST | `/api/v1/vacancies/{id}/applications` | 🅲 | Откликнуться `{cover_letter?}`. До 18 лет — только с действующим согласием представителя, иначе 409 `guardian_consent_required`: отклик открывает компании контакты; 409 `already_applied` — отклик уже есть | ✅ |
| POST | `/api/v1/reports` | 🅼 | Жалоба на вакансию, приглашение или компанию (до 20 в сутки) → 201 `{id, already_open: false}`. Одна открытая на человека и объект: повтор, пока модератор не решил, — 200 и та же жалоба с `already_open: true`, в лимит не идёт. Открыта, пока модератор не решит: отклонённая перестаёт снижать квоту компании, подтверждённая — считается свои 90 дней | ✅ |

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
| POST | `/api/v1/eval/pairs` | 🅰 | `{vacancies: [{id, text, …}], candidates: [{id, resume_text, …}], labels?: [{vacancy_id, candidate_id, relevance 0–3}], k?, now?}` → как распознана каждая вакансия, топ-k нашего подбора с объяснениями и топ-k базовой линии «ключевые слова + стаж», Precision@5/@k, nDCG@k, MAP, MRR по вакансиям и в среднем, покрытие. Тело ≤ 8 МиБ (413 `payload_too_large`), ≤ 200 вакансий, 5 000 кандидатов, 100 000 оценок, тексты ≤ 20 000 символов (422 `validation_failed`, поле в `errors`); синтаксическая ошибка JSON — 400, JSON не по схеме — 422 `invalid_json`, не тот `Content-Type` — 415 `unsupported_media_type`; `k` — 1–20, по умолчанию 10; 10 запросов за 10 минут с IP (429). Достижения ФСП кандидата (`fsp_achievements`) подтверждают навыки по тому же правилу, что в API. Близость описаний — по смыслу (эмбеддинги локальной модели), если в наборе не больше 300 текстов и эмбеддер отвечает (не дольше 60 с), иначе по ключевым словам; `text: {by_meaning, model, fallback}` в отчёте говорит, как посчитано и почему | ✅ |

## Статусы

**Приглашение:** `sent` → `viewed` → `accepted` | `declined`; из `sent`/`viewed` → `expired` (14 дней) | `withdrawn`.

**Отклик:** `sent` → `viewed` → `accepted` | `rejected`; кандидат может `withdrawn` до решения.

**Сессия теста:** `in_progress` → `passed` | `failed` | `expired` (30 минут после дедлайна текущего задания без ответа; проверяется лениво — при старте теста, чтении истории, категории и текущего задания, при ответе; `finished_at` — дедлайн + 30 минут). `expired` — брошенный тест: в правилах смены грейда он считается неудачной попыткой на свой грейд.

**Решение модератора по тесту** (`review.decision`): нет → `confirm` | `annul` | `retest`, одно и окончательное. `annul` и `retest` аннулируют присвоение грейда; сама сессия остаётся `passed`, в правилах смены грейда она — неудачная попытка с моментом решения.

**Жалоба:** открыта → закрыта модератором с решением `dismissed` (не подтвердилась) | `vacancy_unpublished` | `invitations_frozen` (подтвердилась).

**Согласие представителя** (`state`): `not_sent` (подросток ещё не подтвердил почту) → `pending` → `granted` | `declined`; из `pending` → `expired` (ссылка ответа старше 7 дней, письмо можно повторить); `granted` → `revoked`.

**Видеозапись теста** (`proctoring.status`): `recording` пока тест идёт → `recorded` | `interrupted` (может появиться и во время теста — после `camera_lost` или `recording_failed`); без камеры — `declined`. После удаления файлов (срок хранения, отзыв согласия) статус сохраняется, `purged: true`.

## Заглушка реестра ФСП (`fsp-mock`)

Сервис на `:8090` в сети compose (профиль `fsp`): Caddy его не проксирует; для демо-админки он открыт только на `127.0.0.1:8091` — порт 8090 на машине занимает `make fsp-mock` при локальной разработке.

| Метод | Путь | Описание |
|---|---|---|
| GET | `/healthz` | Живость, без авторизации |
| GET | `/participants/{fsp_id}` | Карточка участника: ФИО, регион, разряды, число соревнований, последняя активность |
| GET | `/participants/{fsp_id}/achievements` | Соревнования и разряды (модель — `docs/architecture.md` §7.2) |
| POST | `/admin/participants/{fsp_id}/achievements` | Демо: добавить достижение (тело — запись реестра, `id` можно не указывать; пустое тело — второе место на всероссийском чемпионате по алгоритмическому программированию сегодняшним числом) → 201 `{fsp_id, added, push}`; `push` — `{event_id, status, attempts, error}`: что платформа ответила на событие `achievements.changed`, или `null`, если вебхук не настроен. 404 `participant_not_found`, 409 `achievement_exists`, 422 `invalid_achievement` |
| DELETE | `/admin/participants/{fsp_id}/achievements/{id}` | Демо: убрать достижение и так же отправить событие → 200 `{fsp_id, removed, push}`; 404 `achievement_not_found` |

Доступ ко всем путям, кроме `/healthz`, — `Authorization: Bearer` со статическим токеном `FSP_REGISTRY_TOKEN` (одинаковым у API и заглушки). С настоящим реестром — токен client credentials realm `fsp` (`docs/solution.md` §7.1). Событие уходит на `FSP_REGISTRY_WEBHOOK_URL` (в compose — `http://api:8000/api/v1/fsp/registry/events`) с подписью секретом `FSP_REGISTRY_WEBHOOK_SECRET`; до трёх попыток с тем же `event_id`, пока платформа отвечает 5xx или 429. Правка живёт в памяти процесса: после перезапуска заглушка снова отдаёт `data/participants.json`.

## Демо-ATS (`ats-mock`)

Демо-сервис «ATS работодателя», порт 8082 (профиль compose `ats`, входит в `full`; локально — `make ats-mock`). Секрет вебхука и API-ключ — `ATS_DEMO_WEBHOOK_SECRET` (без него мок не стартует), `ATS_DEMO_API_KEY` из `.env`; адрес API — `ATS_API_URL`. API при старте выдаёт их компании владельца `ATS_DEMO_OWNER_EMAIL` (по умолчанию `employer@demo.fsp`): ключ (должен начинаться с `fsp_live_`) — со всеми четырьмя скоупами, вебхук — если задан `ATS_DEMO_WEBHOOK_URL`, его хост есть в `WEBHOOK_DEV_HOSTS` и вебхука у компании ещё нет.

| Метод | Путь | Описание |
|---|---|---|
| GET | `/` | Последние 200 записей (входящие события и отправленные этапы): тип, проверка подписи, кандидат; кнопки «Собеседование», «Оффер», «Нанят», «Отказ» — у `invitation.accepted` и `application.created` |
| POST | `/webhooks` | Приём вебхука: проверка `FSP-Signature` и метки времени → 200, иначе 401; повтор `FSP-Event-Id` среди последних 200 записей помечается |
| POST | `/outcomes` | Форма кнопки → `POST /api/v1/integrations/outcomes` с ключом → 303 на `/` (без ключа ничего не отправляет) |
| GET | `/events` | Те же записи в JSON, новые сверху (для тестов) |
| GET | `/healthz` | Живость |
