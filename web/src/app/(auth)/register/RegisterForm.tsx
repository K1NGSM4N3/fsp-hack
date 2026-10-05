"use client";

import { Building2, MailCheck, UserRound } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useState } from "react";
import { isApiError, USE_MOCKS } from "@/api/client";
import type { ConsentDocument } from "@/api/types";
import { Alert, Button, ButtonLink, Checkbox, Dialog, ErrorAlert, Field, FieldSet, Input, RadioCards, Skeleton, useToast } from "@/components/ui";
import { useConsentDocuments, useRegister, useResendVerification } from "@/features/auth/hooks";

type RoleChoice = "candidate" | "employer";

interface Errors {
  email?: string;
  password?: string;
  company_name?: string;
  consents?: string;
}

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function validate(role: RoleChoice, email: string, password: string, company: string, docs: ConsentDocument[], accepted: Set<string>): Errors {
  const e: Errors = {};
  if (!EMAIL_RE.test(email.trim())) e.email = "Введите корректный email";
  if (password.length < 8) e.password = "Минимум 8 символов";
  if (role === "employer" && !company.trim()) e.company_name = "Укажите название компании";
  if (docs.some((d) => d.required_for.includes(role) && !accepted.has(d.id))) e.consents = "Чтобы продолжить, примите обязательные согласия";
  return e;
}

function Sent({ email }: { email: string }) {
  const toast = useToast();
  const resend = useResendVerification();
  const [demoLink, setDemoLink] = useState<string | null>(null);
  return (
    <div>
      <div className="flex size-12 items-center justify-center rounded-full bg-primary-50 text-primary">
        <MailCheck className="size-6" aria-hidden />
      </div>
      <h1 className="mt-5 font-mono text-2xl font-semibold tracking-tight text-ink">Подтвердите email</h1>
      <p className="mt-2 text-ink-muted">
        Мы отправили ссылку на <span className="font-medium text-ink">{email}</span>. Перейдите по ней, чтобы активировать аккаунт. Ссылка действует 24 часа.
      </p>
      <div className="mt-6 flex flex-col gap-2 sm:flex-row">
        <Button variant="secondary" loading={resend.isPending} onClick={() => resend.mutate(email, { onSuccess: () => toast.success("Письмо отправлено повторно") })}>
          Отправить ещё раз
        </Button>
        <ButtonLink href="/login" variant="ghost">
          Ко входу
        </ButtonLink>
      </div>
      {USE_MOCKS && (
        <div className="mt-8 rounded-card border border-dashed border-line-strong p-4 text-sm">
          <p className="font-medium text-ink">Демо-почта</p>
          <p className="mt-1 text-ink-muted">В демо-режиме письмо не уходит — откройте ссылку из него здесь.</p>
          {demoLink ? (
            <ButtonLink href={demoLink} className="mt-3" size="sm">
              Перейти по ссылке из письма
            </ButtonLink>
          ) : (
            <Button
              className="mt-3"
              size="sm"
              variant="soft"
              onClick={async () => {
                const m = await import("@/mocks/handlers");
                setDemoLink(m.mockVerificationLink(email));
              }}
            >
              Открыть письмо
            </Button>
          )}
        </div>
      )}
    </div>
  );
}

