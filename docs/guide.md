# Руководство: запуск, настройка, эксплуатация

Как поднять платформу, что настраивается в `.env`, чем заменить демо-заглушки в боевой установке и что делать, если что-то не работает. Что платформа делает и почему устроена именно так - в описании решения (`README.md`); здесь - как с ней работать.

## 1. Что где лежит

### 1.1. Документы

| Документ | Что там |
|---|---|
| `README.md` | Описание решения: функции, архитектура, механики тестирования и подбора, валидация, безопасность и 152-ФЗ, интеграции. `make docs` собирает из него `docs/solution.pdf` и `docs/solution.docx` |
| `docs/guide.md` | Это руководство |
| Swagger UI `/api/docs` | Все методы API: пути, роли, схемы запросов и ответов, коды ошибок. Спецификация - `/api/v1/openapi.json`, её копия в репозитории - `web/openapi.json` |
| `docs/api.md` | Соглашения API и правила, которых не видно из схем: ошибки, пагинация, идемпотентность, лимиты, вебхуки, статусы, заглушки внешних систем |
| `docs/validation/` | Отчёты валидации тестирования и подбора (`make validate`) и пример проверки на своих данных |
| `.env.example` | Шаблон настроек; все переменные описаны в разделе 3 |
| `web/README.md` | Устройство фронтенда: рендеринг, сессия, мок-режим, доступность, SEO, анимации |
| `docs/ТЗ.pdf` | Техническое задание организаторов; покрытие требований - `README.md`, раздел 1.2 |

### 1.2. Репозиторий

| Каталог | Что там |
|---|---|
| `crates/api` | Сервер API (Rust, axum): HTTP, вход, OpenAPI, уведомления, интеграции, фоновые задачи |
| `crates/assessment`, `crates/matching`, `crates/catalog` | Логика без HTTP и базы: адаптивный тест, подбор и скоринг, справочники |
| `crates/db` | Запросы к PostgreSQL (sqlx) |
| `crates/embedder` | Сервис эмбеддингов текстов (`POST /embed`) |
| `crates/fsp-mock`, `crates/ats-mock` | Заглушка реестра ФСП и демо-ATS работодателя |
| `crates/validate` | Валидация на синтетике и проверка подбора на своих данных |
| `db/migrations` | Миграции схемы; API применяет их при старте |
| `web/` | Next.js: публичный сайт и кабинеты |
| `deploy/` | docker compose, Dockerfile'ы, Caddy, realm Keycloak, Prometheus, Loki, Alloy, Grafana, скрипты демо-сервера |
| `tests/blackbox/` | Сквозные тесты (Playwright) против поднятого стека |
| `models/` | Веса модели эмбеддингов для запуска без Docker (`make model`, в git не входят) |

## 2. Запуск

### 2.1. Docker

Требования: Docker Engine с Compose v2 (`docker compose`), 4 ГБ свободной памяти под профиль `full` (сумма лимитов контейнеров - 3776 МиБ, ядро - 1984 МиБ). Первой сборке нужен доступ к Docker Hub, crates.io, npm и Hugging Face (веса модели эмбеддингов; без Hugging Face - раздел 2.5).

```sh
git clone https://github.com/K1NGSM4N3/fsp-hack && cd fsp-hack
cp .env.example .env
make up
```

`make up` - это `docker compose -f deploy/compose.yml --profile full up -d --build`. Значения в `.env.example` - открытые ключи и пароли для локального демо; для общего сервера - раздел 4, все переменные - раздел 3.

**Профили compose.** Без профиля поднимается только ядро; профили складываются (`--profile fsp --profile ats`), `full` включает все.

| Профиль | Сервисы | Что даёт |
|:--|:--|:--|
| ядро | `caddy`, `web`, `api`, `embedder`, `postgres`, `valkey`, `mailpit` | Сайт, кабинеты, API, эмбеддинги, PostgreSQL 17 с pgvector, Valkey (лимиты, отозванные токены, поколение кеша), демо-почта |
| `fsp` | `keycloak`, `fsp-mock` | ФСП ID (realm `fsp`, клиент `fsp-talent`) и заглушка реестра достижений с демо-админкой. Без профиля кнопка "Привязать ФСП ID" отвечает 502 `fsp_unavailable` |
| `ats` | `ats-mock` | Демо-ATS работодателя. Без профиля события демо-компании не доставляются и видны в журнале доставок как ошибки |
| `obs` | `prometheus`, `loki`, `alloy`, `grafana` | Метрики, логи контейнеров, дашборд API |
| `full` | все | Полное демо |

**Адреса.**

| Адрес | Что там | Профиль |
|:--|:--|:--|
| http://localhost:8080 | Сайт и кабинеты | ядро |
| http://localhost:8080/api/docs | Swagger UI; спецификация - `/api/v1/openapi.json` | ядро |
| http://localhost:8025 | Mailpit: все письма платформы (SMTP - `127.0.0.1:1025`) | ядро |
| http://localhost:8081 | Keycloak (ФСП ID), админка - http://localhost:8081/admin | `fsp` |
| http://127.0.0.1:8091 | HTTP-админка заглушки реестра ФСП (JSON, Bearer-токен), только с этой машины | `fsp` |
| http://localhost:8082 | Демо-ATS: принятые вебхуки и кнопки этапов найма | `ats` |
| http://localhost:3001 | Grafana, дашборд API открывается без входа | `obs` |

API (8000), web (3000), эмбеддер (8093), PostgreSQL и Valkey доступны только внутри сети compose; `/metrics` Caddy наружу не отдаёт. Сайт открывайте по адресу из `PUBLIC_URL` (по умолчанию `http://localhost:8080`): с другого адреса изменяющие запросы отклоняются проверкой Origin (раздел 6).

**Первый старт.** Дольше всего идёт сборка образов: release-сборка Rust, сборка Next.js и скачивание весов `multilingual-e5-small` (~490 МБ, ревизия закреплена, sha256 проверяется). Повторные сборки берут слои из кеша. Затем API применяет миграции и на пустой базе заполняет демо-данные (`SEED_DEMO=true`). Keycloak готов примерно через минуту. Векторы демо-данных эмбеддер считает за минуту-две после старта; до этого подбор работает на ключевых словах.

```sh
docker compose -f deploy/compose.yml ps    # все сервисы - healthy
curl http://localhost:8080/readyz          # db=true kv=true
```

**Остановка.**

```sh
make down                                              # контейнеры удаляются, тома остаются
docker compose -f deploy/compose.yml --profile full down -v   # вместе с томами
```

Тома проекта: `pgdata` (база), `proctoring-data` (видеофрагменты), `caddy-data`, `prometheus-data`, `loki-data`, `alloy-data`, `grafana-data`; в Docker они называются с префиксом проекта, например `fsp-hack_pgdata`. После `down -v` следующий `make up` заново заполнит демо-данные.

**macOS.** Своего движка Docker там нет: подойдёт Docker Desktop или Colima из `nix develop`. Виртуальной машине Colima по умолчанию достаётся 2 ГБ памяти - для профиля `full` этого мало:

```sh
colima start --cpu 4 --memory 6
make up
```

### 2.2. Демо-аккаунты

При первом запуске на пустой базе API заполняет демо-данные (`SEED_DEMO=true` в `.env` или команда `api seed`, раздел 5.12). Пароль у всех аккаунтов платформы - `demo12345`. На странице входа есть кнопки быстрого входа "Соискатель", "Компания" и "Модератор" (`NEXT_PUBLIC_DEMO_LOGIN=0` при сборке web их скрывает).

| Логин | Роль и назначение |
|:--|:--|
| `candidate@demo.fsp` | Соискатель с категорией, приглашениями и уведомлениями; привязан к участнику реестра `FSP-000011` (раздел 2.6) |
| `cand1@demo.fsp` ... `cand239@demo.fsp` | Банк соискателей: вместе с `candidate@demo.fsp` 240 человек, по 40 на каждую из шести специализаций; у 15 привязан ФСП ID, у большинства тест пройден с видеозаписью. Два номера входят под адресами ниже |
| `minor@demo.fsp` | Соискатель 14-17 лет, ждёт согласия законного представителя: профиль не публикуется, в кабинете баннер |
| `review@demo.fsp` | Соискатель, чей тест ждёт решения модератора |
| `moderator@demo.fsp` | Модератор (роль `admin`), кабинет `/admin`: семь тестов на разбор, две вакансии с отметками антифейка, три жалобы |
| `employer@demo.fsp` | Работодатель "Северный Код": шесть потребностей (по одной на специализацию), подключён к демо-ATS; домен не подтверждён - раздел "Доверие и квота" показывает, как его подтвердить |
| `hr@pixel-manufactura.example` | Работодатель "Пиксельная Мануфактура", домен подтверждён |
| `hr@oblakostroy.example` | Работодатель "ОблакоСтрой", домен подтверждён |
| `hr@medsignal.example` | Работодатель "МедСигнал", домен подтверждён |
| `it-hr@logplus.example` | Работодатель "Логистика Плюс Цифра", домен подтверждён; у него вакансии с отметками антифейка и жалобы из очереди модератора |
| `career@shield-key.example` | Работодатель "Щит и Ключ", домен подтверждён |

**ФСП ID** (профиль `fsp`): участники `fsp-000001` ... `fsp-000010`, пароль `demo12345`, вход - на странице Keycloak после кнопки "Привязать ФСП ID" в настройках соискателя. Это участники реестра `FSP-000001` ... `FSP-000010`, ни к кому не привязанные (у `FSP-000010` нет достижений). Привязать можно новому соискателю или соискателю банка без ФСП ID; `candidate@demo.fsp` уже привязан. Один ФСП ID привязывается только к одному профилю.

Адреса `parent1@demo.fsp` ... `parent6@demo.fsp` - законные представители демо-подростков, а не логины; контактные адреса компаний (`talents@severny-kod.example` и другие) - тоже.

**Служебные доступы.**

| Сервис | Вход |
|:--|:--|
| Keycloak, админка | `admin` / `admin` (`KC_BOOTSTRAP_ADMIN_USERNAME`, `KC_BOOTSTRAP_ADMIN_PASSWORD` в `.env`) |
| Grafana | просмотр без входа; администратор `admin` / `admin` (`GRAFANA_ADMIN_USER`, `GRAFANA_ADMIN_PASSWORD` из shell или `deploy/.env`) |
| Mailpit | локально без пароля; на демо-сервере - `MAILPIT_UI_AUTH` из `deploy/.env` |
| Демо-ATS | без входа |
| Админка реестра ФСП | `Authorization: Bearer` со значением `FSP_REGISTRY_TOKEN` (в `.env.example` - `dev-registry-token`) |

Вход ограничен 10 попытками за 15 минут на пару IP + email (раздел 6).

### 2.3. Без Docker

Нужны Rust >= 1.85, Node.js >= 20.9, PostgreSQL >= 14 с расширением pgvector; Valkey - по желанию. Всё это, включая sqlx-cli, есть в `nix develop`.

```sh
make db                    # база fsp_hack на localhost:5432 (владелец - $USER)
make embedder              # эмбеддер на :8093; веса скачает make model (~490 МБ)
SEED_DEMO=true make api    # API на :8000: миграции при старте, демо-данные на пустой базе
(cd web && npm ci)         # один раз
make web                   # фронтенд на http://localhost:3000, /api/* -> :8000
make test                  # модульные тесты: без базы и сети
```

`make api` не читает `.env`. Он задаёт `DATABASE_URL=postgres://$USER@localhost:5432/fsp_hack`, `PUBLIC_URL=http://localhost:3000`, `FSP_DEV_LINK=true` и `EMBEDDER_URL=http://localhost:8093`; остальное - значения по умолчанию из кода:

