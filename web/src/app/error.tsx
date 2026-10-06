"use client";

import { Button, ButtonLink } from "@/components/ui/Button";

export default function GlobalError({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return (
    <main id="main" className="flex flex-1 flex-col items-center justify-center px-4 py-24 text-center">
      <h1 className="font-mono text-2xl font-semibold text-ink">Что-то пошло не так</h1>
      <p className="mt-2 max-w-md text-ink-muted">Мы уже знаем об ошибке. Попробуйте обновить страницу.</p>
      <div className="mt-8 flex flex-wrap justify-center gap-3">
        <Button onClick={reset}>Попробовать снова</Button>
        <ButtonLink href="/" variant="secondary">
          На главную
        </ButtonLink>
      </div>
    </main>
  );
}