export function RegisterForm() {
  const params = useSearchParams();
  const initialRole: RoleChoice = params.get("role") === "employer" ? "employer" : "candidate";
  const [role, setRole] = useState<RoleChoice>(initialRole);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [company, setCompany] = useState("");
  const [accepted, setAccepted] = useState<Set<string>>(new Set());
  const [errors, setErrors] = useState<Errors>({});
  const [openDoc, setOpenDoc] = useState<ConsentDocument | null>(null);
  const [sentTo, setSentTo] = useState<string | null>(null);
  const docs = useConsentDocuments();
  const register = useRegister();

  if (sentTo) return <Sent email={sentTo} />;

  const roleDocs = (docs.data ?? []).filter((d) => d.required_for.includes(role));

  const submit = () => {
    const e = validate(role, email, password, company, docs.data ?? [], accepted);
    setErrors(e);
    if (Object.keys(e).length) return;
    register.mutate(
      {
        email: email.trim(),
        password,
        role,
        company_name: role === "employer" ? company.trim() : undefined,
        consents: roleDocs.filter((d) => accepted.has(d.id)).map((d) => d.id),
      },
      {
        onSuccess: () => setSentTo(email.trim()),
        onError: (err) => {
          if (!isApiError(err)) return;
          if (err.code === "email_taken" || err.status === 409) setErrors({ email: "Этот email уже зарегистрирован" });
          else if (err.errors) {
            setErrors({
              email: err.fieldError("email"),
              password: err.fieldError("password"),
              company_name: err.fieldError("company_name"),
              consents: err.fieldError("consents"),
            });
          }
        },
      },
    );
  };

  return (
    <>
      <h1 className="font-mono text-2xl font-semibold tracking-tight text-ink">Регистрация</h1>
      <p className="mt-2 text-sm text-ink-muted">
        Уже есть аккаунт?{" "}
        <Link href="/login" className="font-medium text-primary hover:underline">
          Войти
        </Link>
      </p>

      <form
        noValidate
        className="mt-8 space-y-6"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <FieldSet legend="Кто вы?">
          <RadioCards<RoleChoice>
            name="role"
            value={role}
            onChange={(r) => {
              setRole(r);
              setErrors({});
            }}
            options={[
              { value: "candidate", title: "Ищу работу", description: "Тест, категория и приглашения с вилкой", icon: <UserRound className="size-5" aria-hidden /> },
              { value: "employer", title: "Ищу специалиста", description: "Подборка с объяснением", icon: <Building2 className="size-5" aria-hidden /> },
            ]}
          />
        </FieldSet>

        {role === "employer" && (
          <Field label="Название компании" error={errors.company_name} required>
            <Input value={company} onChange={(e) => setCompany(e.target.value)} autoComplete="organization" placeholder="ООО «Пример»" />
          </Field>
        )}
        <Field label="Email" error={errors.email} required hint="На него придёт ссылка для подтверждения">
          <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="email" placeholder="name@example.ru" />
        </Field>
        <Field label="Пароль" error={errors.password} required hint="Не короче 8 символов">
          <Input type="password" value={password} onChange={(e) => setPassword(e.target.value)} autoComplete="new-password" />
        </Field>

        <FieldSet legend="Согласия" error={errors.consents} required>
          {docs.isPending ? (
            <div className="space-y-2">
              <Skeleton className="h-5 w-full" />
              <Skeleton className="h-5 w-4/5" />
            </div>
          ) : docs.isError ? (
            <ErrorAlert error={docs.error} onRetry={() => void docs.refetch()} />
          ) : (
            <div className="space-y-3">
              {roleDocs.map((d) => (
                <Checkbox
                  key={d.id}
                  checked={accepted.has(d.id)}
                  onChange={(e) => {
                    const next = new Set(accepted);
                    if (e.target.checked) next.add(d.id);
                    else next.delete(d.id);
                    setAccepted(next);
                  }}
                  label={
                    <>
                      Я принимаю: {d.title.charAt(0).toLowerCase() + d.title.slice(1)}{" "}
                      <button type="button" className="font-medium text-primary underline-offset-2 hover:underline" onClick={() => setOpenDoc(d)}>
                        (текст)
                      </button>
                    </>
                  }
                  description={`Версия ${d.version}`}
                />
              ))}
            </div>
          )}
        </FieldSet>

        {register.isError && !isApiError(register.error) && <Alert tone="danger" title="Не удалось зарегистрироваться" />}

        <Button type="submit" size="lg" className="w-full" loading={register.isPending}>
          Создать аккаунт
        </Button>
      </form>

      <Dialog
        open={openDoc !== null}
        onClose={() => setOpenDoc(null)}
        title={openDoc?.title ?? ""}
        description={openDoc ? `Версия ${openDoc.version}` : undefined}
        footer={
          openDoc && (
            <>
              <Button variant="ghost" onClick={() => setOpenDoc(null)}>
                Закрыть
              </Button>
              <Button
                onClick={() => {
                  setAccepted(new Set(accepted).add(openDoc.id));
                  setOpenDoc(null);
                }}
              >
                Принимаю
              </Button>
            </>
          )
        }
      >
        <div className="space-y-3 text-sm leading-relaxed text-ink">
          {openDoc?.body.split(/\n{2,}/).map((p, i) => (
            <p key={i}>{p}</p>
          ))}
        </div>
      </Dialog>
    </>
  );
}