- без `VALKEY_URL` лимиты, отозванные токены и поколение кеша живут в памяти процесса (одна реплика);
- без `MAIL_MODE=smtp` письма не отправляются, а печатаются в лог API (`email (MAIL_MODE=log)`) вместе со ссылками;
- без ключей `FIELD_ENC_KEY` и `BIDX_KEY` берутся фиксированные ключи разработки, без `JWT_ED25519_SEED_B64` - случайный ключ на каждый запуск;
- видеофрагменты пишутся в `data/proctoring`;
- без `FSP_OIDC_*` кнопка "Привязать ФСП ID" отвечает 503 `fsp_oidc_disabled`; с `FSP_DEV_LINK=true` ФСП ID привязывается вводом номера через `POST /api/v1/fsp/link/dev` в Swagger (в интерфейсе такого поля нет);
- без `FSP_REGISTRY_URL` достижения не синхронизируются.

Любую переменную можно задать перед командой: `EMBEDDINGS=off make api` - без эмбеддера, `VALKEY_URL=redis://localhost:6379 make api` - с локальным Valkey. Полный список - раздел 3.

Заглушки на локальных портах:

```sh
make fsp-mock    # реестр ФСП на :8090, токен dev-registry-token
ATS_DEMO_API_KEY=fsp_live_demo_ats_key_do_not_use_in_production make ats-mock  # :8082
```

API, подключённый к обеим:

```sh
SEED_DEMO=true FSP_REGISTRY_URL=http://localhost:8090 \
ATS_DEMO_API_KEY=fsp_live_demo_ats_key_do_not_use_in_production \
ATS_DEMO_WEBHOOK_URL=http://localhost:8082/webhooks \
ATS_DEMO_WEBHOOK_SECRET=whsec_demo_ats_secret_do_not_use_in_production \
WEBHOOK_DEV_HOSTS=localhost:8082 make api
```

Чтобы локальная заглушка реестра сама сообщала API о новых результатах, ей нужны `FSP_REGISTRY_WEBHOOK_URL=http://localhost:8000/api/v1/fsp/registry/events` и `FSP_REGISTRY_WEBHOOK_SECRET`, а API - тот же `FSP_REGISTRY_WEBHOOK_SECRET`.

Фронтенд без бэкенда - `NEXT_PUBLIC_USE_MOCKS=1` в `web/.env.local` (`web/README.md`, "Мок-режим").

### 2.4. Демо-сервер

Работающее решение - https://demo.arewehireyet.ru. Это те же образы и `deploy/compose.yml` плюс оверлей `deploy/compose.demo.yml`: Caddy на 80/443 с сертификатом Let's Encrypt (`deploy/Caddyfile.demo`), ФСП ID по пути `/auth` того же домена, демо-почта под паролем по `/mail/`, остальные порты - только на 127.0.0.1. Профили - `fsp` и `ats`; `obs` на сервере не поднимается: оверлей не закрывает порт Grafana 3001, и он открылся бы в обход ufw.

Сервер: Ubuntu LTS, root по ssh, 2 ядра, 4 ГБ памяти, 40 ГБ диска. DNS-запись домена указывает прямо на сервер, без проксирования: Let's Encrypt проверяет домен по HTTP на порту 80.

**1. Подготовка сервера.** Скопируйте `deploy/bootstrap.sh` на сервер и запустите от root. Скрипт идемпотентен: включает ufw (22, 80, 443), создаёт swap 2 ГБ, ставит Docker с Compose из репозиториев Ubuntu, заранее скачивает сторонние образы и создаёт deploy-ключ GitHub.

```sh
scp deploy/bootstrap.sh root@<сервер>:
ssh root@<сервер> bash bootstrap.sh
```

Скрипт выведет публичный ключ - добавьте его в GitHub -> Settings -> Deploy keys репозитория, только чтение.

**2. Код и секреты.**

```sh
git clone git@github.com:K1NGSM4N3/fsp-hack.git /opt/fsp-hack && cd /opt/fsp-hack
python3 deploy/render_env.py --domain demo.arewehireyet.ru
```

`render_env.py` пишет `.env` (права 0600) из шаблона `deploy/.env.template` со свежими ключами, паролями и токенами, и `deploy/.env` с `SITE_URL` и `MAILPIT_UI_AUTH` - их compose подставляет в `${...}`. Существующий `.env` без `--force` не перезаписывается: новые `FIELD_ENC_KEY` и `BIDX_KEY` сделали бы данные нечитаемыми. Сохраните `JWT_ED25519_SEED_B64`, `FIELD_ENC_KEY` и `BIDX_KEY` в менеджер паролей. Для другого домена замените имя сайта ещё и в `deploy/Caddyfile.demo`: там оно записано явно.

**3. Сборка и запуск.**

```sh
docker compose -f deploy/compose.yml -f deploy/compose.demo.yml \
  --profile fsp --profile ats up -d --build
```

**Обновление.** Миграции API применит сам при старте; `SITE_URL` попадает в образ web при сборке, поэтому `--build` обязателен.

```sh
cd /opt/fsp-hack && git pull
docker compose -f deploy/compose.yml -f deploy/compose.demo.yml \
  --profile fsp --profile ats up -d --build
```

На сервере не используйте `make up`, `make down` и `make logs`: они работают без оверлея. Удобно завести псевдоним:

```sh
alias dc='docker compose -f deploy/compose.yml -f deploy/compose.demo.yml'
dc logs -f api
```

**Пути.** Наружу открыт только Caddy, всё на одном домене:

| Путь | Что там |
|:--|:--|
| `/` | Сайт и кабинеты |
| `/api/docs` | Swagger UI |
| `/auth/...` | ФСП ID (Keycloak): вход `fsp-000001` ... `fsp-000010` / `demo12345`; админка Keycloak по этому пути отвечает 404 |
| `/mail/` | Mailpit: логин и пароль - `MAILPIT_UI_AUTH` в `deploy/.env` (пользователь `demo`) |
| `/healthz`, `/readyz` | Проверки API (раздел 5.3) |

Без настоящего SMTP в `.env` письма остаются в Mailpit; настоящий почтовый сервер задают `SMTP_URL` и `MAIL_FROM` (раздел 3), после правки - та же команда `up -d`.

**Закрытые порты.** Keycloak (8081), Mailpit (8025, SMTP 1025), демо-ATS (8082) и админка реестра (8091) слушают только 127.0.0.1: порты, опубликованные Docker, идут в обход ufw. Доступ - через ssh-туннель:

```sh
ssh -L 8081:127.0.0.1:8081 -L 8082:127.0.0.1:8082 -L 8091:127.0.0.1:8091 root@<сервер>
```

Затем http://localhost:8082 - демо-ATS, http://127.0.0.1:8081/auth/admin - админка Keycloak (`admin`, пароль - `KC_BOOTSTRAP_ADMIN_PASSWORD` из `.env`).

**Обслуживание.** Один раз:

```sh
sudo bash deploy/ops/install.sh
systemctl list-timers 'fsp-*'
```

Скрипт ставит два таймера systemd:

- `fsp-backup.timer` - каждую ночь в 03:30 МСК `pg_dump --format=custom` и копия `.env` той же ночи в `/var/backups/fsp-hack` (`db-<время UTC>.dump`, `env-<время UTC>`), хранятся 7 дней (`BACKUP_DIR`, `BACKUP_KEEP_DAYS`). Без `.env` дамп бесполезен: персональные данные в нём зашифрованы `FIELD_ENC_KEY`. Видеофрагменты (том `proctoring-data`) в бэкап не входят. Вручную - `sudo deploy/ops/backup.sh`.
- `fsp-watchdog.timer` - раз в 2 минуты перезапускает контейнеры с проваленным healthcheck: Docker сам перезапускает только упавшие. Журнал - `journalctl -t fsp-watchdog`.

**Восстановление** - с тем же `.env`, что в бэкапе (если ключи менялись, сначала `cp /var/backups/fsp-hack/env-<время> .env`):

```sh
cd /opt/fsp-hack
docker compose -f deploy/compose.yml -f deploy/compose.demo.yml stop api
docker compose -f deploy/compose.yml -f deploy/compose.demo.yml exec -T postgres \
  sh -c 'pg_restore -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists' \
  < /var/backups/fsp-hack/db-<время>.dump
docker compose -f deploy/compose.yml -f deploy/compose.demo.yml \
  --profile fsp --profile ats up -d
```

### 2.5. Эмбеддер без сети

При сборке образ эмбеддера скачивает веса модели с Hugging Face. Если доступа к Hugging Face нет:

1. На машине с сетью скачайте веса: `make model` (или без Rust: `sh crates/embedder/fetch-model.sh multilingual-e5-small models`). В `models/multilingual-e5-small` появятся `config.json`, `tokenizer.json`, `model.safetensors` и `REVISION`; sha256 каждого файла проверяется. Перенесите каталог `models/` в корень репозитория на целевой машине.
2. Смонтируйте его в сервис `embedder` в `deploy/compose.yml`:

   ```yaml
   embedder:
     volumes: ["../models:/models:ro"]
   ```

   Путь внутри контейнера, `EMBEDDER_MODEL_DIR=/models/multilingual-e5-small`, уже задан в образе.
3. Соберите образ без скачивания весов:

   ```sh
   EMBEDDER_FETCH=0 make up
   ```

   На демо-сервере - тот же префикс `EMBEDDER_FETCH=0` перед командой из раздела 2.4.

Остальной сборке (Docker Hub, crates.io, npm) сеть по-прежнему нужна. Можно обойтись и вовсе без модели: `EMBEDDINGS=off` и `--scale embedder=0` (раздел 5.7).

### 2.6. Демо: реестр ФСП сам сообщает о новом результате

Нужен профиль `fsp` и `FSP_REGISTRY_WEBHOOK_SECRET` в `.env` (есть в `.env.example`): его читают и API, и заглушка реестра. `candidate@demo.fsp` привязан к участнику `FSP-000011`. Добавьте участнику результат в заглушке реестра:

```sh
curl -X POST -H 'Authorization: Bearer dev-registry-token' \
  http://127.0.0.1:8091/admin/participants/FSP-000011/achievements
```

Пустое тело - второе место на чемпионате России по алгоритмическому программированию сегодняшним числом; можно передать свою запись в формате реестра. Заглушка отвечает 201: в `added` - новая запись с её `id`, в `push.status` - 202, то есть API принял подписанное событие `achievements.changed`. API сам забирает достижения из реестра, и у соискателя:

- в колокольчике - уведомление "Новые достижения ФСП в профиле";
- в настройках, блок "ФСП ID" (`/candidate/settings#fsp`), - новый результат;
- у навыков "Алгоритмы" и "Структуры данных" - подтверждение ФСП.

Убрать запись (уведомление "Данные ФСП в профиле обновились"):

```sh
curl -X DELETE -H 'Authorization: Bearer dev-registry-token' \
  http://127.0.0.1:8091/admin/participants/FSP-000011/achievements/<id>
```

Правки живут в памяти заглушки: после её перезапуска реестр снова отдаёт `crates/fsp-mock/data/participants.json`. На демо-сервере команду выполняют на самом сервере (или через туннель, раздел 2.4) с токеном из `.env`: `grep FSP_REGISTRY_TOKEN .env`.

## 3. Конфигурация

Все настройки задаются переменными окружения. Корневой `.env` (копия `.env.example`, на демо-сервере - результат `deploy/render_env.py` из `deploy/.env.template`) compose передаёт через `env_file` сервисам api, postgres, keycloak, fsp-mock и ats-mock; блок `environment:` в compose перекрывает его значения, а web, embedder, mailpit и grafana этот файл не получают. Подстановки `${...}` в файлах compose (`SITE_URL`, `API_PROXY_URL`, `KC_HOSTNAME`, `GRAFANA_ADMIN_*`, `MAILPIT_UI_AUTH`, `EMBEDDER_FETCH`) compose берёт из окружения shell или из `deploy/.env`, а не из корневого `.env`; цели Makefile (`make api`, `make web` и другие) `.env` не читают вовсе. Значения в `.env.example` - публичные демо-значения для локального запуска: для любого общего развёртывания помеченные ниже переменные нужно заменить; сводный чек-лист - раздел 4.11.

Ниже "по умолчанию" - что делает код, если переменная не задана или пуста; демо-значение из `.env.example` указано, когда оно отличается. Флаги принимают `1`, `true`, `yes`, `on`; любое другое значение означает "выключено".

