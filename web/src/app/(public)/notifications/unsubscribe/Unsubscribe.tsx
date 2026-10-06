"use client";

import { BellOff, CheckCircle2, XCircle } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Button } from "@/components/ui";
import { errorText } from "@/components/ui/QueryState";
import { useUnsubscribe } from "@/features/notifications/hooks";

/**
 * Opening the link changes nothing: mail scanners follow links, so the letter's link only leads here and the button
 * does the POST. Mail clients that support one-click unsubscribe (RFC 8058) POST to the API directly.
 */
export function Unsubscribe() {
  const token = useSearchParams().get("token") ?? "";
  const run = useUnsubscribe();

  if (!token || run.isError) {
    const err = run.isError ? errorText(run.error) : null;
    return (
      <div className="mx-auto max-w-2xl">
        <XCircle className="size-10 text-accent-strong" aria-hidden />
        <h1 className="display-3 mt-4 text-balance text-ink">{err?.title ?? "В ссылке нет токена"}</h1>
        <p className="mt-4 text-lg leading-relaxed text-ink-muted">
          Откройте ссылку «Отписаться» из письма целиком. Или войдите в кабинет — письма по видам включаются и отключаются в настройках, раздел «Уведомления».
        </p>
        <Link href="/login" className="mt-6 inline-flex text-primary hover:underline">
          Войти в кабинет
        </Link>
      </div>
    );
  }

  if (run.isSuccess) {
    const done = run.data;
    return (
      <div className="mx-auto max-w-2xl" role="status">
        <CheckCircle2 className="size-10 text-success" aria-hidden />
        <h1 className="display-3 mt-4 text-balance text-ink">Готово, отписали</h1>
        <p className="mt-4 text-lg leading-relaxed text-ink-muted">
          {done.kind ? (
            <>
              Письма «<span className="font-medium text-ink">{done.title}</span>» больше не придут.
            </>
          ) : (
            <>Письма с уведомлениями больше не придут.</>
          )}{" "}
          В кабинете уведомления остаются — под колокольчиком. Письма о подтверждении почты и законному представителю сюда не относятся: без них не работает аккаунт.
        </p>
        <p className="mt-4 text-[15px] text-ink-muted">
          Передумаете — включите снова в{" "}
          <Link href={done.settings_path} className="text-primary hover:underline">
            настройках кабинета
          </Link>
          .
        </p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl">
      <span className="flex size-12 items-center justify-center rounded-2xl bg-primary-50 text-primary">
        <BellOff className="size-6" aria-hidden />
      </span>
      <h1 className="display-3 mt-5 text-balance text-ink">Отписаться от писем?</h1>
      <p className="mt-4 text-lg leading-relaxed text-ink-muted">
        Перестанем присылать на почту такие письма, как то, из которого вы пришли. Уведомления в кабинете останутся, входить в аккаунт не нужно.
      </p>
      <div className="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center">
        <Button size="lg" loading={run.isPending} onClick={() => run.mutate(token)}>
          Отписаться
        </Button>
        <Link href="/" className="text-sm text-ink-muted hover:text-primary sm:ml-2">
          Оставить как есть
        </Link>
      </div>
    </div>
  );
}
