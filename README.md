# Платформа подбора ИТ-специалистов с профилем ФСП

Работодатель не ждёт откликов, а сам выходит на кандидата. Кандидаты распределены по категориям
(специализация × грейд), и категорию даёт только адаптивный тест, а не резюме. Внутри категории
выше стоят те, у кого сильнее результат теста и есть подтверждённые достижения Федерации
спортивного программирования. Зарплатная вилка видна кандидату в первом же приглашении, а контакты
кандидата работодатель получает только после его согласия.

Хакатон "Лидеры цифровой трансформации 2026", спецтрек ФСП.

**Работающее решение: https://demo.arewehireyet.ru** (демо-доступы - в таблице ниже, Swagger - `/api/docs`).

## Быстрый старт (Docker)

```sh
cp .env.example .env
docker compose -f deploy/compose.yml --profile full up -d --build   # то же самое: make up
```

На macOS без Docker Desktop движок даёт Colima из `nix develop`: сначала `colima start`.

### Демо-сервер (отдельная машина)

Стек поднимается теми же образами поверх оверлея: caddy на 80/443 с собственным сертификатом
Let's Encrypt для домена, остальное - без изменений. Локальный запуск и `make blackbox`
базовый `deploy/Caddyfile` (:8080) не затрагивают.

```sh
# 1) на сервере (чистый Ubuntu, root): firewall, swap, docker, ключи deploy
bash deploy/bootstrap.sh          # выведет публичный ключ - добавить в GitHub -> Deploy keys (read-only)

# 2) код и окружение: шаблон рендерит секреты, существующий .env не перезаписывает
git clone git@github.com:K1NGSM4N3/fsp-hack.git && cd fsp-hack
python3 deploy/render_env.py --domain demo.arewehireyet.ru

# 3) сборка и запуск (ядро + ФСП ID + демо-ATS; obs-профиль не нужен)
docker compose -f deploy/compose.yml -f deploy/compose.demo.yml \
  --profile fsp --profile ats up -d --build
```

Обновление: `git pull && docker compose -f deploy/compose.yml -f deploy/compose.demo.yml --profile fsp --profile ats up -d --build`.
Секреты `.env` генерируются один раз (`render_env.py` без `--force` откажется пересоздавать); `SITE_URL`
запекается в образ web при сборке.

Наружу открыт только Caddy (80/443), всё на одном домене:

| Путь | Что |
|---|---|
| `/` | Сайт и кабинеты |
| `/api/docs` | Swagger UI |
| `/auth/...` | ФСП ID (Keycloak): "Привязать ФСП ID" в настройках соискателя, вход `fsp-000001` ... `fsp-000010` / `demo12345`; админка Keycloak наружу не отдаётся |
| `/mail/` | Демо-почта (Mailpit): ловит письма, пока в `.env` не задан настоящий SMTP (`SMTP_URL=smtps://...`, `MAIL_FROM` - тот же ящик). Логин и пароль - `MAILPIT_UI_AUTH` в `deploy/.env` |

Остальные порты (Keycloak 8081, Mailpit 8025, демо-ATS 8082, реестр 8091) слушают только
127.0.0.1: опубликованные Docker порты идут в обход ufw, поэтому закрыты в самом оверлее.
Демо-ATS и админка Keycloak - через туннель: `ssh -L 8082:127.0.0.1:8082 -L 8081:127.0.0.1:8081 ...`,
затем http://localhost:8082 и http://localhost:8081/auth/admin.

Обслуживание - `sudo bash deploy/ops/install.sh` один раз ставит два таймера systemd:

- `fsp-backup.timer` - каждую ночь в 03:30 МСК дамп базы (`pg_dump --format=custom`) и `.env` той же ночи
  в `/var/backups/fsp-hack`, хранятся 7 дней. Без `.env` дамп бесполезен: персональные данные в нём
  зашифрованы `FIELD_ENC_KEY`. Вручную - `sudo deploy/ops/backup.sh`.
- `fsp-watchdog.timer` - раз в 2 минуты перезапускает контейнеры с проваленным healthcheck: Docker сам
  перезапускает только упавшие. Перезапуски - в `journalctl -t fsp-watchdog`.

