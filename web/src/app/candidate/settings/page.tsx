"use client";

import { Eye, RotateCcw, ShieldCheck, Trash2, Unlink } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useRef, useState } from "react";
import { isApiError } from "@/api/client";
import type { ConsentDocument, ContactAccessEntry, Privacy, SearchStatus, UserConsent } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { FspAchievementList, FspEmpty } from "@/components/domain/Fsp";
import { Alert, Badge, Button, Card, CardHeader, Dialog, EmptyState, ErrorAlert, Field, Input, PageHeader, RadioCards, SkeletonCard, Switch, useToast } from "@/components/ui";
import { ConsentDialog } from "@/features/auth/components/ConsentDialog";
import { DataExportCard } from "@/features/auth/components/DataExportCard";
import { useConsentDocuments, useDeleteAccount, useGrantConsent, useMe, useMyConsents, useRevokeConsent } from "@/features/auth/hooks";
import { useContactAccess, useFspStatus, useFspSync, useFspUnlink, usePrivacy, useProfile, useUpdatePrivacy } from "@/features/candidate/hooks";
import { LinkFspButton, SyncIcon } from "@/features/candidate/components/FspCard";
import { NotificationSettingsCard } from "@/features/notifications/components/NotificationSettingsCard";
import { formatDate, formatDateTime } from "@/lib/format";
import { SEARCH_STATUS_LABEL } from "@/lib/labels";

const STATUS_HINT: Record<SearchStatus, string> = {
  active: "Показываем вас выше и отмечаем значком «Активно ищу»",
  open: "Вы в поиске, но без пометки — подойдёт, если вас устраивает текущая работа",
  closed: "Профиль скрыт из поиска, новые приглашения не приходят",
};

/** Turning visibility on may need consents first: the API names the document, the dialog offers it. */
function PrivacySection() {
  const q = usePrivacy();
  const me = useMe();
  const profile = useProfile();
  const docs = useConsentDocuments();
  const save = useUpdatePrivacy();
  const grant = useGrantConsent();
  const toast = useToast();
  const [gate, setGate] = useState<ConsentDocument | null>(null);
  if (q.isPending) return <SkeletonCard lines={5} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const p = q.data;
  const consentFor = (e: unknown) => (isApiError(e) && e.code === "consent_required" ? (docs.data?.find((d) => d.id === e.documentId) ?? null) : null);
  const update = (patch: Partial<Privacy>) =>
    save.mutate(
      { ...p, ...patch },
      {
        onSuccess: () => toast.success("Сохранено"),
        onError: (e) => {
          const doc = consentFor(e);
          if (doc) setGate(doc);
          else toast.error(isApiError(e) ? e.title : "Не удалось сохранить", isApiError(e) ? (e.detail ?? undefined) : undefined);
        },
      },
    );
  const grantAndShow = async (doc: ConsentDocument) => {
    try {
      await grant.mutateAsync(doc.id);
      await save.mutateAsync({ ...p, visible: true });
      setGate(null);
      toast.success("Профиль снова виден компаниям");
    } catch (e) {
      const next = consentFor(e);
      if (next) setGate(next);
      else toast.error(isApiError(e) ? e.title : "Не удалось сохранить", isApiError(e) ? (e.detail ?? undefined) : undefined);
    }
  };
  const code = me.data?.candidate?.public_code;
  const category = me.data?.candidate?.category;
  const age = me.data?.candidate?.age;
  const waitsForGuardian = age?.bracket === "minor" && age.guardian !== "granted";
  return (
    <>
      <Card id="search-status" className="scroll-mt-24">
        <CardHeader title="Статус поиска" description="Компании видят его в карточке" />
        <RadioCards<SearchStatus>
          name="search_status"
          columns={3}
          value={p.search_status}
          onChange={(s) => update({ search_status: s })}
          options={(["active", "open", "closed"] as SearchStatus[]).map((s) => ({ value: s, title: SEARCH_STATUS_LABEL[s], description: STATUS_HINT[s] }))}
        />
      </Card>

      <Card id="visibility" className="scroll-mt-24">
        <CardHeader title="Что видят компании" description="До вашего согласия компании видят анонимную карточку. Контакты не видны никогда, пока вы не примете приглашение или не откликнетесь." />
        <div className="divide-y divide-line">
          <Switch
            className="py-4 first:pt-0"
            checked={p.visible}
            onChange={(v) => update({ visible: v })}
            disabled={save.isPending}
            label="Показывать профиль в поиске"
            description={
              waitsForGuardian ? "Компании увидят профиль, когда законный представитель подтвердит согласие" : "Если выключить, вас не найдут в подборках и поиске по базе"
            }
          />
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
              {age?.bracket === "minor" && <Badge tone="neutral">14–17 лет</Badge>}
              {(!p.visible || waitsForGuardian) && <Badge tone="muted">Скрыт из поиска</Badge>}
            </div>
          </div>
        )}
      </Card>

      <ConsentDialog
        doc={gate}
        onClose={() => setGate(null)}
        onAccept={() => gate && void grantAndShow(gate)}
        loading={grant.isPending || save.isPending}
        title="Чтобы показать профиль, нужно согласие"
        lead={
          gate?.kind === "pd_publication"
            ? "Вы отозвали согласие на публикацию, поэтому профиль скрыт. Дайте его снова — и анонимная карточка вернётся в поиск и подборки."
            : "Без согласия на обработку данных мы не можем показывать ваш профиль компаниям."
        }
        acceptLabel="Дать согласие и показать профиль"
      />
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
      <CardHeader title="ФСП ID" description="Разряды и места на соревнованиях подтягиваются из реестра ФСП. Членство в Федерации для этого не нужно и в подборе не учитывается." />
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

