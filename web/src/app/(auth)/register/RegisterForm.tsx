"use client";

import { BadgeCheck, Building2, MailCheck, Sprout, UserRound } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useState } from "react";
import { isApiError, USE_MOCKS } from "@/api/client";
import type { ConsentDocument, RegisterRequest } from "@/api/types";
import { Alert, Button, ButtonLink, Checkbox, Dialog, ErrorAlert, Field, FieldSet, Input, RadioCards, Skeleton, useToast } from "@/components/ui";
import { useConsentDocuments, useRegister, useResendVerification } from "@/features/auth/hooks";

type RoleChoice = "candidate" | "employer";
type AgeChoice = "adult" | "minor" | "under14";

interface Minor {
  birth: string;
  name: string;
  email: string;
}

interface Errors {
  email?: string;
  password?: string;
  company_name?: string;
  consents?: string;
  age?: string;
  birth?: string;
  guardian_name?: string;
  guardian_email?: string;
}

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/** Full years on `today`; the server repeats the check, this only saves a round trip. */
function fullYears(birth: string, today: Date): number | null {
  const [y, m, d] = birth.split("-").map(Number);
  if (!y || !m || !d) return null;
  let years = today.getFullYear() - y;
  if (today.getMonth() + 1 < m || (today.getMonth() + 1 === m && today.getDate() < d)) years -= 1;
  return years;
}

function validate(role: RoleChoice, age: AgeChoice | null, minor: Minor, email: string, password: string, company: string, docs: ConsentDocument[], accepted: Set<string>): Errors {
  const e: Errors = {};
  if (!EMAIL_RE.test(email.trim())) e.email = "Введите корректный email";
  if (password.length < 8) e.password = "Минимум 8 символов";
  if (role === "employer" && !company.trim()) e.company_name = "Укажите название компании";
  if (docs.some((d) => d.required_for.includes(role) && !accepted.has(d.id))) e.consents = "Чтобы продолжить, примите обязательные согласия";
  if (role === "candidate" && !age) e.age = "Выберите вариант";
  if (role === "candidate" && age === "minor") {
    const years = minor.birth ? fullYears(minor.birth, new Date()) : null;
    if (years === null) e.birth = "Укажите дату рождения";
    else if (years < 14) e.birth = "Регистрация открыта с 14 лет";
    else if (years >= 18) e.birth = "По дате рождения вам уже есть 18 — выберите «Мне есть 18 лет»";
    if (minor.name.trim().length < 2) e.guardian_name = "Укажите ФИО законного представителя";
    if (!EMAIL_RE.test(minor.email.trim())) e.guardian_email = "Введите корректный email";
    else if (minor.email.trim().toLowerCase() === email.trim().toLowerCase()) e.guardian_email = "Нужна почта взрослого, а не ваша";
  }
  return e;
}

/** A kind stop instead of a form: nobody under 14 gets an account, not even a half-made one. */
function TooYoung() {
  return (
    <div className="rounded-card border border-line bg-canvas p-5 sm:p-6" role="status">
      <span className="flex size-10 items-center justify-center rounded-full bg-primary-50 text-primary">
        <Sprout className="size-5" aria-hidden />
      </span>
      <h2 className="mt-4 text-lg font-semibold tracking-[-0.015em] text-ink">Регистрация откроется с 14 лет</h2>
      <p className="mt-2 text-sm leading-relaxed text-ink-muted">
        Здорово, что вы уже думаете о работе в ИТ. Завести аккаунт можно с 14 лет — вместе с согласием родителя или другого законного представителя. Мы ничего не сохранили.
      </p>
      <p className="mt-3 text-sm leading-relaxed text-ink-muted">
        А пока лучшая подготовка — соревнования по спортивному программированию: разряды и призовые места ФСП потом поднимут вас в подборке.
      </p>
      <div className="mt-5 flex flex-col gap-2 sm:flex-row">
        <ButtonLink href="/fsp" variant="secondary">
          Про соревнования ФСП
        </ButtonLink>
        <ButtonLink href="/" variant="ghost">
          На главную
        </ButtonLink>
      </div>
    </div>
  );
}

