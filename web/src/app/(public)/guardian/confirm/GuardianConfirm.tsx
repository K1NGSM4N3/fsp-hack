"use client";

import { Building2, CheckCircle2, Clock3, Database, EyeOff, Lock, Undo2, Video, XCircle } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useState, type ReactNode } from "react";
import { isApiError } from "@/api/client";
import type { GuardianAction, GuardianConsentView } from "@/api/types";
import { Button, Dialog, Spinner, useToast } from "@/components/ui";
import { errorText } from "@/components/ui/QueryState";
import { useAnswerGuardianConsent, useGuardianConsent } from "@/features/candidate/hooks";
import { formatDate } from "@/lib/format";

const FACTS: { icon: typeof Building2; title: string; text: string }[] = [
  {
    icon: Building2,
    title: "Кто мы",
    text: "«ФСП Таланты» — платформа подбора ИТ-специалистов Федерации спортивного программирования России. Участник проходит тест по своему направлению, а компании сами предлагают работу или стажировку и сразу называют условия.",
  },
  {
    icon: Database,
    title: "Какие данные",
    text: "Почта и дата рождения, а дальше — то, что участник укажет сам: имя, контакты, город, навыки, опыт. Плюс результаты тестов, достижения ФСП и приглашения от компаний.",
  },
  {
    icon: EyeOff,
    title: "Что видят компании",
    text: "Анонимную карточку: код вместо имени, категорию и результат теста, навыки и отметку «14–17 лет». Ни имени, ни телефона, ни почты.",
  },
  {
    icon: Lock,
    title: "Когда компания получит контакты",
    text: "Только если участник сам примет её приглашение или сам откликнется на вакансию. Каждое открытие контактов записывается, участник видит этот журнал.",
  },
  {
    icon: Video,
    title: "Видео",
    text: "Камера во время теста включается, только если участник отдельно на это согласится. Без камеры тест проходится так же.",
  },
  {
    icon: Undo2,
    title: "Как отозвать",
    text: "В письме есть вторая ссылка — для отзыва согласия: по ней профиль сразу скроется из поиска. В день 18-летия согласие перестаёт быть нужным, а ваши имя и почту мы удалим через 30 дней после него.",
  },
];

const CONFIRM_TEXT: Record<"decline" | "revoke", { title: string; text: string; button: string }> = {
  decline: { title: "Не давать согласие?", text: "Профиль участника не покажут компаниям. Участник увидит ваш ответ в личном кабинете и сможет попросить вас ещё раз.", button: "Не согласен" },
  revoke: { title: "Отозвать согласие?", text: "Профиль участника сразу скроется из поиска и подборок. Компании, которым участник уже открыл контакты, их сохранят.", button: "Отозвать согласие" },
};

function Outcome({ v }: { v: GuardianConsentView }) {
  const box = (tone: "success" | "muted" | "warning", icon: ReactNode, title: string, text: ReactNode) => (
    <div
      role="status"
      className={
        tone === "success"
          ? "flex gap-3 rounded-card border border-success-200 bg-success-50 p-4 sm:p-5"
          : tone === "warning"
            ? "flex gap-3 rounded-card border border-warning-200 bg-warning-50 p-4 sm:p-5"
            : "flex gap-3 rounded-card border border-line bg-canvas p-4 sm:p-5"
      }
    >
      <span className="mt-0.5 shrink-0">{icon}</span>
      <div className="min-w-0 text-sm">
        <p className="font-semibold text-ink">{title}</p>
        <p className="mt-1 text-ink-muted">{text}</p>
      </div>
    </div>
  );
  if (v.participant_adult) {
    return box(
      "muted",
      <CheckCircle2 className="size-5 text-ink-subtle" aria-hidden />,
      "Участнику уже исполнилось 18",
      "Ваше согласие больше не нужно: участник решает сам. Ничего делать не требуется.",
    );
  }
  switch (v.state) {
    case "granted":
      return box(
        "success",
        <CheckCircle2 className="size-5 text-success" aria-hidden />,
        `Согласие подтверждено ${formatDate(v.answered_at)}`,
        v.link === "revoke" ? "Если передумали, отзовите его кнопкой ниже — профиль сразу скроется из поиска." : "Спасибо! Если передумаете, откройте вторую ссылку из письма — «отозвать согласие».",
      );
    case "declined":
      return box(
        "muted",
        <XCircle className="size-5 text-ink-subtle" aria-hidden />,
        `Вы не дали согласие ${formatDate(v.answered_at)}`,
        "Профиль участника не показывается компаниям. Если участник попросит ещё раз, придёт новое письмо.",
      );
    case "revoked":
      return box(
        "muted",
        <XCircle className="size-5 text-ink-subtle" aria-hidden />,
        `Согласие отозвано ${formatDate(v.revoked_at)}`,
        "Профиль участника скрыт из поиска и подборок. Если участник попросит ещё раз, придёт новое письмо.",
      );
    case "expired":
      return box(
        "warning",
        <Clock3 className="size-5 text-warning" aria-hidden />,
        "Ссылка устарела",
        v.actions.length
          ? "Подтвердить по ней уже нельзя, но отказаться можно. Чтобы дать согласие, попросите участника отправить письмо ещё раз."
          : "Попросите участника отправить письмо ещё раз из личного кабинета — придёт новая ссылка.",
      );
    default:
      return null;
  }
}

