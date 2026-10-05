"use client";

import { Link2, RefreshCw } from "lucide-react";
import { useState } from "react";
import { isApiError, USE_MOCKS } from "@/api/client";
import { FSP_LINK_START_URL } from "@/api/endpoints";
import { Button, Dialog, Field, Input, useToast } from "@/components/ui";
import { useFspLinkDev } from "@/features/candidate/hooks";

const DEMO_IDS = [
  { id: "FSP-000001", note: "алгоритмическое программирование, 9 достижений" },
  { id: "FSP-000003", note: "информационная безопасность, 8 достижений" },
  { id: "FSP-000006", note: "продуктовое программирование, 3 достижения" },
  { id: "FSP-000010", note: "участник без достижений" },
];

/** Real mode: OIDC redirect via the API. Mock mode: a stand-in for the FSP ID login screen. */
export function LinkFspButton({ variant = "primary", size = "md", label = "Привязать ФСП ID" }: { variant?: "primary" | "secondary" | "soft"; size?: "sm" | "md"; label?: string }) {
  const [open, setOpen] = useState(false);
  const [fspId, setFspId] = useState("");
  const [error, setError] = useState<string | null>(null);
  const link = useFspLinkDev();
  const toast = useToast();

  const start = () => {
    if (!USE_MOCKS) {
      window.location.href = FSP_LINK_START_URL;
      return;
    }
    setOpen(true);
  };

  const submit = (id: string) => {
    setError(null);
    link.mutate(id, {
      onSuccess: (s) => {
        setOpen(false);
        toast.success("ФСП ID привязан", s.achievements.length ? `Загружено достижений: ${s.achievements.length}` : "Достижений пока нет — это не влияет на позицию негативно");
      },
      onError: (e) => setError(isApiError(e) ? (e.fieldError("fsp_id") ?? e.detail ?? e.title) : "Не удалось привязать"),
    });
  };

  return (
    <>
      <Button variant={variant} size={size} icon={<Link2 className="size-4" aria-hidden />} onClick={start}>
        {label}
      </Button>
      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Вход через ФСП ID"
        description="Демо-режим: так выглядит шаг входа в Keycloak ФСП. В рабочей версии откроется страница ФСП ID."
        footer={
          <>
            <Button variant="ghost" onClick={() => setOpen(false)}>
              Отмена
            </Button>
            <Button loading={link.isPending} onClick={() => submit(fspId)} disabled={!fspId.trim()}>
              Войти и привязать
            </Button>
          </>
        }
      >
        <form
          noValidate
          onSubmit={(e) => {
            e.preventDefault();
            submit(fspId);
          }}
          className="space-y-4"
        >
          <Field label="ФСП ID участника" error={error ?? undefined} hint="Формат FSP-000001">
            <Input value={fspId} onChange={(e) => setFspId(e.target.value)} placeholder="FSP-000001" className="font-mono uppercase" autoFocus />
          </Field>
          <div>
            <p className="mb-2 text-xs text-ink-subtle">Участники из тестового реестра:</p>
            <ul className="space-y-1.5">
              {DEMO_IDS.map((d) => (
                <li key={d.id}>
                  <button type="button" onClick={() => setFspId(d.id)} className="flex w-full items-center justify-between gap-3 rounded-control border border-line px-3 py-2 text-left text-sm hover:border-primary">
                    <span className="font-mono text-ink">{d.id}</span>
                    <span className="text-xs text-ink-subtle">{d.note}</span>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        </form>
      </Dialog>
    </>
  );
}

export function SyncIcon({ spinning }: { spinning: boolean }) {
  return <RefreshCw className={spinning ? "size-4 animate-spin" : "size-4"} aria-hidden />;
}
