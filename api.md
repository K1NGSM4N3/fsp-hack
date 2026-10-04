# API (рабочий контракт)

> Источник правды — OpenAPI, который генерирует бэкенд (utoipa): `GET /api/v1/openapi.json`,
> Swagger UI — `/api/docs`. Этот файл — план и соглашения на время разработки: добавил или
> поменял эндпоинт — обнови таблицу. Статусы: ⬜ план · ✅ реализован.

## Соглашения

- **База:** `/api/v1`. Ломающее изменение → новая версия `/api/v2` рядом со старой; старая отдаёт заголовки `Deprecation` и `Sunset`.
- **Формат:** JSON, `snake_case`, даты — RFC 3339 UTC, деньги — целые рубли (`i64`), id — UUID v7.
- **Аутентификация:** httpOnly cookie `fsp_at` (access JWT, 15 мин) и `fsp_rt` (refresh, путь `/api/v1/auth`). Альтернатива для API-клиентов и Swagger — `Authorization: Bearer <jwt>`.
- **CSRF:** изменяющие запросы с cookie должны иметь `Origin`, совпадающий с `PUBLIC_URL`.
- **Роли:** 🅰 анонимно · 🅲 candidate · 🅴 employer · 🅼 любой залогиненный.
- **Пагинация:** `?limit=20&offset=0` → `{ "items": [...], "total": 123 }`. Максимум `limit` — 100.
- **Ошибки:** RFC 9457 `application/problem+json`:
  ```json
  { "type": "about:blank", "title": "Смена грейда пока недоступна", "status": 409,
    "code": "grade_change_cooldown", "detail": "Следующая попытка — после 2027-01-03",
    "errors": { "field": ["сообщение"] } }
  ```
  `code` — стабильный машинный код, на него завязывается фронт. `errors` — только для 422.
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
| GET | `/api/v1/catalog/skills?q=` | 🅰 | Автодополнение навыков (с синонимами) | ⬜ |
| GET | `/api/v1/consents/current` | 🅰 | Актуальные версии документов согласия | ✅ |

## Auth и аккаунт

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| POST | `/api/v1/auth/register` | 🅰 | `{email, password, role, consents[]}` → 201, письмо с подтверждением | ✅ |
| POST | `/api/v1/auth/verify-email` | 🅰 | `{token}` → подтверждает email | ✅ |
| POST | `/api/v1/auth/resend-verification` | 🅰 | `{email}` → 204 всегда (не раскрываем наличие) | ✅ |
| POST | `/api/v1/auth/login` | 🅰 | `{email, password}` → cookie + `{access_token, expires_in, user}` | ✅ |
| POST | `/api/v1/auth/refresh` | 🅰 | По cookie `fsp_rt` → новая пара | ✅ |
| POST | `/api/v1/auth/logout` | 🅼 | Отзыв семейства refresh, очистка cookie | ✅ |
| GET | `/api/v1/me` | 🅼 | Пользователь, роль, статус онбординга | ✅ |
| DELETE | `/api/v1/me` | 🅼 | Удаление аккаунта и ПДн | ✅ |
| GET | `/api/v1/me/consents` | 🅼 | Выданные согласия | ✅ |
| POST | `/api/v1/me/consents/{document_id}/revoke` | 🅼 | Отзыв согласия | ✅ |

