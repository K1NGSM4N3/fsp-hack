"use client";

import { Eye, Trash2, Unlink } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useRef, useState } from "react";
import { isApiError } from "@/api/client";
import type { Privacy, SearchStatus, UserConsent } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { FspAchievementList, FspEmpty } from "@/components/domain/Fsp";
import { Alert, Badge, Button, Card, CardHeader, Dialog, ErrorAlert, Field, Input, PageHeader, RadioCards, SkeletonCard, Switch, useToast } from "@/components/ui";
import { useDeleteAccount, useMe, useMyConsents, useRevokeConsent } from "@/features/auth/hooks";
import { useFspStatus, useFspSync, useFspUnlink, usePrivacy, useProfile, useUpdatePrivacy } from "@/features/candidate/hooks";
import { LinkFspButton, SyncIcon } from "@/features/candidate/components/FspCard";
import { formatDate, formatDateTime } from "@/lib/format";
import { SEARCH_STATUS_LABEL } from "@/lib/labels";

const STATUS_HINT: Record<SearchStatus, string> = {
  active: "Показываем вас выше и отмечаем значком «Активно ищу»",
  open: "Вы в поиске, но без пометки — подойдёт, если вас устраивает текущая работа",
  closed: "Профиль скрыт из поиска, новые приглашения не приходят",
};

function PrivacySection() {
  const q = usePrivacy();
  const me = useMe();
  const profile = useProfile();
  const save = useUpdatePrivacy();
  const toast = useToast();
  if (q.isPending) return <SkeletonCard lines={5} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const p = q.data;
  const update = (patch: Partial<Privacy>) =>
    save.mutate(
      { ...p, ...patch },
      {
        onSuccess: () => toast.success("Сохранено"),
        onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось сохранить", isApiError(e) ? (e.detail ?? undefined) : undefined),
      },
    );
  const code = me.data?.candidate?.public_code;
  const category = me.data?.candidate?.category;
  return (
    <>
      <Card>
        <CardHeader title="Статус поиска" description="Работодатели видят его в карточке" />
        <RadioCards<SearchStatus>
          name="search_status"
          columns={3}
          value={p.search_status}
          onChange={(s) => update({ search_status: s })}
          options={(["active", "open", "closed"] as SearchStatus[]).map((s) => ({ value: s, title: SEARCH_STATUS_LABEL[s], description: STATUS_HINT[s] }))}
        />
      </Card>

      <Card>
        <CardHeader title="Что видят работодатели" description="До вашего согласия компании видят анонимную карточку. Контакты не видны никогда, пока вы не примете приглашение или не откликнетесь." />
        <div className="divide-y divide-line">
          <Switch className="py-4 first:pt-0" checked={p.visible} onChange={(v) => update({ visible: v })} disabled={save.isPending} label="Показывать профиль в поиске" description="Если выключить, вас не найдут в подборках и поиске по базе" />
          <Switch className="py-4" checked={p.show_name} onChange={(v) => update({ show_name: v })} disabled={save.isPending} label="Показывать имя" description="Вместо анонимного кода в карточке будет ваше имя" />
          <Switch className="py-4" checked={p.show_fsp} onChange={(v) => update({ show_fsp: v })} disabled={save.isPending} label="Показывать достижения ФСП" description="Достижения поднимают вас внутри категории" />
          <Switch className="py-4 last:pb-0" checked={p.show_experience} onChange={(v) => update({ show_experience: v })} disabled={save.isPending} label="Показывать названия компаний в опыте" description="Иначе — только должности и описание задач" />
        </div>
        {code && (
          <div className="mt-5 rounded-control border border-dashed border-line-strong p-4">
            <p className="mb-2 flex items-center gap-1.5 text-xs text-ink-subtle">
              <Eye className="size-3.5" aria-hidden /> Так выглядит заголовок вашей карточки
            </p>
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-mono font-semibold text-ink">{p.show_name ? profile.data?.full_name || "Ваше имя" : `Кандидат ${code}`}</span>
              {category && <CategoryBadge category={category} size="sm" />}
              {p.search_status === "active" && <Badge tone="success">Активно ищу работу</Badge>}
              {!p.visible && <Badge tone="muted">Скрыт из поиска</Badge>}
            </div>
          </div>
        )}
      </Card>
    </>
  );
}

