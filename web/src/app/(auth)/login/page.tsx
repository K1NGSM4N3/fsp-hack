import type { Metadata } from "next";
import { Suspense } from "react";
import { LoginForm } from "./LoginForm";

export const metadata: Metadata = { title: "Вход" };

export default function LoginPage() {
  return (
    <>
      <h1 className="font-mono text-2xl font-semibold tracking-tight text-ink">Вход</h1>
      <p className="mt-2 text-sm text-ink-muted">Один аккаунт — один кабинет: кандидата или работодателя.</p>
      <Suspense>
        <LoginForm />
      </Suspense>
    </>
  );
}