### 3.1. API: адреса, cookie, журнал, демо-данные

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `API_BIND` | `0.0.0.0:8000` | Адрес, который слушает API. В compose задан в `environment:` сервиса api, значение из `.env` не действует. |
| `PUBLIC_URL` | `http://localhost:3000` (демо: `http://localhost:8080`) | Адрес сайта для браузера: ссылки в письмах и вебхуках, проверка `Origin` у запросов с cookie, возврат после привязки ФСП ID; его же читает Keycloak при импорте realm. **Обязательно в проде**: `https://домен` без слеша в конце. |
| `COOKIE_SECURE` | `false` | Флаг `Secure` у cookie сессии. **Обязательно в проде**: `true` за HTTPS; по голому HTTP с `true` браузер не сохранит сессию. |
| `RUST_LOG` | `info,api=debug` | Фильтр журнала в синтаксисе `EnvFilter`. В проде достаточно `info`. |
| `LOG_FORMAT` | не задана: текстовый журнал (демо: `json`) | `json` - по строке JSON на событие, так журнал разбирают Alloy и Loki. API сравнивает значение с `json` строго, в нижнем регистре. |
| `SEED_DEMO` | `false` (демо: `true`) | Заполнить демо-компании, кандидатов и входы при старте; срабатывает только на пустой базе. **Обязательно в проде**: `false`, иначе в базе появятся аккаунты с публичным паролем `demo12345`. |
| `SEED_CANDIDATES_PER_SPEC` | `40` | Кандидатов на специализацию в демо-сиде. Только для нагрузочных проверок: `1667` даёт 10 002 кандидата. |
| `GRADE_CHANGE_COOLDOWN_DAYS` | `90` | Сколько дней после присвоения категории нельзя сменить грейд (требование Т-6). |

### 3.2. PostgreSQL и Valkey

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `POSTGRES_USER` | демо: `fsp` | Владелец базы в контейнере postgres; читает образ PostgreSQL и `deploy/ops/backup.sh`. Действует только при первой инициализации тома `pgdata`. |
| `POSTGRES_PASSWORD` | демо: `fsp` | Пароль этого пользователя, тоже только при первой инициализации тома. **Обязательно в проде**; на живом томе пароль меняют через `ALTER USER`, не через `.env`. |
| `POSTGRES_DB` | демо: `fsp_hack` | Имя базы, создаётся при первой инициализации тома. |
| `DATABASE_URL` | не задана: API не стартует | Строка подключения API. Должна совпадать с `POSTGRES_*`; в compose хост - `postgres`, у `make api` своё значение `postgres://$USER@localhost:5432/fsp_hack`. |
| `DATABASE_POOL_SIZE` | `16` | Соединений в пуле одной копии API. Около числа ядер сервера базы; больше пул только переносит очередь в PostgreSQL и растягивает p95 (README, раздел 12.4). |
| `VALKEY_URL` | не задана: лимиты запросов, отозванные токены и поколение кеша кандидатов живут в памяти процесса (демо: `redis://valkey:6379`) | Без Valkey допустима только одна копия API. Для нескольких копий обязателен. |

### 3.3. Ключи и секреты

Каждый ключ - 32 случайных байта в base64 (`openssl rand -base64 32`); строка другой длины останавливает старт API с ошибкой. `deploy/render_env.py` генерирует их сам.

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `JWT_ED25519_SEED_B64` | не задана: новый случайный ключ при каждом старте | Подписывает access-токены (EdDSA). **Обязательно в проде**, одинаковый у всех копий API. После смены выданные access-токены перестают действовать, кабинет сам получает новые по refresh-cookie (раздел 4.10). |
| `FIELD_ENC_KEY` | не заданы обе: фиксированные dev-ключи из кода и предупреждение в журнале | Шифрует персональные данные, секреты вебхуков и видеозапись теста (XChaCha20-Poly1305). **Обязательно в проде**; задаётся вместе с `BIDX_KEY`, одна из пары без другой не даёт API стартовать. |
| `BIDX_KEY` | см. `FIELD_ENC_KEY` | Ключ слепого индекса email (HMAC-SHA256): по нему ищется аккаунт при входе. **Обязательно в проде**. Смена `FIELD_ENC_KEY` или `BIDX_KEY` делает существующие данные нечитаемыми: ключи хранят вместе с бэкапом базы. |

Остальные секреты (`FSP_OIDC_CLIENT_SECRET`, `FSP_REGISTRY_TOKEN`, `FSP_REGISTRY_WEBHOOK_SECRET`, `ATS_DEMO_*`, `KC_BOOTSTRAP_ADMIN_PASSWORD`) описаны в своих разделах ниже.

### 3.4. Почта и уведомления

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `MAIL_MODE` | не задана: `log`, ссылки из писем печатаются в журнал API (демо: `smtp`) | `smtp` - отправка через `SMTP_URL`. Любое значение, кроме `smtp`, означает `log`. |
| `SMTP_URL` | не задана при `MAIL_MODE=smtp`: API не стартует (демо: `smtp://mailpit:1025`) | `smtp://host:port` - без шифрования (Mailpit). **Обязательно в проде**: `smtps://логин:пароль@сервер:465` или `smtp://логин:пароль@сервер:587?tls=required`; `@` в логине пишется как `%40`. |
| `MAIL_FROM` | `ФСП Таланты <noreply@fsp-talent.local>` (демо: `no-reply@fsp-talent.local`) | Отправитель всех писем. **Обязательно в проде**: адрес ящика из `SMTP_URL` или домена с записями SPF и DKIM. |
| `NOTIFY_EMAIL_HOLD_SECS` | `120` | Сколько уведомление ждёт попутчиков в одно письмо. Новое приглашение тоже ждёт. |
| `NOTIFY_EMAIL_GAP_SECS` | `600` | Не больше одного письма получателю за этот интервал; новое приглашение его пропускает. |
| `NOTIFY_EMAIL_DAILY_CAP` | `10` | Не больше стольких писем получателю за сутки, приглашения тоже считаются. |
| `NOTIFY_INTERVAL_MS` | `5000` | Как часто рассыльщик писем ищет работу. |

Black-box стек (`deploy/compose.blackbox.yml`) ставит `NOTIFY_EMAIL_HOLD_SECS` и `NOTIFY_EMAIL_GAP_SECS` в 3 с и `NOTIFY_INTERVAL_MS` в 1000, чтобы тест дождался письма; в проде значения по умолчанию и есть рабочее правило.

### 3.5. ФСП ID (Keycloak)

Вход через ФСП ID включается, только когда заданы и `FSP_OIDC_ISSUER`, и `FSP_OIDC_CLIENT_SECRET`; иначе привязка отвечает 503 `fsp_oidc_disabled`, остальное приложение работает.

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `FSP_OIDC_ISSUER` | не задана: ФСП ID выключен (демо: `http://localhost:8081/realms/fsp`) | Адрес realm для браузера и `iss` каждого токена. **Обязательно в проде**: публичный адрес, на демо-сервере `https://домен/auth/realms/fsp`. Должен совпадать с `KC_HOSTNAME` плюс `/realms/fsp`. |
| `FSP_OIDC_DISCOVERY_URL` | `FSP_OIDC_ISSUER` + `/.well-known/openid-configuration` (демо: `http://keycloak:8080/realms/fsp/...`) | Адрес discovery изнутри сети compose; Keycloak отдаёт по нему адреса token, userinfo и jwks на внутреннем хосте, а `issuer` остаётся публичным. |
| `FSP_OIDC_CLIENT_ID` | `fsp-talent` | Клиент в realm `fsp`; менять вместе с `deploy/keycloak/fsp-realm.json`. |
| `FSP_OIDC_CLIENT_SECRET` | не задана: ФСП ID выключен | Секрет клиента; Keycloak читает то же значение при импорте realm. **Обязательно в проде**. |
| `FSP_OIDC_REDIRECT_URL` | `PUBLIC_URL` + `/api/v1/fsp/link/callback` | Адрес возврата после входа в ФСП ID; Keycloak берёт его в список разрешённых при импорте realm. **Обязательно в проде**: публичный домен; если не задавать, адрес строится из `PUBLIC_URL`. |
| `FSP_DEV_LINK` | `false` (у `make api`: `true`) | `true` - привязка введённого `fsp_id` без Keycloak. Только для локальной разработки; **обязательно в проде** `false`. |

### 3.6. Реестр ФСП

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `FSP_REGISTRY_URL` | не задана: привязка работает, достижения не синхронизируются (демо: `http://fsp-mock:8090`) | Адрес реестра достижений ФСП. В проде - адрес настоящего реестра. |
| `FSP_REGISTRY_TOKEN` | `dev-registry-token` | Bearer-токен запросов API к реестру; fsp-mock ждёт его же и на своих админ-методах. **Обязательно в проде**. |
| `FSP_REGISTRY_WEBHOOK_SECRET` | не задана: `POST /api/v1/fsp/registry/events` отвечает 503, работает только опрос реестра | Общий секрет подписи событий реестра (`FSP-Signature`, HMAC-SHA256); fsp-mock подписывает тем же значением. **Обязательно в проде**. |

### 3.7. ATS и вебхуки

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `WEBHOOK_DEV_HOSTS` | пусто: вебхуки только по HTTPS на публичные адреса (демо: `ats-mock:8082`) | Хосты через запятую, куда вебхук может идти по HTTP и на частный адрес. **Обязательно в проде**: пусто, если демо-ATS не нужна. |
| `WEBHOOK_DISPATCH_INTERVAL_MS` | `2000` | Период опроса очереди исходящих вебхуков. |
| `ATS_POLL_LAG_SECS` | `5` | `GET /integrations/invitations` отдаёт только изменения старше этого срока, чтобы курсор не перепрыгнул ещё не закоммиченную запись. |
| `ATS_DEMO_API_KEY` | не задана: демо-интеграция не создаётся | При старте API выдаёт компании `ATS_DEMO_OWNER_EMAIL` этот API-ключ; ключ должен начинаться с `fsp_live_`, иначе пропускается. **Обязательно в проде**: пусто или свой ключ. |
| `ATS_DEMO_OWNER_EMAIL` | `employer@demo.fsp` | Владелец компании, которую подключают к демо-ATS. Компании нет - шаг пропускается с предупреждением. |
| `ATS_DEMO_WEBHOOK_URL` | не задана: ключ выдаётся, вебхук нет (демо: `http://ats-mock:8082/webhooks`) | Адрес вебхука демо-компании; его хост должен быть в `WEBHOOK_DEV_HOSTS`, иначе вебхук отклоняется. Вебхук создаётся, только если у компании его ещё нет. |
| `ATS_DEMO_WEBHOOK_SECRET` | не задана: вебхук не создаётся | Секрет подписи вебхуков демо-компании; ats-mock проверяет подпись тем же значением. **Обязательно в проде**, если демо-ATS оставлена. |

Без профиля `ats` доставки демо-компании падают и видны в журнале доставок кабинета.

### 3.8. Эмбеддинги

Со стороны API:

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `EMBEDDINGS` | `on` | Выключатель эмбеддингов. Выключено (`off`) или нет `EMBEDDER_URL`: близость описаний по ключевым словам, похожие вакансии по навыкам, пересказы чужих вакансий не ищутся. |
| `EMBEDDER_URL` | не задана: эмбеддинги выключены | Адрес сервиса embedder. Compose задаёт `http://embedder:8093` в `environment:` сервиса api, `make api` - `http://localhost:8093`; в `.env` не нужна. |
| `EMBEDDER_TIMEOUT_MS` | `15000` | Таймаут одного запроса API к эмбеддеру. |
| `EMBEDDINGS_INTERVAL_MS` | `1000` | Как часто простаивающий фоновый воркер смотрит в очередь текстов. |