function FspSection() {
  const q = useFspStatus();
  const sync = useFspSync();
  const unlink = useFspUnlink();
  const toast = useToast();
  const [confirm, setConfirm] = useState(false);
  return (
    <Card id="fsp" className="scroll-mt-24">
      <CardHeader title="ФСП ID" description="Связь с профилем участника Федерации спортивного программирования. Достижения синхронизируются из реестра ФСП." />
      {q.isPending ? (
        <SkeletonCard lines={2} className="border-0 p-0" />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : !q.data.linked ? (
        <div className="space-y-4">
          <FspEmpty text="ФСП ID не привязан. Без него профиль участвует в подборе на общих основаниях." />
          <LinkFspButton />
        </div>
      ) : (
        <div className="space-y-5">
          <dl className="grid gap-4 text-sm sm:grid-cols-3">
            <div>
              <dt className="text-ink-subtle">ФСП ID</dt>
              <dd className="font-mono font-medium text-ink">{q.data.fsp_id}</dd>
            </div>
            <div>
              <dt className="text-ink-subtle">Привязан</dt>
              <dd className="text-ink">{formatDate(q.data.linked_at)}</dd>
            </div>
            <div>
              <dt className="text-ink-subtle">Синхронизация</dt>
              <dd className="text-ink">{formatDateTime(q.data.last_sync_at)}</dd>
            </div>
          </dl>
          {q.data.achievements.length ? <FspAchievementList achievements={q.data.achievements} /> : <FspEmpty text="В реестре пока нет достижений — это не влияет на позицию негативно" />}
          <div className="flex flex-wrap gap-2 border-t border-line pt-4">
            <Button
              variant="secondary"
              size="sm"
              icon={<SyncIcon spinning={sync.isPending} />}
              disabled={sync.isPending}
              onClick={() => sync.mutate(undefined, { onSuccess: () => toast.success("Достижения обновлены"), onError: (e) => toast.error(isApiError(e) ? e.title : "Реестр ФСП недоступен") })}
            >
              Обновить из реестра
            </Button>
            <Button variant="ghost" size="sm" icon={<Unlink className="size-4" aria-hidden />} onClick={() => setConfirm(true)}>
              Отвязать
            </Button>
          </div>
        </div>
      )}
      <Dialog
        open={confirm}
        onClose={() => setConfirm(false)}
        size="sm"
        title="Отвязать ФСП ID?"
        footer={
          <>
            <Button variant="ghost" onClick={() => setConfirm(false)}>
              Отмена
            </Button>
            <Button
              variant="danger"
              loading={unlink.isPending}
              onClick={() =>
                unlink.mutate(undefined, {
                  onSuccess: () => {
                    setConfirm(false);
                    toast.success("ФСП ID отвязан");
                  },
                })
              }
            >
              Отвязать
            </Button>
          </>
        }
      >
        <p className="text-sm text-ink-muted">Снимок достижений будет удалён, а подтверждения навыков через ФСП — сняты. Привязать заново можно в любой момент.</p>
      </Dialog>
    </Card>
  );
}

function ConsentsSection() {
  const q = useMyConsents();
  const revoke = useRevokeConsent();
  const toast = useToast();
  const [target, setTarget] = useState<UserConsent | null>(null);
  return (
    <Card>
      <CardHeader title="Согласия" description="Обработка персональных данных по 152-ФЗ" />
      {q.isPending ? (
        <SkeletonCard lines={2} className="border-0 p-0" />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <ul className="divide-y divide-line">
          {q.data.map((c) => (
            <li key={c.id} className="flex flex-col gap-2 py-3 first:pt-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between">
              <div className="min-w-0">
                <p className="text-sm font-medium text-ink">{c.title}</p>
                <p className="text-xs text-ink-subtle">
                  Версия {c.version} · выдано {formatDate(c.granted_at)}
                  {c.revoked_at ? ` · отозвано ${formatDate(c.revoked_at)}` : ""}
                </p>
              </div>
              {c.revoked_at ? (
                <Badge tone="muted">Отозвано</Badge>
              ) : (
                <Button variant="ghost" size="sm" onClick={() => setTarget(c)}>
                  Отозвать
                </Button>
              )}
            </li>
          ))}
        </ul>
      )}
      <Dialog
        open={target !== null}
        onClose={() => setTarget(null)}
        size="sm"
        title="Отозвать согласие?"
        footer={
          <>
            <Button variant="ghost" onClick={() => setTarget(null)}>
              Отмена
            </Button>
            <Button
              variant="danger"
              loading={revoke.isPending}
              onClick={() =>
                target &&
                revoke.mutate(target.document_id, {
                  onSuccess: () => {
                    setTarget(null);
                    toast.success("Согласие отозвано");
                  },
                  onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось отозвать"),
                })
              }
            >
              Отозвать
            </Button>
          </>
        }
      >
        <p className="text-sm text-ink-muted">
          {target?.kind === "publication"
            ? "Профиль сразу скроется из поиска работодателей. Уже выданные контакты останутся у компаний, которым вы их открыли."
            : "Без согласия на обработку данных мы не сможем подбирать вам предложения. Чтобы удалить данные полностью, удалите аккаунт."}
        </p>
      </Dialog>
    </Card>
  );
}

function DangerZone() {
  const router = useRouter();
  const del = useDeleteAccount();
  const [open, setOpen] = useState(false);
  const [confirmText, setConfirmText] = useState("");
  return (
    <Card className="border-accent-100">
      <CardHeader title="Удаление аккаунта" description="Удалим профиль, результаты тестов и персональные данные. Действие необратимо." />
      <Button variant="danger" icon={<Trash2 className="size-4" aria-hidden />} onClick={() => setOpen(true)}>
        Удалить аккаунт
      </Button>
      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        size="sm"
        title="Удалить аккаунт навсегда?"
        footer={
          <>
            <Button variant="ghost" onClick={() => setOpen(false)}>
              Отмена
            </Button>
            <Button variant="danger" disabled={confirmText.trim().toLowerCase() !== "удалить"} loading={del.isPending} onClick={() => del.mutate(undefined, { onSuccess: () => router.replace("/") })}>
              Удалить
            </Button>
          </>
        }
      >
        <div className="space-y-4">
          <p className="text-sm text-ink-muted">Категория, история тестов, приглашения и отклики будут удалены. Связи с компаниями обезличатся.</p>
          <Field label="Введите «удалить», чтобы подтвердить">
            <Input value={confirmText} onChange={(e) => setConfirmText(e.target.value)} autoComplete="off" />
          </Field>
          {del.isError && <Alert tone="danger" title="Не удалось удалить аккаунт" />}
        </div>
      </Dialog>
    </Card>
  );
}

function FspCallbackToast() {
  const params = useSearchParams();
  const toast = useToast();
  const shown = useRef(false);
  const outcome = params.get("fsp");
  useEffect(() => {
    if (!outcome || shown.current) return;
    shown.current = true;
    if (outcome === "linked") toast.success("ФСП ID привязан", "Достижения загружены из реестра");
    else toast.error("Не удалось привязать ФСП ID", "Попробуйте ещё раз");
  }, [outcome, toast]);
  return null;
}

export default function SettingsPage() {
  return (
    <div>
      <Suspense>
        <FspCallbackToast />
      </Suspense>
      <PageHeader title="Настройки" description="Приватность, статус поиска, ФСП ID и согласия." />
      <div className="max-w-3xl space-y-6">
        <PrivacySection />
        <FspSection />
        <ConsentsSection />
        <DangerZone />
      </div>
    </div>
  );
}
