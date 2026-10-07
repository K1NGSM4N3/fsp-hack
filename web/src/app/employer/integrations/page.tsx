"use client";

import { ArrowLeftRight, Check, ChevronDown, Copy, KeyRound, Lock, RefreshCw, RotateCcw, Send, ShieldCheck, Trash2, Webhook } from "lucide-react";
import { useState, type ReactNode } from "react";
import { isApiError } from "@/api/client";
import type { ApiKeyView, ApiScope, DeliveryStatus, DeliveryView, IntegrationEventType, IntegrationsOverview, TestDelivery } from "@/api/types";
import { Alert, Badge, Button, Card, CardHeader, Checkbox, CodeBlock, Dialog, EmptyState, ErrorAlert, Field, Input, LoadMore, PageHeader, PageSkeleton, SkeletonCard, ToggleChip, useToast } from "@/components/ui";
import {
  useApiKeys,
  useCreateApiKey,
  useDeleteWebhook,
  useDeliveries,
  useIntegrations,
  useRetryDelivery,
  useRevokeApiKey,
  useRotateWebhookSecret,
  useSaveWebhook,
  useSendTestEvent,
} from "@/features/integrations/hooks";
import { cn } from "@/lib/cn";
import { formatDateTime, formatRelative, plural } from "@/lib/format";

const EVENT_LABEL: Record<IntegrationEventType, string> = {
  "invitation.sent": "Приглашение отправлено",
  "invitation.viewed": "Кандидат открыл приглашение",
  "invitation.accepted": "Кандидат принял приглашение",
  "invitation.declined": "Кандидат отклонил приглашение",
  "invitation.expired": "Приглашение истекло",
  "invitation.withdrawn": "Приглашение отозвано",
  "application.created": "Новый отклик",
  "application.status_changed": "Статус отклика изменился",
  "contacts.opened": "Контакты открыты в кабинете",
  "integration.test": "Тестовое событие",
};

const SCOPE_INFO: Record<ApiScope, { title: string; description: string }> = {
  "invitations:read": { title: "Читать приглашения", description: "Опрос статусов — если вебхук настроить нельзя" },
  "candidates:read": { title: "Читать профили", description: "Контакты и PDF-профиль — только тех, кто принял приглашение или откликнулся" },
  "outcomes:write": { title: "Отмечать этапы найма", description: "Собеседование, оффер, найм, отказ — кандидат увидит их у себя" },
  "needs:write": { title: "Импортировать вакансии", description: "Текст вакансии из ATS превращается в потребность с подборкой" },
};

const DEFAULT_SCOPES: ApiScope[] = ["invitations:read", "candidates:read", "outcomes:write"];

const VERIFY_SNIPPET = `import hashlib, hmac, time

def verify(secret: str, header: str,
           body: bytes) -> bool:
    parts = [p.split("=", 1)
             for p in header.split(",")]
    t = next(v for k, v in parts if k == "t")
    if abs(time.time() - int(t)) > 300:
        return False
    mac = hmac.new(secret.encode(),
                   f"{t}.".encode() + body,
                   hashlib.sha256).hexdigest()
    return any(hmac.compare_digest(mac, v)
               for k, v in parts if k == "v1")`;

function errorTitle(e: unknown, fallback: string): string {
  return isApiError(e) ? (e.detail ?? e.title) : fallback;
}

function CopyField({ value, label, mono = true }: { value: string; label: string; mono?: boolean }) {
  const [copied, setCopied] = useState(false);
  return (
    <div className="flex items-stretch gap-2">
      <code className={cn("min-w-0 flex-1 rounded-control border border-line bg-canvas px-3 py-2 text-[13px] break-all text-ink", mono && "font-mono")}>{value}</code>
      <Button
        variant="secondary"
        size="md"
        icon={copied ? <Check className="size-4" aria-hidden /> : <Copy className="size-4" aria-hidden />}
        onClick={() => {
          void navigator.clipboard?.writeText(value).then(() => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1500);
          });
        }}
      >
        <span className="max-sm:sr-only">{copied ? "Скопировано" : "Копировать"}</span>
        <span className="sr-only">: {label}</span>
      </Button>
      <span role="status" className="sr-only">
        {copied ? `${label} в буфере обмена` : ""}
      </span>
    </div>
  );
}