Восстановление: `docker compose -f deploy/compose.yml -f deploy/compose.demo.yml exec -T postgres sh -c
'pg_restore -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists' < /var/backups/fsp-hack/db-....dump`
при том же `.env`, что в бэкапе.


| Что | Где |
|---|---|
| Приложение | http://localhost:8080 |
| Swagger UI | http://localhost:8080/api/docs |
| Письма (подтверждение email, письма законному представителю) | http://localhost:8025 |
| Keycloak (заглушка ФСП ID), админка - `/admin` | http://localhost:8081 |
| Демо-ATS работодателя (вебхуки и этапы найма) | http://localhost:8082 |
| Grafana (дашборд API, просмотр без входа) | http://localhost:3001 |

### Сервисы и профили

| Сервис | Порт наружу | Профиль | Назначение |
|---|---|---|---|
| `caddy` | 8080 | ядро | Единый вход: `/api/*`, `/healthz`, `/readyz` -> api, остальное -> web; `/metrics` наружу закрыт |
| `web` | нет (3000 в сети compose) | ядро | Next.js: публичный сайт и кабинеты |
| `api` | нет (8000) | ядро | Rust API; миграции и демо-данные при старте |
| `embedder` | нет (8093) | ядро | Эмбеддинги текстов: модель `multilingual-e5-small` (MIT) на CPU, веса - в образе. Нужен близости описаний в подборе, отметке "пересказ чужой вакансии" и "Похожим вакансиям"; без него всё работает на ключевых словах |
| `postgres` | нет (5432) | ядро | PostgreSQL 17 (образ `pgvector/pgvector`) |
| `valkey` | нет (6379) | ядро | Лимиты запросов, отозванные токены, поколение кеша кандидатов |
| `mailpit` | 8025, SMTP - `127.0.0.1:1025` | ядро | Почта для демо |
| `keycloak` | 8081 | `fsp` | ФСП ID: realm `fsp`, клиент `fsp-talent` |
| `fsp-mock` | `127.0.0.1:8091` | `fsp` | Заглушка реестра достижений ФСП; демо-админка шлёт в API события об изменениях |
| `ats-mock` | 8082 | `ats` | Демо-ATS работодателя |
| `prometheus`, `loki`, `alloy` | - | `obs` | Метрики API и логи контейнеров |
| `grafana` | 3001 | `obs` | Дашборды |

`full` включает все профили. Без профилей поднимается только ядро; профили складываются:
`--profile fsp --profile ats`. Без `ats` события для демо-компании не доставляются и видны в журнале
доставок как ошибки. Демо-компания `employer@demo.fsp` сразу подключена к демо-ATS: события
о приглашениях приходят туда с подписью, а кнопки "Собеседование", "Оффер", "Нанят", "Отказ"
возвращают этап кандидату.

### Эмбеддинги: память и выключатель

Сервис `embedder` занимает до ~250 МБ под нагрузкой (своя память - ~180 МБ, остальное - страницы файла весов; лимит контейнера - 384 МБ) и считает профиль за 50-90 мс на двух потоках; запросы API его не ждут - векторы считает фоновый воркер после записи текста, демо-данные - за минуту-две после первого старта. Тексты кандидатов и компаний не покидают сервер: модель работает внутри сети compose.

- `EMBEDDINGS=off` в `.env` - без эмбеддингов: близость описаний по ключевым словам, похожие вакансии по навыкам, пересказы не ищутся. Сервис тогда можно не поднимать: `docker compose -f deploy/compose.yml --profile full up -d --build --scale embedder=0`.
- Остановленный эмбеддер ничего не ломает: посчитанные векторы продолжают работать, новые тексты ждут в очереди и считаются, когда сервис вернётся.
- **Эмбеддер без сети.** Образ скачивает веса с Hugging Face при сборке (ревизия закреплена, sha256 проверяется). Без доступа к Hugging Face: скачайте веса на машине с сетью (`make model` -> `models/multilingual-e5-small`, три файла и `REVISION`) и перенесите каталог, соберите образ с `EMBEDDER_FETCH=0` (`EMBEDDER_FETCH=0 make up`) и смонтируйте каталог в сервис: `volumes: ["../models:/models:ro"]` у `embedder` в `deploy/compose.yml` (путь внутри - `EMBEDDER_MODEL_DIR=/models/multilingual-e5-small`, он задан в образе).