## Кандидат

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET / PUT | `/api/v1/candidate/profile` | 🅲 | Профиль: имя, контакты, город, формат, ожидания, о себе, стаж, навыки, отрасли | ✅ |
| POST | `/api/v1/candidate/resume` | 🅲 | multipart PDF → распознанные поля (черновик, не сохраняется в профиль сам) | ⬜ |
| GET | `/api/v1/candidate/profile.pdf` | 🅲 | Стандартизированный PDF-профиль | ⬜ |
| GET / PUT | `/api/v1/candidate/privacy` | 🅲 | Что видно работодателю, статус поиска | ✅ |
| GET | `/api/v1/me` → `candidate.onboarding_step` | 🅲 | Шаг пути: профиль → опрос → тест → категория (отдельного эндпоинта нет) | ✅ |
| POST | `/api/v1/candidate/survey` | 🅲 | Опрос: направление, специализация, стек, отрасли, самооценка тем | ✅ |
| GET | `/api/v1/candidate/survey/history` | 🅲 | История опросов | ✅ |
| GET | `/api/v1/candidate/category` | 🅲 | Текущая категория, история присвоений, когда доступна смена грейда | ⬜ |
| POST | `/api/v1/candidate/assessments` | 🅲 | `{specialization, grade}` → новая сессия (проверка политики, 409 `grade_change_cooldown`) | ⬜ |
| GET | `/api/v1/candidate/assessments` | 🅲 | История сессий | ⬜ |
| GET | `/api/v1/candidate/assessments/{id}` | 🅲 | Состояние или результат сессии | ⬜ |
| GET | `/api/v1/candidate/assessments/{id}/current-item` | 🅲 | Текущее задание (без правильного ответа), дедлайн | ⬜ |
| POST | `/api/v1/candidate/assessments/{id}/answer` | 🅲 | `{item_id, answer}` → прогресс; по завершении — результат | ⬜ |
| GET | `/api/v1/candidate/invitations` | 🅲 | Входящие приглашения | ✅ |
| GET | `/api/v1/candidate/invitations/{id}` | 🅲 | Приглашение; первое открытие → `viewed` | ✅ |
| POST | `/api/v1/candidate/invitations/{id}/accept` | 🅲 | → `accepted`, контакты раскрываются компании | ✅ |
| POST | `/api/v1/candidate/invitations/{id}/decline` | 🅲 | `{reason, comment?}` → `declined` | ✅ |
| GET | `/api/v1/candidate/market` | 🅲 | Анонимный спрос в своей категории | ⬜ |
| GET | `/api/v1/candidate/applications` | 🅲 | Свои отклики | ✅ |
| POST | `/api/v1/candidate/applications/{id}/withdraw` | 🅲 | Отозвать отклик | ✅ |
| GET | `/api/v1/candidate/tasks` | 🅲 | Регулярные задания | ⬜ |
| POST | `/api/v1/candidate/tasks/{id}/submit` | 🅲 | Решение или подход | ⬜ |

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
| GET / PUT | `/api/v1/employer/company` | 🅴 | Профиль компании | ✅ |
| GET / POST | `/api/v1/employer/needs` | 🅴 | Потребности: список / создать | ⬜ |
| GET / PUT / DELETE | `/api/v1/employer/needs/{id}` | 🅴 | Потребность | ⬜ |
| POST | `/api/v1/employer/needs/parse` | 🅴 | Свободный текст → специализация, грейд, навыки (черновик) | ⬜ |
| GET | `/api/v1/employer/needs/{id}/categories` | 🅴 | Рекомендованные категории со статистикой | ⬜ |
| POST | `/api/v1/employer/needs/{id}/shortlist` | 🅴 | Сформировать или уточнить подборку `{filters}`; закреплённые сохраняются | ⬜ |
| GET | `/api/v1/employer/needs/{id}/shortlist` | 🅴 | Подборка с объяснениями | ⬜ |
| PATCH | `/api/v1/employer/needs/{id}/shortlist/{candidate_id}` | 🅴 | `{pinned?, dismissed?}` | ⬜ |
| GET | `/api/v1/employer/categories` | 🅴 | Все категории: число кандидатов, с ФСП, медианные ожидания | ⬜ |
| GET | `/api/v1/employer/candidates` | 🅴 | Поиск по банку: `specialization, grade, skills, fsp_only, work_format, city, salary_max, q, sort` | ⬜ |
| GET | `/api/v1/employer/candidates/{id}` | 🅴 | Анонимный профиль; `?need_id=` → объяснение под потребность | ⬜ |
| GET | `/api/v1/employer/candidates/{id}/contacts` | 🅴 | Контакты — только при принятом приглашении или отклике (иначе 403 `contacts_not_granted`); пишет журнал | ✅ |
| GET / POST | `/api/v1/employer/invitations` | 🅴 | Список / отправить `{candidate_id, need_id?, vacancy_id?, title, message, salary: {min, max}, work_format?, contact}`; 409 `invitation_quota_exceeded`, `invitation_already_active` | ✅ |
| GET | `/api/v1/employer/invitations/{id}` | 🅴 | Приглашение со статусом и историей событий | ✅ |
| POST | `/api/v1/employer/invitations/{id}/withdraw` | 🅴 | Отозвать | ✅ |
| GET | `/api/v1/employer/invitations/stats` | 🅴 | Квота, отклик кандидатов, причины отказов | ✅ |
| GET / POST | `/api/v1/employer/vacancies` | 🅴 | Вакансии | ✅ |
| GET / PUT | `/api/v1/employer/vacancies/{id}` | 🅴 | Вакансия; статус `draft` / `published` / `closed` | ✅ |
| GET | `/api/v1/employer/vacancies/{id}/applications` | 🅴 | Отклики (с контактами — кандидат откликнулся сам) | ✅ |
| POST | `/api/v1/employer/applications/{id}/status` | 🅴 | `viewed` / `accepted` / `rejected` | ✅ |
| GET / POST | `/api/v1/employer/tasks` | 🅴 | Регулярные задания | ⬜ |
| GET | `/api/v1/employer/tasks/{id}/submissions` | 🅴 | Решения | ⬜ |
| POST | `/api/v1/employer/tasks/{id}/submissions/{sid}/review` | 🅴 | Оценка решения | ⬜ |

## Публичное

| Метод | Путь | Роль | Описание | Статус |
|---|---|---|---|---|
| GET | `/api/v1/vacancies` | 🅰 | Опубликованные вакансии с фильтрами | ✅ |
| GET | `/api/v1/vacancies/{id}` | 🅰 | Вакансия | ✅ |
| POST | `/api/v1/vacancies/{id}/applications` | 🅲 | Откликнуться `{cover_letter?}` | ✅ |
| POST | `/api/v1/reports` | 🅼 | Жалоба на вакансию, приглашение или компанию | ⬜ |

## Статусы

**Приглашение:** `sent` → `viewed` → `accepted` | `declined`; из `sent`/`viewed` → `expired` (14 дней) | `withdrawn`.

**Отклик:** `sent` → `viewed` → `accepted` | `rejected`; кандидат может `withdrawn` до решения.

**Сессия теста:** `in_progress` → `passed` | `failed` | `expired` (брошена > 1 часа).

## Заглушка реестра ФСП (`fsp-mock`)

Внутренний сервис, не проксируется наружу.

| Метод | Путь | Описание |
|---|---|---|
| GET | `/participants/{fsp_id}` | Карточка участника: ФИО, регион, разряды |
| GET | `/participants/{fsp_id}/achievements` | Соревнования и разряды (модель — `architecture.md` §7.2) |

Доступ — `Authorization: Bearer` с client credentials токеном realm `fsp` (в dev — статический токен `FSP_REGISTRY_TOKEN`).