function RevealDialog({ secret, onClose, title, children }: { secret: string | null; onClose: () => void; title: string; children: ReactNode }) {
  return (
    <Dialog
      open={secret !== null}
      onClose={onClose}
      title={title}
      description="Показываем один раз. Сохраните его в настройках ATS — потом увидите только первые символы."
      footer={<Button onClick={onClose}>Я сохранил</Button>}
    >
      <div className="space-y-4">
        {secret && <CopyField value={secret} label={title} />}
        {children}
      </div>
    </Dialog>
  );
}

function HowItWorks() {
  const steps = [
    {
      icon: <Send className="size-4" aria-hidden />,
      title: "Мы сообщаем вашей ATS",
      text: "Приглашение отправлено, открыто, принято или отклонено, пришёл отклик — на ваш адрес уходит POST-запрос с событием. Он подписан секретом: заголовок FSP-Signature.",
    },
    {
      icon: <Lock className="size-4" aria-hidden />,
      title: "Контакты — только после согласия",
      text: "Пока кандидат не принял приглашение, в событии только анонимный код и категория. Имя, телефон, почта и PDF-профиль уходят в ATS после «Принять» или отклика — и попадают в журнал доступа, который видит кандидат.",
    },
    {
      icon: <ArrowLeftRight className="size-4" aria-hidden />,
      title: "ATS отвечает по API-ключу",
      text: "Отмечает этапы найма — кандидат видит «Компания отметила: собеседование», — забирает профиль принявших и импортирует вакансии. Ключ работает только здесь, в кабинет с ним не войти.",
    },
  ];
  return (
    <Card>
      <CardHeader title="Что куда уходит" description="Подключение занимает пару минут: адрес для событий и ключ для ответов" />
      <ol className="grid gap-5 md:grid-cols-3">
        {steps.map((s, i) => (
          <li key={s.title} className="flex gap-3">
            <span className="flex size-8 shrink-0 items-center justify-center rounded-[10px] bg-primary-50 text-primary" aria-hidden>
              {s.icon}
            </span>
            <div className="min-w-0">
              <p className="text-sm font-medium text-ink">
                <span className="mr-1 font-mono text-ink-subtle">{i + 1}.</span>
                {s.title}
              </p>
              <p className="mt-1 text-sm text-ink-muted">{s.text}</p>
            </div>
          </li>
        ))}
      </ol>
    </Card>
  );
}

function TestResult({ r }: { r: TestDelivery }) {
  return r.delivered ? (
    <Alert tone="success" title="Тестовое событие доставлено">
      Ответ HTTP {r.status_code} за {r.latency_ms} мс.
    </Alert>
  ) : (
    <Alert tone="danger" title="Не доставлено">
      {r.error ?? "Адрес не ответил"}
      {r.status_code ? ` (HTTP ${r.status_code})` : ""}. Событие повторится автоматически — следите за журналом ниже.
    </Alert>
  );
}