### Демо-аккаунты

При первом запуске на пустой базе API заполняет демо-данные (`SEED_DEMO=true` в `.env`).
Пароль у всех демо-аккаунтов - `demo12345`:

| Роль | Вход |
|---|---|
| Работодатель (6 потребностей, подключён к демо-ATS) | `employer@demo.fsp` |
| Работодатели с подтверждённым доменом | `hr@pixel-manufactura.example`, `hr@oblakostroy.example`, `hr@medsignal.example`, `it-hr@logplus.example`, `career@shield-key.example` |
| Кандидат | `candidate@demo.fsp`, `cand1@demo.fsp` ... `cand239@demo.fsp` (один из номеров вместо этого входит как `minor@demo.fsp`) |
| Кандидат 14-17 лет, ждёт согласия законного представителя | `minor@demo.fsp` |
| Кандидат, чей тест ждёт решения модератора (тоже один из номеров `cand...`) | `review@demo.fsp` |
| Модератор: разбор спорных тестов, антифейк и жалобы, журнал (кабинет `/admin`) | `moderator@demo.fsp` |
| Участник ФСП (вход в ФСП ID на Keycloak) | `fsp-000001` ... `fsp-000010` |

Администратор Keycloak - `admin` / `admin` (`KC_BOOTSTRAP_ADMIN_*` в `.env`), Grafana - `admin` / `admin`.
Роль модератора (`admin` в API) регистрацией не выдаётся - только в базе; в демо это `moderator@demo.fsp`. В его очередях сразу есть работа: семь тестов на разбор, две вакансии с отметками антифейка и три жалобы. Каждое решение модератора приходит соискателю или компании уведомлением в кабинет и письмом в Mailpit.

### Демо: реестр ФСП сам сообщает о новом результате

`candidate@demo.fsp` привязан к участнику `FSP-000011`. Добавьте ему результат в заглушке реестра - она отправит в API подписанное событие `achievements.changed`, API заберёт достижения и пришлёт соискателю уведомление "Новые достижения ФСП в профиле":

```sh
curl -X POST -H 'Authorization: Bearer dev-registry-token' \
  http://127.0.0.1:8091/admin/participants/FSP-000011/achievements
```

Пустое тело - второе место на всероссийском чемпионате по алгоритмическому программированию сегодняшним числом; можно передать свою запись в формате реестра. В ответе `push.status: 202` - API принял событие. Колокольчик в кабинете соискателя покажет уведомление, в настройках ("ФСП ID") появится новый результат, а у навыков "Алгоритмы" и "Структуры данных" - подтверждение ФСП. Убрать запись - `DELETE .../achievements/{id}` с тем же токеном. Нужен `FSP_REGISTRY_WEBHOOK_SECRET` в `.env` (есть в `.env.example`).

## Локальная разработка без Docker

Нужны Rust ≥ 1.85, Node.js ≥ 20.9, PostgreSQL ≥ 14 - или `nix develop`, где есть всё, включая
Valkey, sqlx-cli, pandoc, typst, Colima и Docker.

```sh
make db                  # создать базу fsp_hack (нужен PostgreSQL с расширением pgvector)
make embedder            # эмбеддер на :8093; веса скачает `make model` (~490 МБ, sha256 проверяется)
SEED_DEMO=true make api  # API на :8000: миграции при старте, демо-данные на пустой базе
make web                 # фронтенд на :3000, /api/* проксируется на :8000
make test                # модульные тесты: без базы и сети
```

`make api` не читает `.env`: он ищет эмбеддер на `http://localhost:8093` (`EMBEDDINGS=off make api` - без него), без `VALKEY_URL` лимиты и отозванные токены живут в памяти процесса,
без `MAIL_MODE=smtp` ссылки из писем печатаются в лог API, а `FSP_DEV_LINK=true` разрешает привязать
ФСП ID вводом номера, без Keycloak. Реестр ФСП локально - `make fsp-mock` (:8090) и
`FSP_REGISTRY_URL=http://localhost:8090`; демо-ATS - `make ats-mock` (:8082). Фронтенд без бэкенда -
`NEXT_PUBLIC_USE_MOCKS=1` в `web/.env.local` (`web/README.md`).

