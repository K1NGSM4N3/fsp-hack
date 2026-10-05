import { ApiError } from "@/api/client";
import type * as T from "@/api/types";
import { newId } from "./rng";

const EVENT_TYPES: T.IntegrationEventType[] = [
  "invitation.sent",
  "invitation.viewed",
  "invitation.accepted",
  "invitation.declined",
  "invitation.expired",
  "invitation.withdrawn",
  "application.created",
  "application.status_changed",
  "contacts.opened",
  "integration.test",
];
const SCOPES: T.ApiScope[] = ["invitations:read", "candidates:read", "outcomes:write", "needs:write"];

let webhook: T.WebhookView | null = null;
const keys: T.ApiKeyView[] = [];
const deliveries: T.DeliveryView[] = [];

const wait = () => new Promise<void>((resolve) => setTimeout(resolve, 150));
const nowIso = () => new Date().toISOString();

function randomToken(prefix: string): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  const b64 = btoa(String.fromCharCode(...bytes)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  return prefix + b64;
}

function invalid(field: string, message: string): never {
  throw new ApiError({ status: 422, code: "validation_failed", title: "Проверьте введённые данные", errors: { [field]: [message] } });
}

function notFound(): never {
  throw new ApiError({ status: 404, code: "not_found", title: "Не найдено", detail: "Вебхук не настроен" });
}

export async function getIntegrations(): Promise<T.IntegrationsOverview> {
  await wait();
  const count = (s: T.DeliveryStatus) => deliveries.filter((d) => d.status === s).length;
  return {
    webhook,
    deliveries: { pending: count("pending"), delivered: count("delivered"), failed: count("failed") },
    active_keys: keys.filter((k) => !k.revoked_at).length,
    api_base_url: `${window.location.origin}/api/v1/integrations`,
    event_types: EVENT_TYPES,
    scopes: SCOPES,
  };
}

export async function saveWebhook(body: T.WebhookInput): Promise<T.WebhookSaved> {
  await wait();
  if (!/^https:\/\/[^/\s]+/.test(body.url.trim())) invalid("url", "Нужен адрес https://");
  if (webhook) {
    webhook = { ...webhook, url: body.url.trim(), updated_at: nowIso() };
    return { webhook, secret: null };
  }
  const secret = randomToken("whsec_");
  const at = nowIso();
  webhook = { url: body.url.trim(), secret_hint: `${secret.slice(0, 10)}…`, created_at: at, updated_at: at, secret_rotated_at: at, previous_secret_valid_until: null };
  return { webhook, secret };
}

export async function deleteWebhook(): Promise<void> {
  await wait();
  if (!webhook) notFound();
  webhook = null;
}

export async function rotateWebhookSecret(): Promise<T.WebhookSaved> {
  await wait();
  if (!webhook) notFound();
  const secret = randomToken("whsec_");
  webhook = { ...webhook, secret_hint: `${secret.slice(0, 10)}…`, secret_rotated_at: nowIso(), previous_secret_valid_until: new Date(Date.now() + 86_400_000).toISOString() };
  return { webhook, secret };
}

export async function sendTestEvent(): Promise<T.TestDelivery> {
  await wait();
  if (!webhook) notFound();
  const event_id = newId();
  const at = nowIso();
  deliveries.unshift({
    event_id,
    event_type: "integration.test",
    status: "failed",
    created_at: at,
    next_attempt_at: null,
    delivered_at: null,
    last_error: "Демо-режим: запросы наружу не отправляются",
    attempts: [{ at, url: webhook.url, status_code: null, latency_ms: 0, error: "Демо-режим: запросы наружу не отправляются" }],
  });
  return { event_id, delivered: false, status_code: null, latency_ms: 0, error: "Демо-режим: запросы наружу не отправляются" };
}

export async function listDeliveries(limit: number): Promise<T.DeliveryView[]> {
  await wait();
  return deliveries.slice(0, limit);
}

export async function retryDelivery(eventId: T.Uuid): Promise<void> {
  await wait();
  const d = deliveries.find((x) => x.event_id === eventId);
  if (!d) notFound();
  d.status = "pending";
  d.next_attempt_at = nowIso();
}

export async function listApiKeys(): Promise<T.ApiKeyView[]> {
  await wait();
  return [...keys].sort((a, b) => Number(Boolean(a.revoked_at)) - Number(Boolean(b.revoked_at)));
}

export async function createApiKey(body: T.ApiKeyInput): Promise<T.ApiKeyCreated> {
  await wait();
  if (!body.name.trim()) invalid("name", "Назовите ключ, например «Huntflow»");
  if (body.scopes.length === 0) invalid("scopes", "Выберите хотя бы одно право");
  const secret = randomToken("fsp_live_");
  const key: T.ApiKeyView = { id: newId(), name: body.name.trim(), prefix: secret.slice(0, 17), scopes: SCOPES.filter((s) => body.scopes.includes(s)), created_at: nowIso(), last_used_at: null, revoked_at: null };
  keys.unshift(key);
  return { key, secret };
}

export async function revokeApiKey(id: T.Uuid): Promise<void> {
  await wait();
  const k = keys.find((x) => x.id === id && !x.revoked_at);
  if (!k) throw new ApiError({ status: 404, code: "not_found", title: "Не найдено" });
  k.revoked_at = nowIso();
}