/** `onSecret` lives in the page: saving the first address re-keys this card, and the one-time secret must outlive that. */
function WebhookCard({ data, onSecret }: { data: IntegrationsOverview; onSecret: (secret: string) => void }) {
  const toast = useToast();
  const w = data.webhook;
  const [url, setUrl] = useState(w?.url ?? "");
  const [confirm, setConfirm] = useState<"rotate" | "delete" | null>(null);
  const save = useSaveWebhook();
  const rotate = useRotateWebhookSecret();
  const remove = useDeleteWebhook();
  const test = useSendTestEvent();
  const urlError = isApiError(save.error) ? save.error.fieldError("url") : undefined;
  const dirty = url.trim() !== (w?.url ?? "");

  return (
    <Card>
      <CardHeader
        title="Вебхук"
        description="Один адрес вашей ATS, куда мы отправляем события"
        action={w ? <Badge tone="success" dot>Подключён</Badge> : <Badge tone="muted">Не настроен</Badge>}
      />
      <form
        noValidate
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate(
            { url: url.trim() },
            {
              onSuccess: (r) => {
                if (r.secret) onSecret(r.secret);
                else toast.success("Адрес сохранён", "Секрет подписи прежний");
              },
            },
          );
        }}
      >
        <Field label="Адрес для событий" required error={urlError} hint="Только https://. Адреса внутренней сети не принимаются.">
          <Input value={url} onChange={(e) => setUrl(e.target.value)} type="url" inputMode="url" placeholder="https://ats.example.ru/fsp/webhooks" autoComplete="off" />
        </Field>
        {save.isError && !urlError && <ErrorAlert error={save.error} />}
        <div className="flex flex-wrap gap-2">
          <Button type="submit" loading={save.isPending} disabled={!url.trim() || !dirty}>
            {w ? "Сохранить адрес" : "Подключить"}
          </Button>
          {w && (
            <Button variant="secondary" icon={<Send className="size-4" aria-hidden />} loading={test.isPending} onClick={() => test.mutate(undefined, { onError: (e) => toast.error(errorTitle(e, "Не удалось отправить")) })}>
              Тестовое событие
            </Button>
          )}
        </div>
      </form>

      {test.data && (
        <div className="mt-4">
          <TestResult r={test.data} />
        </div>
      )}

      {w && (
        <div className="mt-6 space-y-4 border-t border-line pt-5">
          <dl className="grid gap-4 text-sm sm:grid-cols-2">
            <div>
              <dt className="text-ink-subtle">Секрет подписи</dt>
              <dd className="mt-0.5 font-mono text-ink">{w.secret_hint}</dd>
            </div>
            <div>
              <dt className="text-ink-subtle">Секрет выпущен</dt>
              <dd className="mt-0.5 text-ink">{formatDateTime(w.secret_rotated_at)}</dd>
            </div>
          </dl>
          {w.previous_secret_valid_until && (
            <Alert tone="info">
              До {formatDateTime(w.previous_secret_valid_until)} события подписаны и новым, и прежним секретом — успейте обновить его в ATS.
            </Alert>
          )}
          <div className="flex flex-wrap gap-2">
            <Button variant="soft" size="sm" icon={<RefreshCw className="size-4" aria-hidden />} onClick={() => setConfirm("rotate")}>
              Сменить секрет
            </Button>
            <Button variant="ghost" size="sm" icon={<Trash2 className="size-4" aria-hidden />} onClick={() => setConfirm("delete")} className="text-accent-strong hover:text-accent-strong">
              Отключить вебхук
            </Button>
          </div>
        </div>
      )}

      <Dialog
        open={confirm === "rotate"}
        onClose={() => setConfirm(null)}
        size="sm"
        title="Сменить секрет?"
        footer={
          <>
            <Button variant="ghost" onClick={() => setConfirm(null)}>
              Отмена
            </Button>
            <Button
              loading={rotate.isPending}
              onClick={() =>
                rotate.mutate(undefined, {
                  onSuccess: (r) => {
                    setConfirm(null);
                    if (r.secret) onSecret(r.secret);
                  },
                  onError: (e) => toast.error(errorTitle(e, "Не удалось сменить секрет")),
                })
              }
            >
              Сменить
            </Button>
          </>
        }
      >
        <p className="text-sm text-ink-muted">Новый секрет покажем один раз. Сутки события будут подписаны обоими секретами, так что ATS не потеряет ни одного.</p>
      </Dialog>

      <Dialog
        open={confirm === "delete"}
        onClose={() => setConfirm(null)}
        size="sm"
        title="Отключить вебхук?"
        footer={
          <>
            <Button variant="ghost" onClick={() => setConfirm(null)}>
              Отмена
            </Button>
            <Button
              variant="danger"
              loading={remove.isPending}
              onClick={() =>
                remove.mutate(undefined, {
                  onSuccess: () => {
                    setConfirm(null);
                    setUrl("");
                    toast.success("Вебхук отключён");
                  },
                  onError: (e) => toast.error(errorTitle(e, "Не удалось отключить")),
                })
              }
            >
              Отключить
            </Button>
          </>
        }
      >
        <p className="text-sm text-ink-muted">Адрес и секрет удалятся, события из очереди не будут отправлены. Новые события перестанут формироваться — и контакты в ATS больше не уйдут.</p>
      </Dialog>

    </Card>
  );
}