Доступ к базе - sqlx: запросы проверяются по схеме при компиляции, а сборка идёт по метаданным из
`.sqlx/` и базы не требует. Изменили SQL или миграцию - `make db-prepare` (нужна база из `DATABASE_URL`)
и закоммитьте `.sqlx/` вместе с кодом. Изменили DTO - `make gen-api`.

Остальные команды и правила разработки - в `AGENTS.md`.

## Тесты

Два вида, оба без доступа к внутренностям друг друга:

```sh
make test       # модульные: cargo test --workspace (554 теста) и проверка типов web; Postgres не нужен
make blackbox   # black-box: свежий стек в Docker + Playwright по публичным портам
make blackbox DOWN=1                       # то же, стек после прогона удаляется
make blackbox ARGS="specs/minors.spec.ts"  # один файл; любые аргументы playwright test
make blackbox-degraded                     # после make blackbox: эмбеддер остановлен, приложение работает на ключевых словах
make blackbox-down                         # удалить стек black-box тестов
```

`make blackbox` останавливает стек `make up` (его данные остаются), поднимает отдельный compose-проект
`fsp-hack-blackbox` с чистыми томами на тех же портах - :8080, :8025, :8082, :8091 - и демо-сидом,
собирает образы из текущего кода, ждёт готовности и запускает `tests/blackbox`. Отличие от демо-стека
одно - `deploy/compose.blackbox.yml`: письма-уведомления ждут 3 секунды вместо 2 и 10 минут, без
Keycloak и наблюдаемости. Нужен `.env` (копируется из `.env.example`, если его нет), Node.js ≥ 20.9 и
Docker; браузер Chromium ставит сам Playwright. Тесты ходят только туда, куда ходит пользователь или
внешняя система: веб и API, почтовый ящик Mailpit, демо-ATS, админка реестра ФСП. После прогона стек
остаётся поднятым, отчёт - `tests/blackbox/playwright-report/index.html` (`npx playwright show-report`
в `tests/blackbox`). Вернуться к демо-стеку: `make blackbox-down && make up`.

Против уже поднятого стека тесты запускаются и без пересборки: `cd tests/blackbox && npm ci && npx playwright test`.
Повторный прогон на том же стеке заденет уже потраченные демо-данные (очередь модерации, согласие
представителя у `minor@demo.fsp`) - для чистого результата нужен `make blackbox`.

## Документация

- `docs/solution.md` - описание решения для жюри: архитектура, механики, валидация, 152-ФЗ, API, запуск; `make docs` собирает из него `docs/solution.pdf` и `docs/solution.docx`
- `docs/presentation.pdf`, `docs/presentation.pptx` - презентация
- `docs/architecture.md` - зафиксированные решения: стек, механики тестирования и подбора, ФСП, безопасность, модель данных, развёртывание
- `docs/api.md` - соглашения API, вебхуки, статусы и правила, которых нет в спецификации; сами методы - в OpenAPI: `/api/v1/openapi.json`, Swagger UI - `/api/docs`
- `docs/ТЗ.pdf` - техническое задание организаторов; покрытие требований - `docs/solution.md`, раздел 1.2
- `web/README.md` - фронтенд: страницы, устройство, анимации, переменные окружения
- `docs/validation/` - отчёты валидации тестирования (`assessment.md`), подбора (`matching.md`) и пересказов вакансий (`paraphrase.md`), `make validate`; сравнение моделей эмбеддингов (`embeddings.md`) - `cargo run -p validate --release -- models` (нужны обе модели: `make model MODEL=rubert-tiny2`)
- `docs/validation/pairs-example/` - проверка подбора на своих данных: пример входа и отчёта; запуск - `cargo run -p validate --release -- pairs --input data.json --out report.md` или `POST /api/v1/eval/pairs` в Swagger (`docs/solution.md`, раздел 6.4)
- `AGENTS.md` - структура репозитория, команды, правила кода

## Лицензия

MIT - см. `LICENSE`. Все зависимости - под открытыми лицензиями (MIT, Apache-2.0, BSD, ISC, MPL, OFL), веса модели эмбеддингов `intfloat/multilingual-e5-small` - MIT; исключение - GSAP (бесплатная Standard License), подробности в `docs/solution.md`, раздел 11.