Сервис embedder не получает `.env`: его переменные задаются в `deploy/compose.yml` (или в своём файле-оверлее) и в образе.

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `EMBEDDER_BIND` | `0.0.0.0:8093` (у `make embedder`: `127.0.0.1:8093`) | Адрес, который слушает эмбеддер. |
| `EMBEDDER_MODEL` | `multilingual-e5-small` | Имя модели. Сервис знает ещё `rubert-tiny2`, но API откалиброван и схема базы (`vector(384)`) рассчитана только на `multilingual-e5-small`. |
| `EMBEDDER_MODEL_DIR` | `models/<модель>` (в образе: `/models/multilingual-e5-small`) | Каталог с весами. Менять, когда веса смонтированы снаружи (сборка с `EMBEDDER_FETCH=0`, раздел 3.14). |
| `EMBEDDER_CONCURRENCY` | `1` (compose: `1`) | Сколько проходов модели идёт одновременно. Больше - только больше памяти: проход и так занимает все потоки. |
| `EMBEDDER_QUEUE_WAIT_MS` | `10000` | Запрос, ждущий очереди дольше, получает 503. |
| `RAYON_NUM_THREADS` | все ядра (compose: `2`) | Потоков на проход. На одноядерном VPS - `1`. |

### 3.9. Видеозапись теста

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `PROCTORING` | `optional` | `optional` - кандидат сам решает, писать ли видео; `off` - шага видеозаписи нет. Другое значение останавливает старт API. Камера в браузере работает только по HTTPS или на localhost. |
| `PROCTORING_DIR` | `data/proctoring` относительно рабочего каталога (compose: `/var/lib/fsp/proctoring`, том `proctoring-data`) | Каталог зашифрованных фрагментов видео; в базе только метаданные. В compose задан в `environment:`, значение из `.env` не действует. |
| `PROCTORING_RETENTION_DAYS` | `90` | Сколько дней хранится видео; после срока файлы удаляются, запись помечается удалённой. |

### 3.10. Лимиты, кеши, производительность

Отдельных переменных для лимитов запросов (например, 10 попыток входа за 15 минут), сроков кешей и снимков пула кандидатов нет: они заданы в коде; лимиты перечислены в `docs/api.md`, "Лимиты запросов", кеши - в README, раздел 12.4. На производительность влияют `DATABASE_POOL_SIZE` (раздел 3.2), `VALKEY_URL` (нужен для нескольких копий API), `EMBEDDINGS` и потоки эмбеддера (раздел 3.8), а также `mem_limit` сервисов в `deploy/compose.yml`.

### 3.11. Фронтенд

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `SITE_URL` | `http://localhost:8080` | Публичный адрес для canonical-ссылок, sitemap, OG-картинок и `llms.txt`. Compose берёт его из shell или `deploy/.env` и встраивает в образ web при сборке: после смены пересобрать web. **Обязательно в проде**: `https://домен`. |
| `API_PROXY_URL` | `http://localhost:8000` (в образе web: `http://api:8000`) | Куда сервер Next.js ходит за API: серверный рендер публичных вакансий, тексты согласий на `/privacy` и rewrite `/api/*`. В compose `/api/*` маршрутизирует Caddy, rewrite запасной. Для `make web` задаётся в `web/.env.development`; корневой `.env` её никому не передаёт. |
| `NEXT_PUBLIC_USE_MOCKS` | `0` | `1` - данные из `web/src/mocks` в браузере вместо API, для вёрстки без бэкенда. Встраивается при сборке; образ web собирается с `0`. |
| `NEXT_PUBLIC_DEMO_LOGIN` | не задана: кнопки демо-входа на `/login` показаны | `0` - скрыть кнопки демо-аккаунтов. Встраивается при сборке; `web.Dockerfile` её не передаёт, поэтому образ всегда собирается с кнопками. |
| `PORT`, `HOSTNAME` | `3000`, `0.0.0.0` | Адрес сервера Next.js в образе web; `llms-full.txt` собирается запросом к самому себе на `PORT`. |

Локально переменные web читаются из `web/.env.development` (в git) и `web/.env.local` (шаблон - `web/.env.example`).

### 3.12. Заглушки: реестр ФСП и демо-ATS

Обе заглушки получают корневой `.env`, поэтому общие с API значения (`FSP_REGISTRY_*`, `ATS_DEMO_*`) совпадают без ручных шагов. `LOG_FORMAT=json` и `RUST_LOG` (по умолчанию `info`) действуют так же, как у API.

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `FSP_MOCK_BIND` | `0.0.0.0:8090` | Адрес fsp-mock. В compose задан в `environment:`; снаружи админка доступна на `127.0.0.1:8091`. |
| `FSP_REGISTRY_TOKEN` | `dev-registry-token` | Токен, который fsp-mock ждёт от API и на админ-методах; с умолчанием пишет предупреждение. |
| `FSP_REGISTRY_WEBHOOK_URL` | не задана: правки из админки в API не отправляются (compose: `http://api:8000/api/v1/fsp/registry/events`) | Куда fsp-mock отправляет события `achievements.changed`. `make fsp-mock` её не задаёт. |
| `FSP_REGISTRY_WEBHOOK_SECRET` | не задана: события не отправляются | Секрет подписи событий; должен совпадать с API. |
| `ATS_MOCK_BIND` | `0.0.0.0:8082` (у `make ats-mock`: `127.0.0.1:8082`) | Адрес демо-ATS. |
| `ATS_API_URL` | `http://localhost:8000` (compose: `http://api:8000`) | Куда демо-ATS отправляет этапы найма. |
| `ATS_DEMO_API_KEY` | не задана: кнопки этапов найма выключены | Ключ, с которым демо-ATS ходит в API; тот же, что выдаёт API. |
| `ATS_DEMO_WEBHOOK_SECRET` | не задана: ats-mock не стартует | Секрет проверки подписи входящих вебхуков. У `make ats-mock` подставлено демо-значение. |

### 3.13. Keycloak, Grafana, Mailpit

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `KC_BOOTSTRAP_ADMIN_USERNAME` | демо: `admin` | Администратор консоли Keycloak (`/admin`). Читается из `.env`. |
| `KC_BOOTSTRAP_ADMIN_PASSWORD` | демо: `admin` | Его пароль. **Обязательно в проде**. Keycloak работает в режиме `start-dev` без тома данных: админ и realm создаются заново при каждом пересоздании контейнера. |
| `KC_HOSTNAME` | `http://localhost:8081` (оверлей демо-сервера: `SITE_URL` + `/auth`) | Публичный адрес Keycloak, из него строится `issuer`. Подстановка compose из shell или `deploy/.env`. **Обязательно в проде** вне оверлея демо-сервера. |
| `GRAFANA_ADMIN_USER` | `admin` | Администратор Grafana (профиль `obs`); подстановка compose. Анонимный просмотр дашбордов включён всегда. |
| `GRAFANA_ADMIN_PASSWORD` | `admin` | Его пароль; Grafana применяет его только при первом создании тома `grafana-data`. **Обязательно в проде**, если порт 3001 доступен извне. |
| `MAILPIT_UI_AUTH` | не задана: оверлей демо-сервера не запускается | `логин:пароль` для демо-почты по `/mail/`; только `deploy/compose.demo.yml`, пишет `render_env.py` в `deploy/.env`. В базовом стеке Mailpit на 8025 без пароля. |

Остальное в этих сервисах зафиксировано в compose: `KC_HOSTNAME_BACKCHANNEL_DYNAMIC`, `KC_HTTP_*`, `KC_PROXY_HEADERS`, `MP_MAX_MESSAGES=500`, `MP_SMTP_AUTH_*`, `GF_*`, `GOMEMLIMIT` у Caddy и Loki. Менять их - правкой `deploy/compose.yml` или своего оверлея.

### 3.14. Сборка образов

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `EMBEDDER_FETCH` | `1` | Аргумент сборки образа эмбеддера: `1` - скачать веса с Hugging Face по закреплённой ревизии и проверить sha256. `0` - собрать без сети, веса монтируются в `EMBEDDER_MODEL_DIR` (`EMBEDDER_FETCH=0 make up`). Подстановка compose. |
| `EMBEDDER_MODEL` | `multilingual-e5-small` | Аргумент сборки: какие веса класть в образ. Compose его не передаёт. |
| `SITE_URL` | `http://localhost:8080` | Аргумент сборки web (раздел 3.11). |
| `API_PROXY_URL` | `http://api:8000` | Аргумент сборки web: адрес rewrite `/api/*`, вычисляется при `next build`. Compose его не передаёт, поэтому в образе всегда `http://api:8000`; значение из shell меняет только серверные запросы web во время работы. |

### 3.15. Makefile и обслуживание

| Переменная | По умолчанию | Что делает и когда менять |
|:--|:--|:--|
| `DATABASE_URL` (make) | `postgres://$USER@localhost:5432/fsp_hack` | База для `make api` и `make db-prepare`. |
| `PG_ADMIN_URL` | `postgres://postgres@localhost:5432` | Под кем `make db` создаёт базу. |
| `MODEL` | `multilingual-e5-small` | Модель для `make model`, `make embedder`, `make validate`. |
| `BACKUP_DIR` | `/var/backups/fsp-hack` | Куда `deploy/ops/backup.sh` пишет дамп и `.env` той же ночи. Таймер systemd переменных не задаёт: менять через `Environment=` в `fsp-backup.service`. |
| `BACKUP_KEEP_DAYS` | `7` | Сколько дней хранить дампы. |

## 4. От демо к боевым системам

Демо-стенд поднимает вместо внешних систем заглушки. В этом разделе описано, как заменить каждую из них настоящей системой, что поменять в `.env` и compose и что без доработки кода не заработает. Все переменные окружения с назначением и значениями по умолчанию перечислены в разделе 3, здесь указано только то, что меняется при переходе.

### 4.1. Что в демо заменено заглушками

| Компонент | В демо | В проде | Профиль compose |
|:--|:--|:--|:--|
| ФСП ID | Keycloak, realm `fsp`, клиент `fsp-talent`, 10 тестовых участников `fsp-000001` ... `fsp-000010` | OIDC-провайдер ФСП ID напрямую или через наш Keycloak как брокер | `fsp` |
| Реестр ФСП | `fsp-mock`: участники из `crates/fsp-mock/data/participants.json`, правки через админку живут в памяти | API реестра ФСП по тому же контракту | `fsp` |
| Почта | Mailpit: принимает все письма, веб-ящик на :8025 | SMTP-сервис на домене платформы | ядро (без профиля) |
| ATS работодателя | `ats-mock`, подключён к `employer@demo.fsp` ключом и вебхуком из `.env` | ATS каждой компании; компания подключает её сама в кабинете | `ats` |
| Эмбеддинги | `embedder`, веса скачиваются с Hugging Face при сборке | тот же сервис; веса можно положить без доступа в интернет | ядро |
| Наблюдаемость | Prometheus, Loki, Alloy, Grafana с анонимным просмотром на :3001 | тот же профиль за туннелем или свой стек мониторинга | `obs` |
| HTTPS | Caddy на :8080 без TLS | Caddy на 80 и 443 с сертификатом Let's Encrypt | ядро |
| Данные | демо-сид: 6 компаний, 240 кандидатов, демо-логины с паролем `demo12345`, модератор | пустая база, `SEED_DEMO=false` | нет, переменная `.env` |

Профиль `full` включает `fsp`, `ats` и `obs` сразу, поэтому в проде его не используют.

### 4.2. Домен, TLS и состав compose

Боевой стек - это `deploy/compose.yml` плюс свой оверлей. Оверлей демо-сервера (`deploy/compose.demo.yml`, `deploy/Caddyfile.demo`, раздел 2.4) уже открывает наружу только Caddy на 80 и 443, но оставляет Mailpit под `/mail/` и Keycloak под `/auth`. Для прода удобнее отдельный оверлей.

1. DNS: запись A (и AAAA, если есть IPv6) домена на сервер. Входящие 80 и 443 открыты: на 80 Caddy проходит проверку Let's Encrypt (http-01) и перенаправляет на HTTPS.
2. Caddyfile: скопировать демо-вариант и поставить свой домен.

   ```sh
   cp deploy/Caddyfile.demo deploy/Caddyfile.prod
   sed -i 's/^demo\.arewehireyet\.ru {/example.ru {/' deploy/Caddyfile.prod
   ```

   В `deploy/Caddyfile.prod` удалить блок `handle /mail*` (Mailpit), а при подключении ФСП ID напрямую (раздел 4.3) - и блоки `@kc_admin` и `handle /auth/*`; тогда исключения `/auth` и `/mail` из матчера `@app` тоже не нужны. Заголовок `Strict-Transport-Security` в демо-варианте уже есть.