function VerifyCard() {
  return (
    <Card>
      <CardHeader title="Как проверить подпись" description="Тело — сырые байты запроса, до разбора JSON. Повторы отбрасывайте по заголовку FSP-Event-Id" />
      <CodeBlock code={VERIFY_SNIPPET} language="python" />
    </Card>
  );
}

function StatusBadge({ d }: { d: DeliveryView }) {
  if (d.status === "delivered") return <Badge tone="success" dot>Доставлено</Badge>;
  if (d.status === "failed") return <Badge tone="danger" dot>Не доставлено</Badge>;
  return (
    <Badge tone="warning" dot title={d.next_attempt_at ? formatDateTime(d.next_attempt_at) : undefined}>
      В очереди{d.next_attempt_at ? ` · ${formatRelative(d.next_attempt_at)}` : ""}
    </Badge>
  );
}

function DeliveryRow({ d }: { d: DeliveryView }) {
  const toast = useToast();
  const retry = useRetryDelivery();
  const [open, setOpen] = useState(false);
  const last = d.attempts.at(-1);
  return (
    <li className="py-3 first:pt-0 last:pb-0">
      <div className="grid grid-cols-[1fr_auto] items-start gap-x-3 gap-y-2 sm:grid-cols-[minmax(0,1fr)_13rem_auto] sm:items-center">
        <button type="button" onClick={() => setOpen((v) => !v)} aria-expanded={open} className="group tap min-w-0 text-left">
          <span className="flex items-center gap-1.5 text-sm font-medium text-ink group-hover:text-primary">
            <ChevronDown className={cn("size-4 shrink-0 text-ink-subtle transition-transform", open && "rotate-180")} aria-hidden />
            <span className="truncate">{EVENT_LABEL[d.event_type]}</span>
          </span>
          <span className="mt-0.5 block pl-5.5 text-xs text-ink-subtle">
            {formatDateTime(d.created_at)} · попыток {d.attempts.length}
            {last?.status_code ? ` · HTTP ${last.status_code}, ${last.latency_ms} мс` : ""}
          </span>
        </button>
        <div className="col-start-2 row-start-1 justify-self-end sm:col-start-auto sm:row-start-auto sm:justify-self-start">
          <StatusBadge d={d} />
        </div>
        <div className="col-span-2 pl-5.5 sm:col-span-1 sm:pl-0">
          <Button
            variant="ghost"
            size="sm"
            icon={<RotateCcw className="size-4" aria-hidden />}
            loading={retry.isPending}
            onClick={() =>
              retry.mutate(d.event_id, {
                onSuccess: () => toast.success("Событие снова в очереди", "Отправим в ближайшие секунды"),
                onError: (e) => toast.error(errorTitle(e, "Не удалось поставить в очередь")),
              })
            }
          >
            Отправить ещё раз
          </Button>
        </div>
      </div>
      {open && (
        <div className="mt-3 ml-5.5 space-y-2 rounded-control border border-line bg-canvas p-3 text-xs">
          <p className="font-mono break-all text-ink-subtle">FSP-Event-Id: {d.event_id}</p>
          {d.attempts.length === 0 ? (
            <p className="text-ink-muted">Попыток ещё не было.</p>
          ) : (
            <ol className="space-y-1.5">
              {d.attempts.map((a) => (
                <li key={a.at} className="flex flex-wrap gap-x-3 gap-y-0.5 text-ink-muted">
                  <span className="tabular text-ink">{formatDateTime(a.at)}</span>
                  <span className={cn("font-mono", a.status_code && a.status_code >= 200 && a.status_code < 300 ? "text-success" : "text-accent-strong")}>{a.status_code ? `HTTP ${a.status_code}` : "нет ответа"}</span>
                  <span className="tabular">{a.latency_ms} мс</span>
                  {a.error && <span className="w-full break-words">{a.error}</span>}
                </li>
              ))}
            </ol>
          )}
        </div>
      )}
    </li>
  );
}

