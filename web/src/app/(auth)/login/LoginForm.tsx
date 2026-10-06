"use client";

import { useQueryClient } from "@tanstack/react-query";
import { Building2, LogIn, ShieldCheck, UserRound } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useState, type ChangeEvent, type FormEvent } from "react";
import { isApiError, USE_MOCKS } from "@/api/client";
import type { Me } from "@/api/types";
import { Alert, Button, Field, Input, useToast } from "@/components/ui";
import { useLogin, useResendVerification } from "@/features/auth/hooks";
import { qk } from "@/features/keys";

const DEMO_PASSWORD = "demo12345";
const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

interface Values {
  email: string;
  password: string;
}

type Errors = Partial<Record<keyof Values, string>>;

/** Two fields do not need a schema library; keeping zod out of this route keeps it out of every public page that prefetches «Войти». */
function validate(v: Values): Errors {
  const email = v.email.trim();
  const errors: Errors = {};
  if (!email) errors.email = "Введите email";
  else if (!EMAIL.test(email)) errors.email = "Некорректный email";
  if (!v.password) errors.password = "Введите пароль";
  return errors;
}

const HOME: Record<Me["role"], string> = { candidate: "/candidate", employer: "/employer", admin: "/admin" };

function destination(me: Me | undefined, next: string | null): string {
  const home = me ? HOME[me.role] : "/candidate";
  if (next && next.startsWith(home)) return next;
  if (next && next.startsWith("/vacancies")) return next;
  return home;
}

export function LoginForm() {
  const router = useRouter();
  const params = useSearchParams();
  const next = params.get("next");
  const toast = useToast();
  const qc = useQueryClient();
  const login = useLogin();
  const resend = useResendVerification();
  const [unverifiedEmail, setUnverifiedEmail] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const [values, setValues] = useState<Values>({ email: "", password: "" });
  const [errors, setErrors] = useState<Errors>({});
  const [submitted, setSubmitted] = useState(false);

  const change = (key: keyof Values) => (e: ChangeEvent<HTMLInputElement>) => {
    const updated = { ...values, [key]: e.target.value };
    setValues(updated);
    if (submitted) setErrors(validate(updated));
  };

  const submit = (raw: Values) => {
    setSubmitted(true);
    const found = validate(raw);
    setErrors(found);
    if (found.email || found.password) return;
    const body = { ...raw, email: raw.email.trim() };
    setFormError(null);
    setUnverifiedEmail(null);
    login.mutate(body, {
      onSuccess: () => router.replace(destination(qc.getQueryData<Me>(qk.me), next)),
      onError: (err) => {
        if (isApiError(err) && err.code === "email_not_verified") setUnverifiedEmail(body.email);
        else if (isApiError(err) && err.status === 401) setFormError("Неверный email или пароль");
        else setFormError(isApiError(err) ? err.title : "Не удалось войти");
      },
    });
  };

  const demo = (email: string) => {
    const filled = { email, password: DEMO_PASSWORD };
    setValues(filled);
    submit(filled);
  };

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    submit(values);
  };

  return (
    <div className="mt-8 space-y-6">
      {unverifiedEmail && (
        <Alert
          tone="warning"
          title="Подтвердите email"
          action={
            <Button
              variant="secondary"
              size="sm"
              loading={resend.isPending}
              onClick={() => resend.mutate(unverifiedEmail, { onSuccess: () => toast.success("Письмо отправлено", `Проверьте ${unverifiedEmail}`) })}
            >
              Отправить письмо ещё раз
            </Button>
          }
        >
          Мы отправили ссылку на {unverifiedEmail}. Перейдите по ней и войдите снова.
        </Alert>
      )}
      {formError && <Alert tone="danger" title={formError} />}

      <form noValidate onSubmit={onSubmit} className="space-y-4">
        <Field label="Email" error={errors.email} required>
          <Input type="email" name="email" autoComplete="email" placeholder="name@example.ru" value={values.email} onChange={change("email")} />
        </Field>
        <Field label="Пароль" error={errors.password} required>
          <Input type="password" name="password" autoComplete="current-password" value={values.password} onChange={change("password")} />
        </Field>
        <Button type="submit" size="lg" className="w-full" loading={login.isPending} icon={<LogIn className="size-5" aria-hidden />}>
          Войти
        </Button>
      </form>

      <p className="text-center text-sm text-ink-muted">
        Нет аккаунта?{" "}
        <Link href="/register" className="link-inline font-medium">
          Зарегистрироваться
        </Link>
      </p>

      {(USE_MOCKS || process.env.NEXT_PUBLIC_DEMO_LOGIN !== "0") && (
        <div className="rounded-card border border-dashed border-line-strong p-4">
          <p className="text-sm font-medium text-ink">Демо-аккаунты</p>
          <p className="mt-1 text-xs text-ink-subtle">Пароль {DEMO_PASSWORD}. {USE_MOCKS ? "Данные живут в этом браузере." : "Доступны, если API запущен с SEED_DEMO=true."}</p>
          <div className={USE_MOCKS ? "mt-3 grid gap-2 sm:grid-cols-2" : "mt-3 grid gap-2 sm:grid-cols-3"}>
            <Button variant="soft" size="sm" icon={<UserRound className="size-4" aria-hidden />} onClick={() => demo("candidate@demo.fsp")} disabled={login.isPending}>
              Соискатель
            </Button>
            <Button variant="soft" size="sm" icon={<Building2 className="size-4" aria-hidden />} onClick={() => demo("employer@demo.fsp")} disabled={login.isPending}>
              Компания
            </Button>
            {!USE_MOCKS && (
              <Button variant="soft" size="sm" icon={<ShieldCheck className="size-4" aria-hidden />} onClick={() => demo("moderator@demo.fsp")} disabled={login.isPending}>
                Модератор
              </Button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