3. Оверлей `deploy/compose.prod.yml`: Caddy на 80 и 443 со своим Caddyfile, API без зависимости от Mailpit, заглушки и Mailpit выведены в профиль `demo`, который в проде не включают, Grafana только на 127.0.0.1.

   ```yaml
   services:
     caddy:
       ports: !override
         - "80:80"
         - "443:443"
       volumes: !override
         - ./Caddyfile.prod:/etc/caddy/Caddyfile:ro
         - caddy-data:/data
     api:
       depends_on: !override
         postgres:
           condition: service_healthy
         valkey:
           condition: service_healthy
     mailpit:
       profiles: !override [demo]
     fsp-mock:
       profiles: !override [demo]
     ats-mock:
       profiles: !override [demo]
     grafana:
       ports: !override
         - "127.0.0.1:3001:3000"
   ```

4. Адреса. В `.env`: `PUBLIC_URL=https://example.ru` (без слеша в конце) и `COOKIE_SECURE=true`. `PUBLIC_URL` - это и проверка `Origin`: если он не совпадает с адресом в браузере, все изменяющие запросы кабинетов получают 403 `csrf`. В `deploy/.env` (его compose читает для подстановки `${...}`, корневой `.env` - нет): `SITE_URL=https://example.ru`. `SITE_URL` запекается в образ web при сборке, после смены нужен `--build`.
5. Запуск:

   ```sh
   docker compose -f deploy/compose.yml -f deploy/compose.prod.yml up -d --build
   curl -fsS https://example.ru/readyz
   ```

   Без профилей поднимается ядро: Caddy, web, API, эмбеддер, PostgreSQL, Valkey. Наблюдаемость добавляется `--profile obs`, Keycloak в режиме брокера - `--profile fsp` (заглушка реестра при этом не стартует: она в профиле `demo`).

Сертификаты Caddy хранит в томе `caddy-data`; том не удаляют, иначе при каждом пересоздании Caddy будет заново запрашивать сертификат и упрётся в лимиты Let's Encrypt.

Если перед Caddy стоит балансировщик или CDN, Caddy нужно указать его адреса в глобальной опции `servers { trusted_proxies static ... }`. Иначе Caddy записывает в `X-Forwarded-For` адрес балансировщика, лимиты запросов по IP (вход, регистрация, события реестра) станут общими для всех посетителей, а в отметки согласий попадёт его адрес вместо адреса пользователя. Порт API (8000) наружу не публикуют: API берёт IP клиента из первого значения `X-Forwarded-For` и верит ему, потому что перед ним всегда Caddy.

Видеозапись теста работает только по HTTPS: браузер не даёт камеру странице на http (кроме localhost). Если прокторинг не нужен - `PROCTORING=off`.

### 4.3. ФСП ID

API проходит Authorization Code с PKCE S256 и берёт ФСП ID из claim `fsp_id` в `id_token`. Есть два способа подключить настоящий ФСП ID.

**Напрямую к провайдеру ФСП ID.** Keycloak из compose не нужен.

| Переменная | Значение |
|:--|:--|
| `FSP_OIDC_ISSUER` | issuer провайдера, ровно как в поле `iss` его токенов |
| `FSP_OIDC_DISCOVERY_URL` | оставить пустой: тогда это `FSP_OIDC_ISSUER` + `/.well-known/openid-configuration`. Задают, только если discovery доступен API по другому адресу |
| `FSP_OIDC_CLIENT_ID` | выдаёт провайдер |
| `FSP_OIDC_CLIENT_SECRET` | выдаёт провайдер; без него вход через ФСП ID выключен (503 `fsp_oidc_disabled`) |
| `FSP_OIDC_REDIRECT_URL` | `https://example.ru/api/v1/fsp/link/callback`; пустое значение даёт тот же адрес от `PUBLIC_URL` |
| `FSP_DEV_LINK` | `false` |

У провайдера регистрируется клиент с redirect URI `https://example.ru/api/v1/fsp/link/callback` (точное совпадение). Что API ожидает от провайдера:

- discovery с `authorization_endpoint`, `token_endpoint`, `jwks_uri`;
- код обменивается на токены с аутентификацией клиента `client_secret_basic` (логин и секрет в заголовке `Authorization: Basic`) и параметром `code_verifier`;
- в ответе есть `id_token`, подписанный RS256 ключом из JWKS (по `kid`, без `kid` берётся первый ключ);
- в `id_token`: `iss` равен `FSP_OIDC_ISSUER`, `aud` содержит `FSP_OIDC_CLIENT_ID`, `nonce` из запроса и строковый claim `fsp_id`;
- API запрашивает только scope `openid`, поэтому `fsp_id` должен попадать в `id_token` при этом scope.

Без `fsp_id` привязка заканчивается на `/candidate/settings?fsp=error&reason=no_fsp_id`, при любой другой ошибке проверки - `reason=verification_failed` (подробность в логе API, сообщение `fsp id_token exchange failed`). После настройки профиль `fsp` не включают, а из `.env` убирают `KC_BOOTSTRAP_ADMIN_*`.

**Через наш Keycloak как брокер.** Подходит, если провайдер ФСП ID не выполняет требования выше: отдаёт ФСП ID под другим именем claim, подписывает не RS256, требует другой scope или другой способ аутентификации клиента. Keycloak переводит его токены в формат, который ждёт API, и код менять не нужно.

1. Keycloak запускается профилем `fsp` по образцу `deploy/compose.demo.yml`: блок `keycloak` (`KC_HOSTNAME=${SITE_URL}/auth`, `KC_HTTP_RELATIVE_PATH=/auth`, `KC_PROXY_HEADERS=xforwarded`, порт только на 127.0.0.1) переносится в `deploy/compose.prod.yml`, блоки `/auth` в Caddyfile остаются.
2. В compose Keycloak работает в режиме `start-dev`, и его данные живут внутри контейнера: настройки из админки пропадают при пересоздании. Для прода - команда `start --import-realm` и отдельная база (`KC_DB=postgres`, `KC_DB_URL`, `KC_DB_USERNAME`, `KC_DB_PASSWORD`), либо все настройки realm держат в `deploy/keycloak/fsp-realm.json`.
3. Из `deploy/keycloak/fsp-realm.json` удалить массив `users`: там 10 тестовых участников с паролем `demo12345`.
4. В realm `fsp` добавить Identity Provider типа OpenID Connect с данными, выданными ФСП ID. У провайдера ФСП ID регистрируется redirect URI Keycloak: `https://example.ru/auth/realms/fsp/broker/<alias>/endpoint`, где `<alias>` - имя провайдера в Keycloak.
5. У Identity Provider добавить mapper "Attribute Importer": claim с ФСП ID у провайдера -> атрибут пользователя `fsp_id`, режим синхронизации `force`. Существующий mapper клиента `fsp-talent` уже кладёт этот атрибут в `id_token`. Атрибут проверяется шаблоном `^FSP-[0-9]{6}$` в профиле пользователя realm; если у настоящих ФСП ID другой формат, шаблон меняют.
6. Чтобы участник не видел форму входа Keycloak, в Authentication -> flow `browser` у шага "Identity Provider Redirector" указать этого провайдера по умолчанию.

`FSP_OIDC_*` в этом режиме остаются как в `deploy/.env.template`: issuer `https://example.ru/auth/realms/fsp`, discovery по внутреннему адресу `http://keycloak:8080/auth/realms/fsp/.well-known/openid-configuration`.

### 4.4. Реестр ФСП

```sh
FSP_REGISTRY_URL=https://registry.example.ru
FSP_REGISTRY_TOKEN=<токен, выданный реестром>
FSP_REGISTRY_WEBHOOK_SECRET=<общий секрет событий>
```

Без `FSP_REGISTRY_URL` привязка ФСП ID работает, но достижения не загружаются (`POST /fsp/sync` - 503 `fsp_registry_disabled`). Заглушка `fsp-mock` в проде не запускается.

**Чтение (pull).** Контракт, который реализует `fsp-mock` и который должен выполнять настоящий реестр:

| Что | Как |
|:--|:--|
| Запрос | `GET {FSP_REGISTRY_URL}/participants/{fsp_id}/achievements` |
| Авторизация | `Authorization: Bearer {FSP_REGISTRY_TOKEN}`, токен статический |
| Таймаут | 10 с |
| 200 | `{"fsp_id": "...", "achievements": [...]}`; API читает только `achievements` |
| 404 | участника нет в реестре: у кандидата пустой список достижений |
| Другой код или нет ответа | кандидату 502 `fsp_unavailable`, привязка и прежний снимок сохраняются |

Когда API читает реестр: при привязке ФСП ID, по кнопке "Обновить из реестра" (`POST /fsp/sync`) и по событию реестра. Фоновой периодической сверки нет. Карточку участника (`GET /participants/{fsp_id}`), которую тоже отдаёт заглушка, API не запрашивает.

Запись о достижении (схема `Achievement` в OpenAPI); поля, которые к типу записи не относятся, передаются как `null`:

| Поле | Значения |
|:--|:--|
| `id` | постоянный id записи в реестре; по нему API понимает, что добавилось и что исчезло |
| `kind` | `rank` (разряд) или `competition` (соревнование) |
| `discipline` | `algorithmic`, `product`, `security`, `ai`, `robotics`, `uav` |
| `rank` | для `rank`: `youth`, `third`, `second`, `first`, `cms`, `ms` |
| `event_name`, `event_level`, `place` | для `competition`; уровень `regional`, `interregional`, `national`, `international`; место `first`, `second`, `third`, `finalist`, `participant` |
| `team`, `role` | команда и роль в ней, необязательны |
| `date` | `YYYY-MM-DD` |
| `protocol_url` | ссылка на протокол, необязательна |

```json
{
  "id": "ACH-000001-02",
  "kind": "competition",
  "discipline": "algorithmic",
  "rank": null,
  "event_name": "Чемпионат Москвы по спортивному программированию",
  "event_level": "regional",
  "place": "first",
  "team": null,
  "role": null,
  "date": "2022-03-20",
  "protocol_url": "https://registry.example.ru/protocols/2022/reg-algorithmic.pdf"
}
```

Значения перечислений - часть контракта: если хотя бы одна запись в ответе содержит значение вне списка (например, новую дисциплину), API не разберёт весь список, и у кандидата не будет ни ФСП-скора, ни навыков, подтверждённых ФСП (раздел 4.12).

**События (push).** Реестру передают:

- адрес: `https://example.ru/api/v1/fsp/registry/events`;
- секрет: значение `FSP_REGISTRY_WEBHOOK_SECRET` (сгенерировать: `openssl rand -hex 32`);
- формат: `POST` с телом `{"event_id": "...", "type": "achievements.changed", "fsp_id": "...", "occurred_at": "<RFC 3339>"}` и заголовком `FSP-Signature: t=<unix>,v1=<hex>`, где `v1 = hex(HMAC-SHA256(секрет, t + "." + тело))` - та же схема, что у исходящих вебхуков (`docs/api.md`, "Вебхуки (исходящие)");
- правила: часы реестра расходятся с часами сервера не больше чем на 5 минут (иначе 401 `signature_expired`), тело до 16 КиБ, не больше 300 запросов в минуту с одного адреса, `event_id` до 128 видимых ASCII-символов и одинаковый при повторной доставке;
- ответы: 202 - принято (в том числе повтор и незнакомый `fsp_id`); 502 `fsp_unavailable` - API не смог прочитать реестр, событие нужно доставить ещё раз; 5xx и 429 - тоже повторить.

Проверка подписи с сервера после настройки (незнакомый `fsp_id` даёт 202 и ничего не меняет):