type DeliveryFilter = "all" | DeliveryStatus;

const FILTER_LABEL: Record<DeliveryFilter, string> = { all: "Все", delivered: "Доставлено", pending: "В очереди", failed: "Не доставлено" };

function DeliveriesCard({ counts }: { counts: IntegrationsOverview["deliveries"] }) {
  const [filter, setFilter] = useState<DeliveryFilter>("all");
  const q = useDeliveries(filter === "all" ? null : filter);
  const total = counts.pending + counts.delivered + counts.failed;
  const count = (f: DeliveryFilter) => (f === "all" ? total : counts[f]);
  const list = q.data?.pages.flatMap((p) => p.items) ?? [];
  return (
    <Card id="deliveries" className="scroll-mt-24">
      <CardHeader title="Журнал доставок" description="Новые сверху, хранятся 30 дней. Неудачные повторяем через 1 мин, 5 мин, 30 мин, 2 ч и 12 ч; через сутки сдаёмся" />
      {total === 0 && !q.isPending && list.length === 0 ? (
        <EmptyState compact icon={<Webhook className="size-5" aria-hidden />} title="Событий пока нет" description="Подключите вебхук и нажмите «Тестовое событие» — оно появится здесь вместе с ответом вашей ATS." />
      ) : (
        <>
          <div role="group" aria-label="Фильтр по статусу доставки" className="mb-4 flex flex-wrap gap-2">
            {(["all", "delivered", "pending", "failed"] as DeliveryFilter[]).map((f) => (
              <ToggleChip key={f} selected={filter === f} onToggle={() => setFilter(f)}>
                {FILTER_LABEL[f]}
                <span className="tabular font-mono text-xs opacity-80">{count(f)}</span>
              </ToggleChip>
            ))}
          </div>
          {q.isPending ? (
            <SkeletonCard lines={3} className="border-0 p-0" />
          ) : q.isError ? (
            <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
          ) : list.length === 0 ? (
            <p className="rounded-control bg-canvas px-4 py-3 text-sm text-ink-muted">Событий с таким статусом нет.</p>
          ) : (
            <ul className="divide-y divide-line">
              {list.map((d) => (
                <DeliveryRow key={d.event_id} d={d} />
              ))}
            </ul>
          )}
          <LoadMore key={filter} pager={q} loaded={list.length} noun="События" className="border-t border-line pt-4" />
        </>
      )}
    </Card>
  );
}

