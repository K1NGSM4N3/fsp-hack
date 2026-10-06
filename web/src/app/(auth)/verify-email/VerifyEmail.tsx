"use client";

import { CheckCircle2, XCircle } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { Button, ButtonLink, Field, Input, Spinner, useToast } from "@/components/ui";
import { errorText } from "@/components/ui/QueryState";
import { useResendVerification, useVerifyEmail } from "@/features/auth/hooks";

function Resend() {
  const toast = useToast();
  const resend = useResendVerification();
  const [email, setEmail] = useState("");
  return (
    <form
      noValidate
      className="mt-6 space-y-3"
      onSubmit={(e) => {
        e.preventDefault();
        if (email.trim()) resend.mutate(email.trim(), { onSuccess: () => toast.success("Если такой аккаунт есть, письмо уже в пути") });
      }}
    >
      <Field label="Email для новой ссылки">
        <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="email" />
      </Field>
      <Button type="submit" variant="secondary" loading={resend.isPending}>
        Отправить новую ссылку
      </Button>
    </form>
  );
}

export function VerifyEmail() {
  const token = useSearchParams().get("token");
  const verify = useVerifyEmail();
  const started = useRef(false);

  useEffect(() => {
    if (!token || started.current) return;
    started.current = true;
    verify.mutate(token);
  }, [token, verify]);

  if (!token) {
    return (
      <div>
        <XCircle className="size-10 text-accent-strong" aria-hidden />
        <h1 className="mt-4 text-[28px] leading-tight font-semibold tracking-[-0.03em]">В ссылке нет токена</h1>
        <p className="mt-2 text-ink-muted">Откройте ссылку из письма целиком или запросите новую.</p>
        <Resend />
      </div>
    );
  }

  if (verify.isSuccess) {
    return (
      <div>
        <CheckCircle2 className="size-10 text-success" aria-hidden />
        <h1 className="mt-4 text-[28px] leading-tight font-semibold tracking-[-0.03em]">Email подтверждён</h1>
        <p className="mt-2 text-ink-muted">Теперь можно войти. Если вы ищете работу, после входа заполните профиль и пройдите тест, чтобы получить категорию.</p>
        <ButtonLink href="/login" size="lg" className="mt-6">
          Войти
        </ButtonLink>
      </div>
    );
  }

  if (verify.isError) {
    const { title, detail } = errorText(verify.error);
    return (
      <div>
        <XCircle className="size-10 text-accent-strong" aria-hidden />
        <h1 className="mt-4 text-[28px] leading-tight font-semibold tracking-[-0.03em]">{title}</h1>
        {detail && <p className="mt-2 text-ink-muted">{detail}</p>}
        <p className="mt-4 text-sm text-ink-muted">
          Уже подтверждали почту по этой ссылке?{" "}
          <Link href="/login" className="link-inline font-medium">
            Войти
          </Link>
        </p>
        <Resend />
      </div>
    );
  }

  return (
    <div className="flex items-center gap-3 text-ink-muted" role="status">
      <Spinner />
      Подтверждаем email…
    </div>
  );
}