```sh
secret='<FSP_REGISTRY_WEBHOOK_SECRET>'
body='{"event_id":"evt_smoke_1","type":"achievements.changed",'
body="$body"'"fsp_id":"FSP-999999","occurred_at":"2026-10-10T09:00:00Z"}'
t=$(date +%s)
sig=$(printf '%s.%s' "$t" "$body" | openssl dgst -sha256 -hmac "$secret" | awk '{print $NF}')
curl -i -X POST https://example.ru/api/v1/fsp/registry/events \
  -H 'Content-Type: application/json' -H "FSP-Signature: t=$t,v1=$sig" --data "$body"
```

Если реестр не умеет отправлять события, `FSP_REGISTRY_WEBHOOK_SECRET` оставляют пустым: адрес событий отвечает 503 `fsp_webhook_disabled`, достижения обновляются только при привязке и по кнопке.

### 4.5. Почта

```sh
MAIL_MODE=smtp
SMTP_URL=smtps://no-reply%40example.ru:<пароль>@smtp.example.ru:465
MAIL_FROM=ФСП Таланты <no-reply@example.ru>
```

- `SMTP_URL`: `smtps://...:465` - TLS с первого байта, `smtp://...:587?tls=required` - STARTTLS; `@` в логине записывается как `%40`. Простой `smtp://host:port` без TLS годится только для Mailpit.
- `MAIL_MODE` с любым значением, кроме `smtp`, не отправляет письма, а печатает ссылки из них в лог API. В проде так оставлять нельзя: без писем никто не подтвердит email.
- `MAIL_FROM` - ящик, от имени которого почтовый сервис разрешает отправку (обычно тот же, что в логине). Домен отправителя в DNS: SPF с почтовым сервисом, DKIM-ключ, который выдал сервис, и DMARC. Без них письма уходят в спам или отклоняются.
- Письма-уведомления несут `List-Unsubscribe` и `List-Unsubscribe-Post` (отписка в один клик по RFC 8058) со ссылкой от `PUBLIC_URL`, поэтому `PUBLIC_URL` должен быть публичным адресом на HTTPS.
- Частота писем (`NOTIFY_EMAIL_*`) по умолчанию уже боевая, раздел 3.
- Mailpit в проде не нужен: в оверлее из раздела 4.2 он выведен в профиль `demo`.

Проверка: зарегистрировать аккаунт на свой ящик и получить письмо подтверждения. Ошибки отправки пишутся в лог API.

### 4.6. ATS работодателя

Заменять на нашей стороне нечего: `ats-mock` изображает ATS одной демо-компании, а в проде каждая компания подключает свою ATS сама. Что сделать на сервере:

- не включать профиль `ats` (в оверлее из раздела 4.2 `ats-mock` в профиле `demo`);
- `WEBHOOK_DEV_HOSTS` пустой: вебхуки уходят только по `https://` и только на публичные адреса;
- удалить `ATS_DEMO_API_KEY`, `ATS_DEMO_WEBHOOK_URL`, `ATS_DEMO_WEBHOOK_SECRET`, `ATS_DEMO_OWNER_EMAIL`. Если `ATS_DEMO_API_KEY` задан, API при каждом старте выдаёт этот ключ со всеми скоупами компании, владелец которой входит как `ATS_DEMO_OWNER_EMAIL` (по умолчанию `employer@demo.fsp`), а значение из `.env.example` известно всем;
- API должен иметь исходящий доступ в интернет на 443 - туда уходят вебхуки.

Что настраивает компания в кабинете "Интеграции" (`/employer/integrations`, нужен заполненный профиль компании):

| Что | Как |
|:--|:--|
| API-ключ | до 10 действующих; скоупы `invitations:read`, `candidates:read`, `outcomes:write`, `needs:write`; ключ `fsp_live_...` показывается один раз; 120 запросов в минуту на ключ; работает только в `/api/v1/integrations` |
| Вебхук | один адрес `https://` на компанию; секрет подписи показывается один раз при первом сохранении; смена секрета - сутки события подписаны новым и прежним |
| Проверка | кнопка "Тестовое событие" и журнал доставок с кодом ответа каждой попытки |

Требования к адресу вебхука: `https://`, без логина и пароля в URL, имя находится в DNS и указывает только на публичные адреса (частные сети, loopback, link-local и служебные диапазоны отклоняются при сохранении и при каждом соединении), редиректы не выполняются. Что должна делать ATS: проверять `FSP-Signature` и отклонять метку старше 5 минут, отбрасывать повторы по `FSP-Event-Id` и отвечать 2xx за 10 с. Неответивший адрес получает повторы в течение суток. ATS без входящих соединений из интернета опрашивает `GET /api/v1/integrations/invitations` с курсором. Каталог событий и формат - `docs/api.md`, "Вебхуки (исходящие)", и README, раздел 10.2.

### 4.7. Эмбеддинги

Эмбеддер - локальный сервис, тексты не покидают сервер. Для прода менять ничего не нужно; варианты:

- **Без доступа к Hugging Face.** На машине с сетью - `make model` (каталог `models/multilingual-e5-small`, sha256 проверяется); каталог перенести на сервер в корень репозитория и добавить в `services` файла `deploy/compose.prod.yml`:

  ```yaml
    embedder:
      build:
        args:
          EMBEDDER_FETCH: "0"
      volumes:
        - ../models:/models:ro
  ```

  Подробно - раздел 2.5.
- **Память и процессор.** До ~250 МБ под нагрузкой при лимите контейнера 384 МБ, два потока (`RAYON_NUM_THREADS` в `deploy/compose.yml`); на одноядерном сервере - `RAYON_NUM_THREADS=1`.
- **Выключить.** `EMBEDDINGS=off` в `.env` и запуск с `--scale embedder=0`: близость описаний считается по ключевым словам, похожие вакансии - по навыкам, пересказы чужих вакансий не ищутся.
- После первого старта на пустой базе векторы считаются фоновым воркером по мере появления текстов; остановка эмбеддера ничего не ломает, очередь досчитается после его возвращения.

### 4.8. Наблюдаемость

Профиль `obs`: Prometheus собирает `/metrics` API и эмбеддера внутри сети compose (хранение 7 дней или 512 МБ), Alloy отправляет в Loki логи контейнеров проекта `fsp-hack`, Grafana показывает дашборд API. Наружу `/metrics` закрыт в Caddy.

- В базовом `deploy/compose.yml` Grafana слушает :3001 на всех интерфейсах с анонимным просмотром. Оверлей из раздела 4.2 оставляет её только на 127.0.0.1; доступ - через `ssh -L 3001:127.0.0.1:3001 ...`.
- Пароль администратора Grafana - `GRAFANA_ADMIN_USER` и `GRAFANA_ADMIN_PASSWORD` в `deploy/.env` до первого старта.
- Alloy читает Docker socket от root: на сервере, где это недопустимо, профиль `obs` не включают.
- Свой мониторинг вместо профиля: собирать `api:8000/metrics` и `embedder:8093/metrics` из сети compose, логи - JSON в stdout контейнеров (`LOG_FORMAT=json`), проверки живости - `/healthz` и `/readyz` через Caddy.

### 4.9. Демо-данные

- `SEED_DEMO=false` в `.env` **до первого старта**. Сид запускается при старте API, только если в базе нет ни одной компании, и создаёт аккаунты с общим паролем `demo12345`, в том числе модератора `moderator@demo.fsp`.
- Удаления демо-данных нет. Если база уже заполнена сидом, её пересоздают: `docker compose -f deploy/compose.yml -f deploy/compose.prod.yml down`, затем `docker volume rm fsp-hack_pgdata` (все данные базы теряются) и снова `up -d`.
- `FSP_DEV_LINK=false`: иначе любой соискатель привяжет к себе любой ФСП ID, просто введя номер.
- Модератора на чистой базе регистрацией не создать - раздел 5.8.

### 4.10. Ключи и секреты

`deploy/render_env.py` генерирует все ключи разом из `deploy/.env.template`; это шаблон демо-сервера, после генерации в нём меняют значения по разделам 4.3-4.9. Ключи сохраняют вне сервера (менеджер паролей): дамп базы без `FIELD_ENC_KEY` и `BIDX_KEY` бесполезен.

| Переменная | Как сгенерировать | Что будет, если сменить позже |
|:--|:--|:--|
| `JWT_ED25519_SEED_B64` | `openssl rand -base64 32` | Выданные access-токены (живут 15 минут) перестают проходить проверку; браузер сам получает новый по refresh-cookie, клиенты с Bearer входят заново. Без значения ключ случайный на каждый старт; у нескольких реплик API он должен быть одинаковым |
| `FIELD_ENC_KEY` | `openssl rand -base64 32` | Не расшифровываются email и контакты пользователей, секреты вебхуков компаний, тела событий в очереди и видеозаписи тестов. Перешифровки нет: менять нельзя |
| `BIDX_KEY` | `openssl rand -base64 32` | Вход по email не находит аккаунты, тот же email можно зарегистрировать снова, ссылки отписки в отправленных письмах перестают работать. Менять нельзя |
| `POSTGRES_PASSWORD` и пароль в `DATABASE_URL` | `openssl rand -hex 24` (без символов, которые ломают URL) | Образ PostgreSQL применяет его только при создании тома; позже - `ALTER USER` в базе и новое значение в обеих переменных |
| `FSP_OIDC_CLIENT_SECRET` | выдаёт провайдер ФСП ID; для своего Keycloak - `openssl rand -hex 24` | Менять у провайдера и в `.env` одновременно; привязки, начатые в момент смены, завершатся `verification_failed`, повтор проходит. Наш Keycloak читает значение из `.env` только при первом импорте realm, позже секрет меняют и в его админке |
| `FSP_REGISTRY_TOKEN` | выдаёт реестр | Между сменой у реестра и в `.env` чтение реестра отвечает 502 `fsp_unavailable`; привязки и снимки сохраняются |
| `FSP_REGISTRY_WEBHOOK_SECRET` | `openssl rand -hex 32`, передать реестру | События со старой подписью получают 401 `signature_invalid`. На время смены реестр может подписывать обоими секретами: заголовок допускает несколько `v1` |
| `KC_BOOTSTRAP_ADMIN_PASSWORD` | `openssl rand -hex 16` | Keycloak создаёт по нему администратора только на пустой базе; позже пароль меняют в админке Keycloak |
| `GRAFANA_ADMIN_PASSWORD` (`deploy/.env`) | `openssl rand -hex 16` | Grafana берёт его только при первом старте с пустым томом `grafana-data`; позже - в интерфейсе Grafana |
| Секреты вебхуков и API-ключи компаний | создаёт платформа в кабинете компании | Компания меняет сама: секрет - сутки подписи двумя секретами, ключ - отзыв и выпуск нового |

`FIELD_ENC_KEY` и `BIDX_KEY` задаются только вместе. Если не задан ни один, API работает на фиксированных ключах из исходного кода и пишет в лог предупреждение `FIELD_ENC_KEY/BIDX_KEY not set`.

### 4.11. Чек-лист перед запуском

1. DNS указывает на сервер, открыты только 22, 80 и 443.
2. `.env` создан с новыми ключами (раздел 4.10), ключи сохранены вне сервера.
3. `PUBLIC_URL` и `SITE_URL` (в `deploy/.env`) - `https://` адрес домена, `COOKIE_SECURE=true`.
4. `SEED_DEMO=false`, `FSP_DEV_LINK=false`, `WEBHOOK_DEV_HOSTS` пустой, `ATS_DEMO_*` удалены.
5. `deploy/Caddyfile.prod` и `deploy/compose.prod.yml` созданы (раздел 4.2), Mailpit и заглушки не запущены: `docker compose -f deploy/compose.yml -f deploy/compose.prod.yml ps`.
6. SMTP настроен, SPF, DKIM и DMARC опубликованы, письмо подтверждения регистрации дошло.
7. ФСП ID: redirect URI зарегистрирован у провайдера, привязка настоящей учётной записью заканчивается `?fsp=linked`.
8. Реестр: `FSP_REGISTRY_URL` и токен заданы, "Обновить из реестра" загружает достижения; подписанное тестовое событие получает 202 (раздел 4.4).
9. `https://example.ru/readyz` отвечает 200; в логе API нет предупреждений `FIELD_ENC_KEY/BIDX_KEY not set` и `JWT_ED25519_SEED_B64 not set`.
10. Создан аккаунт модератора (раздел 5.8).
11. Резервные копии: дамп базы и `.env` по расписанию (`deploy/ops/install.sh`, оговорка о `deploy/ops/backup.sh` - в разделе 4.12); видеозаписи тестов лежат в томе `proctoring-data` и в дамп базы не входят.