function KeyRow({ k }: { k: ApiKeyView }) {
  const toast = useToast();
  const revoke = useRevokeApiKey();
  const [confirm, setConfirm] = useState(false);
  return (
    <li className="py-3 first:pt-0 last:pb-0">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
        <div className="min-w-0">
          <p className="flex flex-wrap items-center gap-2 text-sm font-medium text-ink">
            {k.name}
            {k.revoked_at ? <Badge tone="muted">Отозван</Badge> : <Badge tone="success" dot>Действует</Badge>}
          </p>
          <p className="mt-0.5 font-mono text-xs break-all text-ink-subtle">{k.prefix}…</p>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {k.scopes.map((s) => (
              <Badge key={s} tone={k.revoked_at ? "muted" : "primary"} title={SCOPE_INFO[s].description}>
                {SCOPE_INFO[s].title}
              </Badge>
            ))}
          </div>
          <p className="mt-2 text-xs text-ink-subtle">
            Создан {formatDateTime(k.created_at)} · {k.last_used_at ? `последний запрос ${formatRelative(k.last_used_at)}` : "ещё не использовался"}
            {k.revoked_at ? ` · отозван ${formatDateTime(k.revoked_at)}` : ""}
          </p>
        </div>
        {!k.revoked_at && (
          <Button variant="ghost" size="sm" icon={<Trash2 className="size-4" aria-hidden />} onClick={() => setConfirm(true)} className="self-start text-accent-strong hover:text-accent-strong">
            Отозвать
          </Button>
        )}
      </div>
      <Dialog
        open={confirm}
        onClose={() => setConfirm(false)}
        size="sm"
        title={`Отозвать ключ «${k.name}»?`}
        footer={
          <>
            <Button variant="ghost" onClick={() => setConfirm(false)}>
              Отмена
            </Button>
            <Button
              variant="danger"
              loading={revoke.isPending}
              onClick={() =>
                revoke.mutate(k.id, {
                  onSuccess: () => {
                    setConfirm(false);
                    toast.success("Ключ отозван");
                  },
                  onError: (e) => toast.error(errorTitle(e, "Не удалось отозвать")),
                })
              }
            >
              Отозвать
            </Button>
          </>
        }
      >
        <p className="text-sm text-ink-muted">Ключ перестанет работать сразу. ATS, которая им пользуется, начнёт получать 401.</p>
      </Dialog>
    </li>
  );
}

function ApiKeysCard({ data }: { data: IntegrationsOverview }) {
  const q = useApiKeys();
  const create = useCreateApiKey();
  const [name, setName] = useState("");
  const [scopes, setScopes] = useState<ApiScope[]>(DEFAULT_SCOPES);
  const [secret, setSecret] = useState<string | null>(null);
  const err = (f: string) => (isApiError(create.error) ? create.error.fieldError(f) : undefined);
  const toggle = (s: ApiScope, on: boolean) => setScopes((cur) => (on ? [...cur, s] : cur.filter((x) => x !== s)));

  return (
    <Card>
      <CardHeader title="API-ключи" description="Для запросов вашей ATS к платформе: Authorization: Bearer fsp_live_…" action={<Badge tone="neutral">{data.active_keys} действующих</Badge>} />
      <form
        noValidate
        className="space-y-4 rounded-control border border-line p-4"
        onSubmit={(e) => {
          e.preventDefault();
          create.mutate(
            { name: name.trim(), scopes },
            {
              onSuccess: (r) => {
                setSecret(r.secret);
                setName("");
              },
            },
          );
        }}
      >
        <Field label="Название" required error={err("name")} hint="Чтобы потом понять, где он используется">
          <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="Huntflow" maxLength={100} />
        </Field>
        <fieldset>
          <legend className="mb-2 text-sm font-medium text-ink">Что разрешено</legend>
          <div className="grid gap-3 sm:grid-cols-2">
            {data.scopes.map((s) => (
              <Checkbox key={s} checked={scopes.includes(s)} onChange={(e) => toggle(s, e.target.checked)} label={SCOPE_INFO[s].title} description={SCOPE_INFO[s].description} />
            ))}
          </div>
          {err("scopes") && <p className="mt-2 text-sm text-accent-strong">{err("scopes")}</p>}
        </fieldset>
        {create.isError && !err("name") && !err("scopes") && <ErrorAlert error={create.error} />}
        <Button type="submit" icon={<KeyRound className="size-4" aria-hidden />} loading={create.isPending} disabled={!name.trim() || scopes.length === 0}>
          Создать ключ
        </Button>
      </form>

      <div className="mt-5">
        {q.isPending ? (
          <SkeletonCard lines={2} className="border-0 p-0" />
        ) : q.isError ? (
          <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
        ) : q.data.length === 0 ? (
          <p className="text-sm text-ink-subtle">Ключей пока нет.</p>
        ) : (
          <ul className="divide-y divide-line">
            {q.data.map((k) => (
              <KeyRow key={k.id} k={k} />
            ))}
          </ul>
        )}
      </div>

      <div className="mt-6 space-y-2 border-t border-line pt-5">
        <p className="text-sm font-medium text-ink">Адрес API для ATS</p>
        <CopyField value={data.api_base_url} label="Адрес API" />
        <p className="text-sm text-ink-muted">
          Методы и схемы событий —{" "}
          <a href="/api/docs" target="_blank" rel="noreferrer" className="link-inline font-medium">
            в Swagger, раздел «Интеграции ATS»
          </a>
          .
        </p>
      </div>

      <RevealDialog secret={secret} onClose={() => setSecret(null)} title="API-ключ">
        <p className="text-sm text-ink-muted">Проверить ключ можно так:</p>
        <code className="block rounded-control bg-night p-3 font-mono text-xs break-all text-white">
          curl -H &quot;Authorization: Bearer {secret ?? "fsp_live_…"}&quot; {data.api_base_url}/whoami
        </code>
      </RevealDialog>
    </Card>
  );
}