export function GuardianConfirm() {
  const token = useSearchParams().get("token");
  const q = useGuardianConsent(token);
  const answer = useAnswerGuardianConsent(token ?? "");
  const toast = useToast();
  const [confirming, setConfirming] = useState<"decline" | "revoke" | null>(null);

  if (!token || q.isError) {
    const err = q.isError ? errorText(q.error) : null;
    return (
      <div className="mx-auto max-w-2xl">
        <XCircle className="size-10 text-accent-strong" aria-hidden />
        <h1 className="display-3 mt-4 text-balance text-ink">{err?.title ?? "В ссылке нет токена"}</h1>
        <p className="mt-4 text-lg leading-relaxed text-ink-muted">
          Откройте ссылку из последнего письма целиком: если участник отправил письмо повторно, прежние ссылки перестают работать. Если вы не знаете, о ком речь, ничего делать не нужно — без вашего
          согласия профиль не покажут компаниям.
        </p>
      </div>
    );
  }
  if (q.isPending) {
    return (
      <div className="flex items-center gap-3 text-ink-muted" role="status">
        <Spinner />
        Загружаем согласие…
      </div>
    );
  }

  const v = q.data;
  const act = (action: GuardianAction) =>
    answer.mutate(action, {
      onSuccess: (next) => {
        setConfirming(null);
        requestAnimationFrame(() => document.getElementById("answer")?.scrollIntoView({ behavior: "smooth", block: "start" }));
        if (next.state === "granted") toast.success("Согласие подтверждено", "Профиль участника появится в поиске");
        else toast.success(action === "revoke" ? "Согласие отозвано" : "Ответ сохранён", "Профиль участника скрыт от компаний");
      },
      onError: (e) => {
        setConfirming(null);
        toast.error(isApiError(e) ? e.title : "Не удалось сохранить ответ", isApiError(e) ? (e.detail ?? undefined) : undefined);
        void q.refetch();
      },
    });
  const can = (a: GuardianAction) => v.actions.includes(a);

  return (
    <article className="mx-auto max-w-3xl">
      <p className="eyebrow text-primary">Для родителей и законных представителей</p>
      <h1 className="display-3 mt-4 text-balance text-ink">Согласие на участие в «ФСП Таланты»</h1>
      <p className="mt-5 text-lg leading-relaxed text-ink-muted">
        Аккаунт <span className="font-medium text-ink">{v.participant}</span> указал вас{v.guardian_name ? <> — {v.guardian_name} —</> : null} как законного представителя. Участнику от 14 до 18 лет,
        поэтому компании увидят его профиль только с вашего согласия. Пройти тест он может и без него.
      </p>

      <div id="answer" className="mt-8 scroll-mt-28">
        <Outcome v={v} />
      </div>

      <section aria-labelledby="facts" className="mt-10">
        <h2 id="facts" className="text-2xl font-semibold tracking-[-0.025em] text-ink">
          Коротко о главном
        </h2>
        <ul className="mt-6 grid gap-4 sm:grid-cols-2">
          {FACTS.map(({ icon: Icon, title, text }) => (
            <li key={title} className="rounded-card border border-line p-5">
              <span className="flex size-9 items-center justify-center rounded-[10px] bg-primary-50 text-primary">
                <Icon className="size-4.5" aria-hidden />
              </span>
              <h3 className="mt-3 font-semibold tracking-[-0.01em] text-ink">{title}</h3>
              <p className="mt-1.5 text-[15px] leading-relaxed text-ink-muted">{text}</p>
            </li>
          ))}
        </ul>
      </section>

      <section aria-labelledby="document" className="mt-10">
        <h2 id="document" className="text-2xl font-semibold tracking-[-0.025em] text-ink">
          Текст согласия
        </h2>
        <p className="mt-1 text-sm text-ink-subtle">
          {v.document.title}, редакция {v.document.version} ·{" "}
          <Link href="/privacy#guardian_consent" className="link-inline">
            политика обработки данных
          </Link>
        </p>
        <div className="mt-4 rounded-card border border-line bg-canvas p-5 text-[15px] leading-relaxed whitespace-pre-line text-ink-muted">{v.document.body}</div>
      </section>

      {v.actions.length > 0 && (
        <div className="mt-8 flex flex-col gap-3 border-t border-line pt-6 sm:flex-row sm:items-center">
          {can("grant") && (
            <Button size="lg" loading={answer.isPending && answer.variables === "grant"} disabled={answer.isPending} onClick={() => act("grant")}>
              Подтверждаю согласие
            </Button>
          )}
          {can("decline") && (
            <Button size="lg" variant="secondary" disabled={answer.isPending} onClick={() => setConfirming("decline")}>
              Не согласен
            </Button>
          )}
          {can("revoke") && (
            <Button size="lg" variant="danger" disabled={answer.isPending} onClick={() => setConfirming("revoke")}>
              Отозвать согласие
            </Button>
          )}
          {can("grant") && v.link_expires_at && <p className="text-sm text-ink-subtle sm:ml-auto">Ссылка действует до {formatDate(v.link_expires_at)}</p>}
        </div>
      )}

      <Dialog
        open={confirming !== null}
        onClose={() => setConfirming(null)}
        size="sm"
        title={confirming ? CONFIRM_TEXT[confirming].title : ""}
        footer={
          confirming && (
            <>
              <Button variant="ghost" onClick={() => setConfirming(null)}>
                Отмена
              </Button>
              <Button variant={confirming === "revoke" ? "danger" : "primary"} loading={answer.isPending} onClick={() => act(confirming)}>
                {CONFIRM_TEXT[confirming].button}
              </Button>
            </>
          )
        }
      >
        <p className="text-sm text-ink-muted">{confirming ? CONFIRM_TEXT[confirming].text : null}</p>
      </Dialog>
    </article>
  );
}