function Sent({ email, guardian }: { email: string; guardian: string | null }) {
  const toast = useToast();
  const resend = useResendVerification();
  const [demoLink, setDemoLink] = useState<string | null>(null);
  return (
    <div>
      <div className="flex size-12 items-center justify-center rounded-full bg-primary-50 text-primary">
        <MailCheck className="size-6" aria-hidden />
      </div>
      <h1 className="mt-5 text-[28px] leading-tight font-semibold tracking-[-0.03em] text-ink">Подтвердите email</h1>
      <p className="mt-2 text-ink-muted">
        Мы отправили ссылку на <span className="font-medium text-ink">{email}</span>. Перейдите по ней, чтобы активировать аккаунт. Ссылка действует 24 часа. Не видите письма — загляните в «Спам».
      </p>
      {guardian && (
        <p className="mt-3 text-ink-muted">
          Как только вы подтвердите почту, мы напишем вашему законному представителю на <span className="font-medium text-ink">{guardian}</span>. Пройти тест можно, не дожидаясь его ответа.
        </p>
      )}
      <div className="mt-6 flex flex-col gap-2 sm:flex-row">
        <Button variant="secondary" loading={resend.isPending} onClick={() => resend.mutate(email, { onSuccess: () => toast.success("Письмо отправлено повторно", "Если его нет во входящих, проверьте «Спам»") })}>
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
  const [age, setAge] = useState<AgeChoice | null>(null);
  const [minor, setMinor] = useState<Minor>({ birth: "", name: "", email: "" });
  const [accepted, setAccepted] = useState<Set<string>>(new Set());
  const [errors, setErrors] = useState<Errors>({});
  const [openDoc, setOpenDoc] = useState<ConsentDocument | null>(null);
  const [sentTo, setSentTo] = useState<{ email: string; guardian: string | null } | null>(null);
  const docs = useConsentDocuments();
  const register = useRegister();

  if (sentTo) return <Sent email={sentTo.email} guardian={sentTo.guardian} />;

  const roleDocs = (docs.data ?? []).filter((d) => d.required_for.includes(role));
  const tooYoung = role === "candidate" && age === "under14";
  const asMinor = role === "candidate" && age === "minor";

  const submit = () => {
    const e = validate(role, age, minor, email, password, company, docs.data ?? [], accepted);
    setErrors(e);
    if (Object.keys(e).length || tooYoung) return;
    const body: RegisterRequest = {
      email: email.trim(),
      password,
      role,
      company_name: role === "employer" ? company.trim() : undefined,
      consents: roleDocs.filter((d) => accepted.has(d.id)).map((d) => d.id),
    };
    if (role === "candidate") {
      body.age = asMinor ? "minor" : "adult";
      if (asMinor) body.minor = { birth_date: minor.birth, guardian: { full_name: minor.name.trim(), email: minor.email.trim() } };
    }
    register.mutate(body, {
      onSuccess: () => setSentTo({ email: email.trim(), guardian: asMinor ? minor.email.trim() : null }),
      onError: (err) => {
        if (!isApiError(err)) return;
        if (err.code === "email_taken" || err.status === 409) setErrors({ email: "Этот email уже зарегистрирован" });
        else if (err.errors) {
          setErrors({
            email: err.fieldError("email"),
            password: err.fieldError("password"),
            company_name: err.fieldError("company_name"),
            consents: err.fieldError("consents"),
            age: err.fieldError("age") ?? err.fieldError("minor"),
            birth: err.fieldError("minor.birth_date"),
            guardian_name: err.fieldError("minor.guardian.full_name"),
            guardian_email: err.fieldError("minor.guardian.email"),
          });
        }
      },
    });
  };
  const setMinorField = (patch: Partial<Minor>) => setMinor((m) => ({ ...m, ...patch }));

  return (
    <>
      <h1 className="text-[28px] leading-tight font-semibold tracking-[-0.03em] text-ink">Регистрация</h1>
      <p className="mt-2 text-sm text-ink-muted">
        Уже есть аккаунт?{" "}
        <Link href="/login" className="link-inline font-medium">
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
              { value: "candidate", title: "Ищу работу", description: "Один тест, дальше компании пишут вам сами и называют зарплату", icon: <UserRound className="size-5" aria-hidden /> },
              { value: "employer", title: "Ищу специалистов", description: "Подборка людей, чей уровень уже проверен тестом", icon: <Building2 className="size-5" aria-hidden /> },
            ]}
          />
        </FieldSet>

        {role === "candidate" && (
          <FieldSet legend="Сколько вам лет?" error={errors.age} required>
            <RadioCards<AgeChoice>
              name="age"
              columns={3}
              value={age}
              onChange={(a) => {
                setAge(a);
                setErrors((e) => ({ ...e, age: undefined }));
              }}
              options={[
                { value: "adult", title: "Мне есть 18 лет" },
                { value: "minor", title: "Мне 14–17 лет" },
                { value: "under14", title: "Мне меньше 14" },
              ]}
            />
          </FieldSet>
        )}

        {tooYoung && <TooYoung />}

        {asMinor && (
          <div className="space-y-4 rounded-card border border-line bg-canvas p-4 sm:p-5">
            <p className="text-sm leading-relaxed text-ink-muted">
              До 18 лет компании увидят ваш профиль, когда согласие подтвердит родитель или другой законный представитель. Мы напишем ему сами, как только вы подтвердите свою почту. Тест можно пройти
              сразу.
            </p>
            <Field label="Дата рождения" error={errors.birth} required hint="Нужна, чтобы в день 18-летия ограничения снялись сами">
              <Input type="date" value={minor.birth} onChange={(e) => setMinorField({ birth: e.target.value })} max={new Date().toISOString().slice(0, 10)} autoComplete="bday" />
            </Field>
            <Field label="ФИО законного представителя" error={errors.guardian_name} required>
              <Input value={minor.name} onChange={(e) => setMinorField({ name: e.target.value })} placeholder="Петрова Мария Ивановна" autoComplete="off" />
            </Field>
            <Field label="Email законного представителя" error={errors.guardian_email} required hint="Родитель, усыновитель или попечитель — не ваша почта">
              <Input type="email" value={minor.email} onChange={(e) => setMinorField({ email: e.target.value })} placeholder="parent@example.ru" autoComplete="off" />
            </Field>
          </div>
        )}

        {!tooYoung && role === "employer" && (
          <Field label="Название компании" error={errors.company_name} required>
            <Input value={company} onChange={(e) => setCompany(e.target.value)} autoComplete="organization" placeholder="ООО «Пример»" />
          </Field>
        )}
        {!tooYoung && (
          <>
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
                          <button type="button" className="link-inline inline font-medium" onClick={() => setOpenDoc(d)}>
                            (текст)
                          </button>
                        </>
                      }
                      description={
                        <>
                          Редакция {d.version} ·{" "}
                          <Link href={`/privacy#${d.kind}`} target="_blank" className="link-inline">
                            на отдельной странице
                          </Link>
                        </>
                      }
                    />
                  ))}
                  {role === "employer" && (
                    <p className="flex items-start gap-2.5 text-sm text-ink">
                      <BadgeCheck className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />Я представляю компанию и мне есть 18 лет
                    </p>
                  )}
                  <p className="text-xs text-ink-subtle">
                    Что мы храним, кому показываем и как всё удалить —{" "}
                    <Link href="/privacy" target="_blank" className="link-inline font-medium">
                      в политике обработки персональных данных
                    </Link>
                    .
                  </p>
                </div>
              )}
            </FieldSet>

            {register.isError && !isApiError(register.error) && <Alert tone="danger" title="Не удалось зарегистрироваться" />}

            <Button type="submit" size="lg" className="w-full" loading={register.isPending}>
              Создать аккаунт
            </Button>
          </>
        )}
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