export default function IntegrationsPage() {
  const q = useIntegrations();
  const [webhookSecret, setWebhookSecret] = useState<string | null>(null);
  return (
    <div>
      <PageHeader
        eyebrow={
          <span className="eyebrow inline-flex items-center gap-1.5 text-primary">
            <ShieldCheck className="size-3.5" aria-hidden />
            ATS компании
          </span>
        }
        title="Интеграции"
        description="Подключите свою систему подбора: события о приглашениях и откликах придут туда сами, а этапы найма вернутся обратно к кандидату."
        actions={
          q.data ? (
            <>
              {q.data.webhook ? (
                <Badge tone="success" dot>
                  Вебхук подключён
                </Badge>
              ) : (
                <Badge tone="muted">Вебхук не настроен</Badge>
              )}
              <Badge tone="neutral">
                {q.data.active_keys} {plural(q.data.active_keys, ["ключ", "ключа", "ключей"])} API
              </Badge>
            </>
          ) : undefined
        }
      />
      {q.isPending ? (
        <PageSkeleton cards={3} />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <div className="space-y-6">
          <HowItWorks />
          <div className="grid gap-6 lg:grid-cols-2">
            <div className="min-w-0 space-y-6">
              <WebhookCard key={q.data.webhook?.url ?? "none"} data={q.data} onSecret={setWebhookSecret} />
              <VerifyCard />
            </div>
            <div className="min-w-0">
              <ApiKeysCard data={q.data} />
            </div>
          </div>
          <DeliveriesCard counts={q.data.deliveries} />
        </div>
      )}
      <RevealDialog secret={webhookSecret} onClose={() => setWebhookSecret(null)} title="Секрет подписи">
        <p className="text-sm text-ink-muted">
          Проверяйте каждое событие: <span className="font-mono text-ink">HMAC-SHA256(секрет, t + &quot;.&quot; + тело)</span> должно совпасть с <span className="font-mono text-ink">v1</span> из заголовка{" "}
          <span className="font-mono text-ink">FSP-Signature: t=…,v1=…</span>, а метка <span className="font-mono text-ink">t</span> — отличаться от ваших часов не больше чем на 5 минут.
        </p>
      </RevealDialog>
    </div>
  );
}
