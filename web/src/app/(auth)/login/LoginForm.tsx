"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { Building2, LogIn, UserRound } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { isApiError, USE_MOCKS } from "@/api/client";

const DEMO_PASSWORD = "demo12345";
import type { Me } from "@/api/types";
import { Alert, Button, Field, Input, useToast } from "@/components/ui";
import { useLogin, useResendVerification } from "@/features/auth/hooks";
import { qk } from "@/features/keys";
import { useQueryClient } from "@tanstack/react-query";

const schema = z.object({
  email: z.string().trim().min(1, "Введите email").email("Некорректный email"),
  password: z.string().min(1, "Введите пароль"),
});

type Values = z.infer<typeof schema>;

function destination(me: Me | undefined, next: string | null): string {
  const home = me?.role === "employer" ? "/employer" : "/candidate";
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

  const form = useForm<Values>({ resolver: zodResolver(schema), defaultValues: { email: "", password: "" } });

  const submit = (values: Values) => {
    setFormError(null);
    setUnverifiedEmail(null);
    login.mutate(values, {
      onSuccess: () => router.replace(destination(qc.getQueryData<Me>(qk.me), next)),
      onError: (err) => {
        if (isApiError(err) && err.code === "email_not_verified") setUnverifiedEmail(values.email);
        else if (isApiError(err) && err.status === 401) setFormError("Неверный email или пароль");
        else setFormError(isApiError(err) ? err.title : "Не удалось войти");
      },
    });
  };

  const demo = (email: string) => {
    form.setValue("email", email);
    form.setValue("password", DEMO_PASSWORD);
    void form.handleSubmit(submit)();
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

      <form noValidate onSubmit={form.handleSubmit(submit)} className="space-y-4">
        <Field label="Email" error={form.formState.errors.email?.message} required>
          <Input type="email" autoComplete="email" placeholder="name@example.ru" {...form.register("email")} />
        </Field>
        <Field label="Пароль" error={form.formState.errors.password?.message} required>
          <Input type="password" autoComplete="current-password" {...form.register("password")} />
        </Field>
        <Button type="submit" size="lg" className="w-full" loading={login.isPending} icon={<LogIn className="size-5" aria-hidden />}>
          Войти
        </Button>
      </form>

      <p className="text-center text-sm text-ink-muted">
        Нет аккаунта?{" "}
        <Link href="/register" className="font-medium text-primary hover:underline">
          Зарегистрироваться
        </Link>
      </p>

      {(USE_MOCKS || process.env.NEXT_PUBLIC_DEMO_LOGIN !== "0") && (
        <div className="rounded-card border border-dashed border-line-strong p-4">
          <p className="text-sm font-medium text-ink">Демо-аккаунты</p>
          <p className="mt-1 text-xs text-ink-subtle">Пароль {DEMO_PASSWORD}. {USE_MOCKS ? "Данные живут в этом браузере." : "Доступны, если API запущен с SEED_DEMO=true."}</p>
          <div className="mt-3 grid gap-2 sm:grid-cols-2">
            <Button variant="soft" size="sm" icon={<UserRound className="size-4" aria-hidden />} onClick={() => demo("candidate@demo.fsp")} disabled={login.isPending}>
              Соискатель
            </Button>
            <Button variant="soft" size="sm" icon={<Building2 className="size-4" aria-hidden />} onClick={() => demo("employer@demo.fsp")} disabled={login.isPending}>
              Компания
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