const KIND_ORDER = ["pd_processing", "pd_publication", "terms", "proctoring_video", "guardian_consent"];

const REVOKE_TEXT: Record<string, string> = {
  pd_processing: "Профиль сразу скроется из поиска, и мы перестанем подбирать вам предложения. Чтобы удалить данные полностью, удалите аккаунт.",
  pd_publication: "Профиль сразу скроется из поиска компаний. Уже выданные контакты останутся у компаний, которым вы их открыли. Вернуть согласие можно здесь же.",
  proctoring_video: "Все видеозаписи ваших тестов удалятся сразу. Отметки «с видеозаписью» у результатов останутся.",
};

/** One row per document: whether it is in force, and the way to revoke or give it again. */
function ConsentsSection() {
  const mine = useMyConsents();
  const docs = useConsentDocuments();
  const revoke = useRevokeConsent();
  const grant = useGrantConsent();
  const toast = useToast();
  const [target, setTarget] = useState<UserConsent | null>(null);
  const [offer, setOffer] = useState<ConsentDocument | null>(null);
  const pending = mine.isPending || docs.isPending;
  const error = mine.error ?? docs.error;
  const rows =
    mine.data && docs.data
      ? docs.data
          .filter((d) => d.required_for.includes("candidate") || mine.data.some((c) => c.kind === d.kind))
          .sort((a, b) => KIND_ORDER.indexOf(a.kind) - KIND_ORDER.indexOf(b.kind))
          .map((doc) => {
            const history = mine.data.filter((c) => c.kind === doc.kind);
            return { doc, active: history.find((c) => !c.revoked_at) ?? null, last: history[0] ?? null };
          })
      : [];
  return (
    <Card id="consents" className="scroll-mt-24">
      <CardHeader
        title="Согласия"
        description={
          <>
            Обработка персональных данных по 152-ФЗ. Что и зачем мы храним —{" "}
            <Link href="/privacy" className="link-inline">
              в политике обработки данных
            </Link>
            .
          </>
        }
      />
      {pending ? (
        <SkeletonCard lines={2} className="border-0 p-0" />
      ) : error ? (
        <ErrorAlert error={error} onRetry={() => void Promise.all([mine.refetch(), docs.refetch()])} />
      ) : (
        <>
          <ul className="divide-y divide-line">
            {rows.map(({ doc, active, last }) => (
              <li key={doc.kind} className="flex flex-col gap-2 py-3 first:pt-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between">
                <div className="min-w-0">
                  <p className="text-sm font-medium text-ink">{doc.title}</p>
                  <p className="text-xs text-ink-subtle">
                    {active
                      ? `Действует · редакция ${active.version} · выдано ${formatDate(active.granted_at)}`
                      : last?.revoked_at
                        ? `Отозвано ${formatDate(last.revoked_at)}`
                        : "Не выдано"}
                  </p>
                </div>
                {doc.kind === "guardian_consent" ? (
                  <p className="text-xs text-ink-subtle sm:max-w-56 sm:text-right">Даёт и отзывает законный представитель — по ссылкам из письма</p>
                ) : active ? (
                  <Button variant="ghost" size="sm" className="self-start sm:self-auto" onClick={() => setTarget(active)}>
                    Отозвать
                  </Button>
                ) : (
                  <Button variant="soft" size="sm" className="self-start sm:self-auto" icon={<RotateCcw className="size-4" aria-hidden />} onClick={() => setOffer(doc)}>
                    {last ? "Дать согласие снова" : "Дать согласие"}
                  </Button>
                )}
              </li>
            ))}
          </ul>
          {(mine.data?.length ?? 0) > 0 && (
            <details className="group mt-4 border-t border-line pt-3 text-sm">
              <summary className="tap cursor-pointer text-ink-muted marker:text-ink-faint hover:text-ink">История согласий</summary>
              <ul className="mt-2 space-y-1.5 text-xs text-ink-subtle">
                {mine.data?.map((c) => (
                  <li key={c.id}>
                    {c.title}, редакция {c.version}: выдано {formatDateTime(c.granted_at)}
                    {c.revoked_at ? `, отозвано ${formatDateTime(c.revoked_at)}` : ""}
                  </li>
                ))}
              </ul>
            </details>
          )}
        </>
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
        <p className="text-sm text-ink-muted">{(target && REVOKE_TEXT[target.kind]) ?? "Согласие перестанет действовать с этого момента. Дать его снова можно здесь же."}</p>
      </Dialog>
      <ConsentDialog
        doc={offer}
        onClose={() => setOffer(null)}
        loading={grant.isPending}
        lead={offer?.kind === "pd_publication" ? "Профиль вернётся в поиск, когда вы включите «Показывать профиль в поиске» выше." : undefined}
        onAccept={() =>
          offer &&
          grant.mutate(offer.id, {
            onSuccess: () => {
              setOffer(null);
              toast.success("Согласие дано");
            },
            onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось сохранить", isApiError(e) ? (e.fieldError("document_id") ?? undefined) : undefined),
          })
        }
      />
    </Card>
  );
}

const BASIS_LABEL: Record<ContactAccessEntry["basis"], string> = {
  invitation: "Вы приняли приглашение",
  application: "Вы откликнулись на вакансию",
};

function ContactAccessSection() {
  const q = useContactAccess();
  return (
    <Card id="contact-access" className="scroll-mt-24">
      <CardHeader title="Кто открывал ваши контакты" description="Компания получает телефон, почту и имя только после того, как вы примете её приглашение или сами откликнетесь. Каждое открытие записывается сюда." />
      {q.isPending ? (
        <SkeletonCard lines={2} className="border-0 p-0" />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState compact icon={<ShieldCheck className="size-5" aria-hidden />} title="Пока никто" description="Здесь появится каждая компания, которой вы откроете контакты: принятым приглашением или своим откликом." />
      ) : (
        <ul className="divide-y divide-line">
          {q.data.map((a, i) => (
            <li key={`${a.company_id}-${a.accessed_at}-${i}`} className="flex flex-col gap-1 py-3 first:pt-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
              <div className="min-w-0">
                <p className="truncate text-sm font-medium text-ink">{a.company_name ?? "Компания удалила аккаунт"}</p>
                <p className="text-xs text-ink-subtle">
                  {BASIS_LABEL[a.basis]}
                  {a.channel !== "cabinet" && " · передано в систему подбора компании"}
                </p>
              </div>
              <time dateTime={a.accessed_at} className="shrink-0 text-xs text-ink-subtle tabular">
                {formatDateTime(a.accessed_at)}
              </time>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}

function DangerZone() {
  const router = useRouter();
  const del = useDeleteAccount();
  const [open, setOpen] = useState(false);
  const [confirmText, setConfirmText] = useState("");
  return (
    <Card id="delete" className="scroll-mt-24 border-accent-100">
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
          <Field label="Введите «удалить», чтобы подтвердить" hint="Так мы убедимся, что это не случайное нажатие">
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

const SECTIONS: { id: string; label: string; group: string }[] = [
  { id: "search-status", label: "Статус поиска", group: "Поиск" },
  { id: "visibility", label: "Что видят компании", group: "Поиск" },
  { id: "fsp", label: "ФСП ID", group: "Профиль" },
  { id: "consents", label: "Согласия", group: "Данные" },
  { id: "contact-access", label: "Доступ к контактам", group: "Данные" },
  { id: "notifications", label: "Уведомления", group: "Данные" },
  { id: "data", label: "Выгрузка данных", group: "Данные" },
  { id: "delete", label: "Удаление аккаунта", group: "Аккаунт" },
];

/** Section index with the section in view highlighted; a horizontal strip on phones. */
function SectionNav() {
  const [current, setCurrent] = useState(SECTIONS[0]?.id ?? "");
  useEffect(() => {
    // Measured on each scroll frame rather than observed once: sections replace their skeletons after mount.
    let frame = 0;
    const update = () => {
      frame = 0;
      let active = SECTIONS[0]?.id ?? "";
      for (const s of SECTIONS) {
        const el = document.getElementById(s.id);
        if (el && el.getBoundingClientRect().top <= 140) active = s.id;
      }
      if (window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 4) active = SECTIONS.at(-1)?.id ?? active;
      setCurrent(active);
    };
    const schedule = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };
    schedule();
    window.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
    };
  }, []);
  const groups = SECTIONS.reduce<{ group: string; items: typeof SECTIONS }[]>((acc, s) => {
    const last = acc.at(-1);
    if (last?.group === s.group) last.items.push(s);
    else acc.push({ group: s.group, items: [s] });
    return acc;
  }, []);
  const link = (s: (typeof SECTIONS)[number], className: string) => (
    <a href={`#${s.id}`} aria-current={current === s.id ? "location" : undefined} onClick={() => setCurrent(s.id)} className={className}>
      {s.label}
    </a>
  );
  return (
    <nav aria-label="Разделы настроек" className="min-w-0 lg:sticky lg:top-20 lg:self-start">
      <ul className="scrollbar-none -mx-4 flex gap-2 overflow-x-auto px-4 pb-1 lg:hidden">
        {SECTIONS.map((s) => (
          <li key={s.id} className="shrink-0">
            {link(
              s,
              "inline-flex h-10 items-center rounded-full border px-3.5 text-sm whitespace-nowrap transition-colors aria-[current=location]:border-primary aria-[current=location]:bg-primary-50 aria-[current=location]:text-primary-strong border-line bg-white text-ink-muted hover:text-ink",
            )}
          </li>
        ))}
      </ul>
      <div className="hidden space-y-5 lg:block">
        {groups.map((g) => (
          <div key={g.group}>
            <p className="eyebrow mb-1.5 px-3 text-ink-subtle">{g.group}</p>
            <ul className="space-y-0.5">
              {g.items.map((s) => (
                <li key={s.id}>
                  {link(
                    s,
                    "relative flex min-h-10 items-center rounded-control px-3 text-sm text-ink-muted transition-colors hover:bg-white hover:text-ink aria-[current=location]:bg-white aria-[current=location]:font-medium aria-[current=location]:text-ink aria-[current=location]:shadow-card",
                  )}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </nav>
  );
}

export default function SettingsPage() {
  return (
    <div className="max-w-6xl">
      <Suspense>
        <FspCallbackToast />
      </Suspense>
      <PageHeader eyebrow="Аккаунт" title="Настройки" description="Статус поиска и видимость профиля, ФСП ID, согласия на обработку данных и всё, что платформа о вас хранит." />
      <div className="grid grid-cols-[minmax(0,1fr)] gap-6 lg:grid-cols-[13rem_minmax(0,1fr)] lg:gap-10">
        <SectionNav />
        <div className="min-w-0 max-w-3xl space-y-6">
          <PrivacySection />
          <FspSection />
          <ConsentsSection />
          <ContactAccessSection />
          <div id="notifications" className="scroll-mt-24">
            <NotificationSettingsCard />
          </div>
          <div id="data" className="scroll-mt-24">
            <DataExportCard description="JSON-файл со всем, что платформа хранит о вас: профиль, согласия, тесты и их итоги, история категории, приглашения, отклики, кто открывал контакты, сводки видеозаписи. Сами видеозаписи в файл не входят — их можно посмотреть в истории тестов." />
          </div>
          <DangerZone />
        </div>
      </div>
    </div>
  );
}
