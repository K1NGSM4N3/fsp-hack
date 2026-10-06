import { ApiError } from "@/api/client";
import type * as T from "@/api/types";
import { newId } from "./rng";
import { getDb } from "./store";

type Audience = "candidate" | "employer";

const KINDS: Record<Audience, { kind: T.NotificationKind; title: string; description: string }[]> = {
  candidate: [
    { kind: "invitation_received", title: "Новые приглашения", description: "Компания, должность, вилка и срок ответа" },
    { kind: "invitation_expiring", title: "Напоминание о сроке ответа", description: "За 48 часов до конца срока, если вы ещё не ответили — один раз" },
    { kind: "invitation_withdrawn", title: "Компания отозвала приглашение", description: "Приглашение больше не ждёт ответа" },
    { kind: "hiring_stage", title: "Этапы найма", description: "Компания отметила собеседование, оффер, найм или отказ" },
    { kind: "application_status", title: "Ответы на отклики", description: "Приглашение или отказ по отклику; «просмотрено» — только в кабинете" },
    { kind: "assessment_result", title: "Итоги теста", description: "Присвоена категория или тест не пройден" },
    { kind: "guardian_decision", title: "Решение законного представителя", description: "Согласие дано, не дано или отозвано — для участников 14–17 лет" },
  ],
  employer: [
    { kind: "invitation_accepted", title: "Кандидат принял приглашение", description: "Контакты кандидата откроются в кабинете; в письме их нет" },
    { kind: "invitation_declined", title: "Кандидат отклонил приглашение", description: "С причиной отказа" },
    { kind: "application_received", title: "Новые отклики", description: "Отклик на опубликованную вакансию" },
    { kind: "webhook_failing", title: "Сбои доставки в ATS", description: "Вебхук в вашу ATS не принимает события" },
    { kind: "trust_changed", title: "Уровень доверия и квота", description: "Изменилась квота приглашений за 7 дней" },
  ],
};

const HOUR = 3_600_000;
const wait = () => new Promise<void>((resolve) => setTimeout(resolve, 120));
const feeds = new Map<string, T.NotificationView[]>();
const emailOff = new Map<string, Set<T.NotificationKind>>();

function audience(): { id: string; role: Audience } {
  const db = getDb();
  const user = db.session ? db.users.find((u) => u.id === db.session?.user_id) : undefined;
  if (!user) throw new ApiError({ status: 401, code: "unauthorized", title: "Требуется вход" });
  return { id: user.id, role: user.role };
}

function feed(): T.NotificationView[] {
  const { id, role } = audience();
  let items = feeds.get(id);
  if (!items) {
    const at = (h: number) => new Date(Date.now() - h * HOUR).toISOString();
    items =
      role === "candidate"
        ? [
            { id: newId(), kind: "invitation_received", title: "Пиксельная Мануфактура приглашает: Backend-разработчик", body: "Вилка 250 000 – 320 000 ₽, удалённо. Ответить можно до 20 октября.", link: "/candidate/invitations", created_at: at(2), read_at: null },
            { id: newId(), kind: "invitation_withdrawn", title: "Компания ОблакоСтрой отозвала приглашение", body: "«Go-разработчик во внутреннюю платформу» больше не ждёт ответа.", link: "/candidate/invitations", created_at: at(26), read_at: at(20) },
          ]
        : [
            { id: newId(), kind: "application_received", title: "Новый отклик на «Go-разработчик»", body: "Кандидат K-7F3A2C. Контакты и сопроводительное письмо — в кабинете.", link: "/employer/vacancies", created_at: at(1), read_at: null },
            { id: newId(), kind: "invitation_declined", title: "Кандидат K-2B9D41 отклонил приглашение", body: "«Backend-разработчик в команду платежей». Причина: зарплата.", link: "/employer/invitations", created_at: at(3), read_at: at(2) },
          ];
    feeds.set(id, items);
  }
  return items;
}

export async function listNotifications(query: T.NotificationQuery = {}): Promise<T.NotificationPage> {
  await wait();
  const all = feed().filter((n) => !query.unread || !n.read_at);
  const start = query.cursor ? all.findIndex((n) => n.id === query.cursor) + 1 : 0;
  const limit = query.limit ?? 20;
  const items = all.slice(start, start + limit);
  const more = start + limit < all.length;
  return { items, next_cursor: more ? (items.at(-1)?.id ?? null) : null };
}

export async function getUnreadCount(): Promise<T.UnreadCount> {
  return { count: feed().filter((n) => !n.read_at).length };
}

export async function markNotificationsRead(body: T.MarkRead): Promise<T.UnreadCount> {
  await wait();
  const now = new Date().toISOString();
  for (const n of feed()) if (!n.read_at && (body.all || body.ids?.includes(n.id))) n.read_at = now;
  return getUnreadCount();
}

function settings(): T.NotificationSettings {
  const { id, role } = audience();
  const off = emailOff.get(id) ?? new Set();
  return { items: KINDS[role].map((k) => ({ ...k, in_app: true, email: !off.has(k.kind) })) };
}

export async function getNotificationSettings(): Promise<T.NotificationSettings> {
  await wait();
  return settings();
}

export async function updateNotificationSettings(body: T.NotificationSettingsInput): Promise<T.NotificationSettings> {
  await wait();
  const { id } = audience();
  const off = emailOff.get(id) ?? new Set<T.NotificationKind>();
  for (const t of body.email) {
    if (t.enabled) off.delete(t.kind);
    else off.add(t.kind);
  }
  emailOff.set(id, off);
  return settings();
}

export async function unsubscribe(token: string): Promise<T.Unsubscribed> {
  await wait();
  if (!token) throw new ApiError({ status: 400, code: "invalid_token", title: "Ссылка недействительна" });
  return { kind: null, title: "все письма", settings_path: "/candidate/settings#notifications" };
}
