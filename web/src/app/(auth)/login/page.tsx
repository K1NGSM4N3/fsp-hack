import type { Metadata } from "next";
import { Suspense } from "react";
import { LoginForm } from "./LoginForm";

export const metadata: Metadata = { title: "Вход" };

export default function LoginPage() {
  return (
    <>
      <h1 className="text-[28px] leading-tight font-semibold tracking-[-0.03em] text-ink">Вход</h1>
      <p className="mt-2 text-sm text-ink-muted">У каждого аккаунта свой кабинет: для поиска работы или для подбора специалистов.</p>
      <Suspense>
        <LoginForm />
      </Suspense>
    </>
  );
}