### 4.12. Что потребует доработки кода

- **Токен реестра только статический.** `FSP_REGISTRY_TOKEN` уходит как есть; получения токена по client credentials и его обновления по сроку нет (`crates/api/src/fsp_client.rs`, `fetch_achievements`).
- **Требования OIDC зашиты в код.** Scope только `openid`, claim только `fsp_id` и только в `id_token`, подпись только RS256, аутентификация клиента только `client_secret_basic`. Другой провайдер подключается без правок кода только через Keycloak-брокер (раздел 4.3).
- **Одна непонятная запись обнуляет все достижения.** Снимок разбирается целиком (`crates/api/src/pool.rs`, `achievements`): одно значение перечисления вне контракта - и у кандидата пропадают ФСП-скор и навыки из ФСП, без записи в лог.
- **ФСП ID подставляется в путь запроса к реестру без кодирования.** ID с `/`, `?` или `#` уйдёт не на тот адрес; для формата `FSP-NNNNNN` это не проявляется.
- **Нет периодической сверки с реестром.** Если событие реестра потерялось, достижения обновятся только при следующем событии или по кнопке соискателя.
- **Нет ротации `FIELD_ENC_KEY` и `BIDX_KEY`.** Ключ один, команды перешифровки нет; версия в формате шифротекста (`v1`) есть, идентификатора ключа нет.
- **Модератора не создать без демо-сида.** Роль `admin` выдаёт только сид (`moderator@demo.fsp`), регистрация её отклоняет, отдельной команды нет. Обходной путь - роль записью в базе, раздел 5.8.

- **Нет удаления демо-данных.** Сид однократный, обратной команды нет: переход с демо-базы на боевую - только новой базой (раздел 4.9).
- **`deploy/ops/backup.sh` привязан к демо-оверлею.** Скрипт вызывает compose с `deploy/compose.demo.yml`, а тот требует `SITE_URL` и `MAILPIT_UI_AUTH` в `deploy/.env`; без `MAILPIT_UI_AUTH` резервная копия не создаётся. Для прода - заменить в скрипте оверлей на `deploy/compose.prod.yml` или оставить в `deploy/.env` любое значение `MAILPIT_UI_AUTH`.

## 5. Эксплуатация

Команды ниже - для локального стека (`deploy/compose.yml`). На демо-сервере к ним добавляется `-f deploy/compose.demo.yml` (раздел 2.4).

### 5.1. Логи

```sh
make logs                                        # API, в реальном времени
docker compose -f deploy/compose.yml logs -f caddy embedder
```

API пишет JSON (`LOG_FORMAT=json`, `pretty` - для чтения глазами), уровень задаёт `RUST_LOG` (в `.env.example` - `info,api=debug`). Docker хранит у каждого контейнера три файла логов по 10 МБ.

У каждого ответа API есть заголовок `x-request-id`: UUID, который API создаёт сам или берёт из запроса клиента. В логе API идентификатора нет; найти запрос по нему можно в журнале доступа Caddy, где записаны заголовки ответа:

```sh
docker compose -f deploy/compose.yml logs caddy | grep <x-request-id>
```

Там время, путь и статус, по ним - строки API рядом. Каждый HTTP-запрос в логе API появится с `RUST_LOG=info,api=debug,tower_http=debug`.

С профилем `obs` логи всех контейнеров, кроме самих Loki и Alloy, собирает Alloy в Loki (хранение - 7 дней); в Grafana они на панели "Логи API" и в Explore.

### 5.2. Метрики

`/metrics` в формате Prometheus есть у API и у эмбеддера; Caddy снаружи на него отвечает 404. Prometheus (профиль `obs`) опрашивает оба раз в 15 секунд и хранит 7 дней, не больше 512 МБ.

Grafana - http://localhost:3001, домашний дашборд "API: запросы, ошибки, задержки": запросы в секунду, доля 5xx, p50 и p95 по маршрутам, попадания в кеши, доставки вебхуков и их очередь, загрузки видео, отказы по лимитам, очередь эмбеддингов и время прохода модели.

| Метрика | Что считает |
|:--|:--|
| `http_requests_total`, `http_request_duration_seconds` | Запросы и задержки по шаблону маршрута |
| `cache_lookups_total` | Попадания и промахи кешей: пул кандидатов, карточки, статистика категорий, компания |
| `webhook_deliveries_total`, `webhook_outbox_pending` | Доставки вебхуков ATS и очередь неотправленных |
| `proctoring_chunks_total`, `proctoring_chunk_bytes_total`, `proctoring_events_total` | Загрузки видеофрагментов и событий теста |
| `rate_limit_rejections_total` | Отказы 429 |
| `embeddings_total`, `embedding_jobs_pending` | Посчитанные векторы и очередь на расчёт |
| `embedder_requests_total`, `embedder_texts_total`, `embedder_inference_seconds` | Эмбеддер: запросы, тексты, время прохода модели |

Накопленные наблюдения гистограмм API сбрасывает сам раз в 5 секунд, поэтому без Prometheus его память не растёт.

### 5.3. Проверки здоровья

| Путь | Ответ |
|:--|:--|
| `GET /healthz` | 200 `ok`: процесс жив; базу не трогает |
| `GET /readyz` | 200 или 503 с телом `db=<bool> kv=<bool>`; без `VALKEY_URL` kv всегда `true` |

Оба доступны через Caddy: http://localhost:8080/healthz. Healthcheck контейнеров `api`, `embedder`, `fsp-mock` и `ats-mock` - их `/healthz`, Keycloak - `/health/ready` на порту 9000. Упавший контейнер Docker перезапускает сам (`restart: unless-stopped`), а зависший с проваленным healthcheck - только watchdog демо-сервера (раздел 2.4).

### 5.4. Резервные копии

Ночные дампы и восстановление на демо-сервере - раздел 2.4. На любом другом стенде - тот же `pg_dump`:

```sh
docker compose -f deploy/compose.yml exec -T postgres \
  sh -c 'pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" --format=custom' > fsp.dump
```

Рядом с дампом храните `.env`: без `FIELD_ENC_KEY` и `BIDX_KEY` персональные данные из дампа не расшифровать и по email не войти. Видеозаписи лежат файлами в томе `proctoring-data` и зашифрованы тем же ключом.

### 5.5. Обновление

```sh
git pull && make up
```

API применяет новые миграции при старте под advisory lock - несколько реплик могут стартовать одновременно. Только миграции, без запуска сервера:

```sh
docker compose -f deploy/compose.yml run --rm api migrate
```

Миграции только добавляются и не откатываются: вернуться на прежнюю версию после миграции - значит восстановить дамп, снятый до обновления. `SITE_URL` попадает в образ web при сборке, поэтому после его смены web нужно пересобрать (`make up` это делает).

### 5.6. Несколько реплик API

Общее состояние API - в PostgreSQL, Valkey и каталоге видеофрагментов, поэтому реплик может быть несколько. Условия:

- `VALKEY_URL` у всех реплик: в Valkey общие лимиты запросов, отозванные access-токены и счётчик поколения кеша кандидатов; без него у каждой реплики всё это своё.
- Одинаковые `JWT_ED25519_SEED_B64`, `FIELD_ENC_KEY` и `BIDX_KEY`: иначе токен, выданный одной репликой, не примет другая, а данные, записанные одной, не прочитает другая.
- Общий каталог `PROCTORING_DIR`: видеофрагменты - файлы на диске.
- `DATABASE_POOL_SIZE` (16 по умолчанию) на число реплик не больше `max_connections` PostgreSQL (по умолчанию 100).
- В `deploy/Caddyfile` у `reverse_proxy` указан один адрес `api:8000`; для балансировки перечислите там все реплики.

У каждой реплики свой кеш в памяти: снимок пула кандидатов и карточек, компания работодателя, статистика категорий. Изменение, сделанное через одну реплику, доходит до остальных через счётчик в Valkey не дольше чем за 250 мс; жёсткий срок жизни снимка - 30 секунд. Фоновые задачи работают в каждой реплике и делят работу через `FOR UPDATE SKIP LOCKED`: очередь эмбеддингов, outbox вебхуков, письма-уведомления. Эмбеддер масштабируется отдельно от API.

### 5.7. Эмбеддер: память и выключатель

Эмбеддер занимает под нагрузкой до ~250 МБ (своих ~180 МБ, остальное - страницы файла весов) при лимите контейнера 384 МБ. В `deploy/compose.yml` ему заданы `EMBEDDER_CONCURRENCY=1` и `RAYON_NUM_THREADS=2`: один проход модели за раз на двух потоках, профиль считается за 50-90 мс. На одноядерной машине там же ставят `RAYON_NUM_THREADS=1`. Запросы API эмбеддер не ждут: векторы считает фоновый воркер после записи текста, очередь видна в метрике `embedding_jobs_pending`.

Выключить эмбеддинги целиком - `EMBEDDINGS=off` в `.env`: близость описаний считается по ключевым словам, похожие вакансии - по навыкам, пересказы чужих вакансий не ищутся. Сервис тогда можно не поднимать:

```sh
docker compose -f deploy/compose.yml --profile full up -d --build --scale embedder=0
```

Остановленный эмбеддер ничего не ломает: посчитанные векторы продолжают работать, новые тексты ждут в очереди и считаются, когда сервис вернётся.

### 5.8. Роли и модераторы

Соискатель и работодатель получают роль при регистрации. Роль модератора (`admin` в API) регистрацией не выдаётся (422): только записью в базе. Email в базе зашифрован, поэтому пользователя находят по id:

1. Зарегистрируйте аккаунт соискателя на сайте и подтвердите почту. Профиль и тест не заполняйте.
2. Войдите и откройте http://localhost:8080/api/v1/me - поле `id`.
3. Смените роль:

   ```sh
   docker compose -f deploy/compose.yml exec postgres \
     sh -c 'psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"'
   ```

   ```sql
   UPDATE users SET role = 'admin' WHERE id = '<id>';
   ```

4. Роль записана и в access-токене, поэтому новая действует после следующего обновления сессии (не позже 15 минут) или повторного входа. Кабинет модератора - `/admin`.

Снять роль - тот же `UPDATE` с `role = 'candidate'`. В демо-данных модератор уже есть: `moderator@demo.fsp`.

### 5.9. Запросы о персональных данных

- **Выгрузка (ст. 14 152-ФЗ).** Кнопка "Скачать мои данные" в настройках соискателя и на странице компании - это `GET /api/v1/me/export`: JSON-файл формата `fsp-talents-export/1` со всем, что хранится об аккаунте (состав - README, раздел 8.2). Видео в файл не входит, только сводки записей.
- **Удаление.** Соискатель удаляет аккаунт в настройках, работодатель - запросом `DELETE /api/v1/me` (в Swagger; кнопки в кабинете компании нет). Удаляются профиль соискателя или компания, данные законного представителя, уведомления и видеофайлы; строка пользователя остаётся обезличенной, чтобы журналы ссылались на существующий id; все сессии отзываются.
- **Законный представитель** отзывает согласие по ссылке из письма (`/guardian/confirm`); профиль подростка сразу скрывается.
- **Видеозапись.** Отзыв согласия на видеозапись в настройках сразу удаляет все записи соискателя.

Инструмента, которым оператор выгружает или удаляет чужой аккаунт, нет: оба действия выполняет сам пользователь. Для остальных запросов страница `/privacy` отсылает к ответственному за обработку персональных данных; его контакт вписывают туда при вводе в эксплуатацию.

### 5.10. Сроки хранения и фоновая очистка

