# web — фронтенд «ФСП Карьера»

Next.js 16 (App Router) + TypeScript strict, Tailwind CSS 4, TanStack Query, react-hook-form + zod.
Публичные страницы, кабинет кандидата (`/candidate/*`) и кабинет работодателя (`/employer/*`).

## Скрипты

```sh
npm install
npm run dev        # http://localhost:3000
npm run build      # production-сборка (output: standalone, для Docker)
npm run start      # запуск собранного приложения
npm run lint       # eslint
npm run typecheck  # tsc --noEmit
npm run gen:api    # openapi.json → src/api/schema.d.ts (см. make gen-api)
```

## Переменные окружения

| Переменная | По умолчанию | Назначение |
|---|---|---|
| `NEXT_PUBLIC_USE_MOCKS` | `1` в `.env.development`, иначе выключено | `1` — данные из встроенного мок-API в браузере, без бэкенда. Подставляется при сборке |
| `API_PROXY_URL` | `http://localhost:8000` | Куда `next dev` / `next start` проксирует `/api/*` (rewrite в `next.config.ts`). В Docker `/api` отдаёт Caddy |
| `NEXT_PUBLIC_DEMO_LOGIN` | включено | `0` — скрыть кнопки демо-входа на `/login` (в мок-режиме они видны всегда) |

Шаблон — `.env.example`. Локальные переопределения — в `.env.local` (не коммитится).

## Мок-режим

Включён по умолчанию в `npm run dev`. Все вызовы `src/api/endpoints.ts` уходят в `src/mocks/handlers.ts`:
состояние живёт в `localStorage` браузера, поэтому сквозные сценарии работают целиком —
регистрация → подтверждение email («демо-почта» на экране) → профиль → опрос → адаптивный тест (IRT/EAP,
задания генерируются шаблонами) → категория; работодатель видит нового кандидата, приглашает,
кандидат принимает — у работодателя открываются контакты.

- Демо-аккаунты: `candidate@demo.fsp` и `employer@demo.fsp`, пароль `demo12345` (кнопки на странице входа) —
  совпадают с сидом бэкенда. Кнопки скрываются в боевом режиме через `NEXT_PUBLIC_DEMO_LOGIN=0`.
- ФСП ID в демо: `FSP-000001`, `FSP-000003`, `FSP-000006`, `FSP-000010` (без достижений) — те же, что в `crates/fsp-mock`.
- «Сбросить демо-данные» — в боковой панели кабинета.
- Тест в моках короче боевого: 6–12 заданий вместо 8–20.

Чтобы работать с настоящим API: `NEXT_PUBLIC_USE_MOCKS=0` в `.env.local`, запустить `make api` и `npm run dev`.

## Устройство

```
src/api/schema.d.ts     СГЕНЕРИРОВАНО из OpenAPI бэкенда (make gen-api), руками не править
src/api/types.ts        короткие имена-алиасы над schema.d.ts
src/api/client.ts       fetch-обёртка: cookie-сессия, refresh при 401, problem+json → ApiError
src/api/endpoints.ts    одна функция на эндпоинт (+ переключатель на моки)
src/features/*/hooks.ts TanStack Query-хуки; страницы вызывают только их
src/components/ui       примитивы: Button, Field/Input/Select/MoneyInput, Chip, Card, Badge, Tabs, Dialog,
                        Stepper, ProgressBar, EmptyState, Skeleton, Alert, Toast, CodeBlock, Markdown
src/components/domain   категории, зарплата, навыки, ФСП, объяснение подбора, карточки кандидатов и вакансий
src/mocks               мок-API: сид-данные, оценивание и объяснения, адаптивный тест
src/proxy.ts            только редирект на /login без cookie `fsp_session`; права проверяет API
```

Цвета и шрифты ФСП — токены в `src/app/globals.css` (стандартная палитра Tailwind отключена).
Логотипы — `public/brand/`.

## Типы API

Типы берутся только из OpenAPI бэкенда. После изменения DTO в `crates/api`:

```sh
make gen-api          # из корня: cargo run -p api -- openapi > web/openapi.json + openapi-typescript
npm run typecheck     # компилятор покажет, какие страницы и моки нужно поправить
```

Мок-API (`src/mocks`) типизирован теми же схемами, поэтому расхождение с бэкендом ловится на `tsc`.
Запросы идут через `http()` в `src/api/client.ts` (refresh при 401, problem+json → `ApiError`);
`openapi-fetch` установлен и может заменить её без изменения сигнатур в `endpoints.ts`.
