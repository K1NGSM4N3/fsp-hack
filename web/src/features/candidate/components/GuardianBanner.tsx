"use client";

import { PencilLine, RotateCcw } from "lucide-react";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { GuardianState, GuardianStatus } from "@/api/types";
import { Alert, Button, Dialog, Field, Input, useToast } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { useChangeGuardian, useGuardianStatus, useResendGuardianLetter } from "@/features/candidate/hooks";
import { formatDate } from "@/lib/format";

type Waiting = Exclude<GuardianState, "granted">;

const TITLE: Record<Waiting, string> = {
  not_sent: "Ждём подтверждения от законного представителя",
  pending: "Ждём подтверждения от законного представителя",
  expired: "Ждём подтверждения от законного представителя",
  declined: "Законный представитель не дал согласие",
  revoked: "Законный представитель отозвал согласие",
};

function Explanation({ s, state }: { s: GuardianStatus; state: Waiting }) {
  const who = (
    <>
      <span className="font-medium text-ink">{s.guardian_name}</span> ({s.guardian_email})
    </>
  );
  switch (state) {
    case "not_sent":
      return <>Письмо законному представителю — {who} — ещё не ушло. Отправьте его — после согласия компании увидят ваш профиль.</>;
    case "pending":
      return (
        <>
          Письмо законному представителю — {who} — ушло {formatDate(s.sent_at)}. Пока согласия нет, компании не видят ваш профиль, но тест можно пройти уже сейчас. Ссылка в письме действует до {formatDate(s.link_expires_at)}.
        </>
      );
    case "expired":
      return <>Ссылка в письме законному представителю — {who} — устарела, а ответа нет. Отправьте письмо ещё раз или укажите другого представителя.</>;
    case "declined":
      return <>Ответ от {formatDate(s.answered_at)}: «Не согласен». Поэтому профиль скрыт от компаний. Если это ошибка, отправьте письмо ещё раз или укажите другого представителя.</>;
    case "revoked":
      return (
        <>Согласие отозвано {formatDate(s.revoked_at)}, поэтому профиль скрыт от компаний. Чтобы вернуться в поиск, нужно новое согласие: отправьте письмо ещё раз или укажите другого представителя.</>
      );
  }
}

function ChangeDialog({ open, onClose, current }: { open: boolean; onClose: () => void; current: GuardianStatus }) {
  const change = useChangeGuardian();
  const toast = useToast();
  const [name, setName] = useState(current.guardian_name ?? "");
  const [email, setEmail] = useState(current.guardian_email ?? "");
  const fieldError = (f: string) => (isApiError(change.error) ? change.error.fieldError(f) : undefined);
  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="sm"
      title="Другой адрес представителя"
      description="Ссылки из прежнего письма сразу перестанут работать."
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Отмена
          </Button>
          <Button
            loading={change.isPending}
            onClick={() =>
              change.mutate(
                { full_name: name, email },
                {
                  onSuccess: (s) => {
                    toast.success("Письмо отправлено", `На ${s.guardian_email}`);
                    onClose();
                  },
                  onError: (e) => {
                    if (isApiError(e) && !e.errors) toast.error(e.title, e.detail ?? undefined);
                  },
                },
              )
            }
          >
            Сохранить и отправить
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <Field label="ФИО законного представителя" error={fieldError("full_name")} required>
          <Input value={name} onChange={(e) => setName(e.target.value)} autoComplete="off" />
        </Field>
        <Field label="Email" error={fieldError("email")} required hint="Родитель, усыновитель или попечитель">
          <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="off" />
        </Field>
      </div>
    </Dialog>
  );
}

/** Shown across the cabinet while a 14–17-year-old waits for the representative or lost the consent. */
export function GuardianBanner() {
  const me = useMe();
  const age = me.data?.candidate?.age;
  const waiting = age?.bracket === "minor" && age.guardian !== "granted";
  const q = useGuardianStatus(waiting);
  const resend = useResendGuardianLetter();
  const toast = useToast();
  const [changing, setChanging] = useState(false);
  const s = q.data;
  if (!waiting || !s?.required || !s.state || s.state === "granted") return null;
  const state = s.state;
  return (
    <>
      <Alert
        tone={state === "declined" || state === "revoked" ? "danger" : state === "expired" ? "warning" : "info"}
        title={TITLE[state]}
        className="mb-6"
        action={
          <>
            <Button
              size="sm"
              variant="secondary"
              icon={<RotateCcw className="size-4" aria-hidden />}
              loading={resend.isPending}
              onClick={() =>
                resend.mutate(undefined, {
                  onSuccess: (r) => toast.success("Письмо отправлено", `На ${r.guardian_email}`),
                  onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось отправить", isApiError(e) ? (e.detail ?? undefined) : undefined),
                })
              }
            >
              {state === "not_sent" ? "Отправить письмо" : "Отправить письмо ещё раз"}
            </Button>
            <Button size="sm" variant="ghost" icon={<PencilLine className="size-4" aria-hidden />} onClick={() => setChanging(true)}>
              Изменить адрес
            </Button>
          </>
        }
      >
        <p>
          <Explanation s={s} state={state} />
        </p>
        {s.adult_from && <p className="mt-1.5 text-xs text-ink-subtle">С {formatDate(`${s.adult_from}T12:00:00`)}, в день 18-летия, согласие представителя больше не понадобится.</p>}
      </Alert>
      {changing && <ChangeDialog open onClose={() => setChanging(false)} current={s} />}
    </>
  );
}