| Данные | Срок | Когда удаляются |
|:--|:--|:--|
| Видеофрагменты теста | `PROCTORING_RETENTION_DAYS` (90 дней); сводка и отметка остаются | При старте API и каждые 6 часов |
| ФИО, email, IP и браузер представителя, дата рождения подростка | 30 дней после 18-летия; при удалении аккаунта - сразу | При старте API и каждые 6 часов |
| События вебхуков ATS после доставки или отказа, с данными кандидата | 30 дней | Раз в час |
| Идентификаторы событий реестра ФСП | 30 дней | При обработке следующего события |
| Состояния привязки ФСП ID (state, PKCE) | Действуют 10 минут | При следующем начале привязки |
| Письма в Mailpit | Последние 500 | Mailpit |
| Метрики Prometheus, логи Loki | 7 дней | Prometheus, Loki |

Очистку выполняет работающий сервер; команды `api migrate` и `api seed` её не запускают.

### 5.11. Смена ключей

Что происходит при смене каждого ключа и секрета - раздел 4.10. Два случая, которые там не описаны:

- `FSP_OIDC_CLIENT_SECRET` у нашего Keycloak: в режиме `start-dev` данные Keycloak живут в контейнере, поэтому `make up` пересоздаёт его и realm импортируется с новым значением.
- `ATS_DEMO_*`: ключ и вебхук демо-компании API выдаёт при старте, но существующий вебхук не перезаписывает - новый секрет применится на чистой базе или после удаления вебхука в разделе интеграций компании и перезапуска API.

### 5.12. Команды бинаря `api`

| Команда | Что делает |
|:--|:--|
| `api` | Сервер: миграции, демо-данные при `SEED_DEMO=true`, фоновые задачи, HTTP |
| `api migrate` | Миграции и выход |
| `api seed` | Миграции, демо-данные на пустой базе (в ней нет ни одной компании) и выход; `SEED_DEMO` не нужен |
| `api openapi` | Печатает спецификацию OpenAPI в stdout; ни базы, ни переменных не требует |

В Docker - через `run`, точка входа образа уже `api`:

```sh
docker compose -f deploy/compose.yml run --rm api seed
```

## 6. Частые вопросы и неполадки

**Где документация API?**
Swagger UI - http://localhost:8080/api/docs (на демо-сервере - https://demo.arewehireyet.ru/api/docs), спецификация OpenAPI - `/api/v1/openapi.json`. Правила, которых не видно из схем (формат ошибок, лимиты, кеширование, вебхуки, подписи), - в `docs/api.md`; обзор - README, раздел 9.

**Письмо не пришло. Где письма?**
В Docker все письма платформы ловит Mailpit: http://localhost:8025, на демо-сервере - `/mail/` (раздел 2.4). Письмо подтверждения почты уходит сразу; повторное - не больше 3 в час на адрес. Письма-уведомления копятся: первое уходит через 2 минуты, дальше не чаще одного письма в 10 минут и не больше 10 в сутки на адрес (`NOTIFY_EMAIL_*`, раздел 3). Под `make api` без `MAIL_MODE=smtp` письма печатаются в лог API. Ошибку SMTP API пишет в свой лог (`make logs`). Mailpit хранит последние 500 писем.

**Действия на сайте отклоняются: "Запрос отклонён: неверный источник" (403 `csrf`).**
Сайт открыт не по адресу из `PUBLIC_URL`. Изменяющие запросы с cookie API принимает только с этого Origin: при `PUBLIC_URL=http://localhost:8080` не годятся ни `127.0.0.1:8080`, ни адрес машины в сети. Откройте сайт по `PUBLIC_URL` или поменяйте его в `.env` и выполните `make up`.

**После входа сразу снова просят войти.**
Скорее всего, `COOKIE_SECURE=true`, а сайт открыт по http не на localhost: браузер не сохраняет Secure-cookie. По http оставьте `COOKIE_SECURE=false`.

**После перезапуска всех выкинуло из кабинетов.**
Обычный перезапуск сессии не рвёт: refresh-токены хранятся в базе, и кабинет сам получает новый access-токен, даже если `JWT_ED25519_SEED_B64` не задан и ключ подписи сменился. Сессии пропадают, если удалён том базы (`down -v`) или сменились `FIELD_ENC_KEY` и `BIDX_KEY` - тогда и вход по email невозможен (раздел 5.11).

**429 во время показа.**
Лимиты считаются в Valkey (полная таблица - `docs/api.md`, "Лимиты запросов"):

| Действие | Лимит |
|:--|:--|
| Вход | 10 попыток за 15 минут на пару IP + email |
| Регистрация | 20 в час с IP |
| Повтор письма подтверждения | 3 в час на адрес |
| Письмо законному представителю | 3 в час от соискателя, 5 в сутки на адрес |

В ответе есть `Retry-After`. Если жюри сидит за одним NAT, регистрации с общего IP быстро упираются в 20 в час, а входы под разными аккаунтами друг другу не мешают. Сбросить все счётчики - перезапустить Valkey (он ничего не хранит на диске):

```sh
docker compose -f deploy/compose.yml restart valkey
```

Вместе со счётчиками сбрасываются отозванные access-токены: токен вышедшего пользователя снова действует до конца своих 15 минут.

**Камера не предлагается перед тестом.**
Браузер даёт доступ к камере только защищённой странице: по HTTPS или на `localhost`. По http на адресе машины в сети шаг с камерой не появится. Ещё причины: `PROCTORING=off` в `.env`; соискатель не принял согласие на видеозапись; камера запрещена в настройках сайта в браузере. Caddy разрешает странице только камеру (`Permissions-Policy: camera=(self)`), микрофон - нет.

**Запись теста у демо-соискателя не воспроизводится.**
Так и задумано: в демо-данных есть метаданные записей, но нет самих видеофайлов. Свежие записи отвечают `recording_unavailable`, записи старше срока хранения помечены удалёнными. Чтобы посмотреть настоящую запись, пройдите тест с камерой новым соискателем.

**Привязка ФСП ID не работает.**
По ответу:

- 502 `fsp_unavailable` сразу после кнопки - Keycloak не запущен (нет профиля `fsp`) или ещё стартует (до минуты).
- 503 `fsp_oidc_disabled` - не заданы `FSP_OIDC_ISSUER` и `FSP_OIDC_CLIENT_SECRET` (так под `make api`).
- Keycloak пишет "Invalid parameter: redirect_uri" - `FSP_OIDC_REDIRECT_URL` не совпадает с адресом, который Keycloak импортировал в realm. Поправьте `.env` и выполните `make up`: контейнер Keycloak пересоздастся и импортирует realm заново.
- Возврат в настройки с `?fsp=error&reason=verification_failed` - чаще всего `FSP_OIDC_ISSUER` не совпадает с адресом Keycloak, который видит браузер (`KC_HOSTNAME` + `/realms/fsp`). Это `iss` токена; API ходит в Keycloak по внутреннему `FSP_OIDC_DISCOVERY_URL`, но проверяет именно `FSP_OIDC_ISSUER`. Подробность - в логе API, строка `fsp id_token exchange failed`.
- `reason=expired` - вход в ФСП ID занял больше 10 минут; `reason=already_linked` - этот ФСП ID уже у другого профиля. Свободны `fsp-000001` ... `fsp-000010` (раздел 2.2).

Если сайт открыт не с той машины, где запущен стек, адрес `localhost:8081` из `FSP_OIDC_ISSUER` браузеру недоступен: нужны публичные адреса (раздел 4).

**Достижения ФСП не обновляются.**
Проверьте по порядку: запущен ли профиль `fsp`; задан ли `FSP_REGISTRY_URL` (без него синхронизация отвечает 503 `fsp_registry_disabled`); задан ли `FSP_REGISTRY_WEBHOOK_SECRET` - без него реестр не может сам сообщать об изменениях, работает только синхронизация из настроек соискателя (`POST /api/v1/fsp/sync`). Если реестр недоступен, привязка сохраняется со старым снимком. Правки в заглушке реестра живут в её памяти и пропадают при перезапуске. Сценарий целиком - раздел 2.6.

**Демо-ATS не получает события, кнопки этапов не работают.**
Нужен профиль `ats`: без него доставки демо-компании падают и видны в журнале доставок раздела интеграций компании. Неудачную доставку платформа повторяет примерно через 1 минуту, 5 минут, 30 минут, 2 часа и дальше раз в 12 часов в течение суток. Кнопкам этапов в демо-ATS нужен `ATS_DEMO_API_KEY` - без него заглушка их отключает.

**Порт уже занят.**
Стек публикует 8080, 8025, 8081, 8082, 3001, а на 127.0.0.1 - 1025 и 8091. Чаще всего их держит стек black-box тестов: `make blackbox-down`. Под `make api` и `make ats-mock` порты 8000 и 8082 заняты локальными процессами. Кто занял порт:

```sh
ss -ltnp 'sport = :8080'     # Linux
lsof -iTCP:8080 -sTCP:LISTEN # macOS
```

Сменить внешний порт - левая часть `ports` в `deploy/compose.yml`; для 8080 вместе с ним меняются `PUBLIC_URL`, `SITE_URL` и `FSP_OIDC_REDIRECT_URL`, для 8081 - `KC_HOSTNAME` и `FSP_OIDC_ISSUER` (раздел 3).

**Эмбеддер: долгий первый старт, нехватка памяти, как выключить.**
Веса (~490 МБ) скачиваются при сборке образа, а не при старте; без доступа к Hugging Face - раздел 2.5. После первого старта векторы демо-данных считаются минуту-две, подбор тем временем работает на ключевых словах. Лимит контейнера - 384 МБ; если `docker inspect` показывает `OOMKilled`, на одноядерной или слабой машине поставьте `RAYON_NUM_THREADS=1` у сервиса в `deploy/compose.yml`. Выключить совсем - `EMBEDDINGS=off` и `--scale embedder=0` (раздел 5.7).

**Почему кандидата нет в поиске?**
Кандидат попадает в выдачу, только если одновременно:

- у него есть категория: пройден тест и грейд не аннулирован модератором;
- профиль опубликован в настройках приватности и действует согласие на публикацию;
- ему есть 18 или действует согласие законного представителя;
- статус поиска не "Не ищу работу";
- в подборке совпадает специализация потребности, а фильтры работодателя (грейд, "Только 18+", навыки) его пропускают.

Новый или изменённый профиль виден сразу на той реплике, через которую он сохранён, и не позже 250 мс на остальных.

**Как сбросить демо-данные?**
Демо-данные заполняются только на пустой базе (в ней нет ни одной компании), поэтому базу удаляют:

```sh
docker compose -f deploy/compose.yml --profile full down -v && make up
```

`down -v` удаляет и видео, метрики, логи. Только база:

```sh
make down && docker volume rm fsp-hack_pgdata && make up
```

Заглушка реестра ФСП возвращается к исходным данным при любом своём перезапуске. В мок-режиме фронтенда - кнопка "Сбросить демо-данные" в боковой панели кабинета.

**Как добавить работодателя или модератора?**
Работодатель регистрируется сам на `/register` как компания и подтверждает почту - письмо в Mailpit. Модератора назначают записью в базе - раздел 5.8.

**Black-box тесты испортили демо-данные?**
`make blackbox` поднимает отдельный проект `fsp-hack-blackbox` с чистыми томами на тех же портах и останавливает стек `make up`, не трогая его данные. Вернуться: `make blackbox-down && make up`. А вот запуск `npx playwright test` против стека `make up` тратит его демо-данные: разбирает очередь модератора, отвечает за представителя `minor@demo.fsp`, приглашает и откликается от имени `cand5`, `cand8`, `cand10`, `cand21`, `cand32`. После такого прогона демо-данные сбрасывают (выше).

**Grafana пустая.**
Нужен профиль `obs`. Панели заполняются после первых опросов Prometheus (раз в 15 секунд) и после первых запросов к API; логи - с момента старта Alloy.

**Что учесть на macOS?**
Нужен движок Docker: Docker Desktop или Colima (`nix develop`), причём Colima по умолчанию даёт виртуальной машине 2 ГБ - запускайте `colima start --cpu 4 --memory 6` (раздел 2.1). Образы сервисов собираются под архитектуру машины, на Apple Silicon - под arm64.
