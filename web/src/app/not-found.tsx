import { NodeLines } from "@/components/brand/Logo";
import { ButtonLink } from "@/components/ui";

export default function NotFound() {
  return (
    <main id="main" className="relative flex flex-1 flex-col items-center justify-center overflow-hidden px-4 py-24 text-center">
      <NodeLines className="pointer-events-none absolute -top-10 right-0 h-80 w-[30rem] text-primary-100" />
      <p className="font-mono text-6xl font-semibold tracking-tight text-primary">404</p>
      <h1 className="mt-4 font-mono text-2xl font-semibold text-ink">Страница не найдена</h1>
      <p className="mt-2 max-w-md text-ink-muted">Возможно, ссылка устарела или страница была перемещена.</p>
      <div className="mt-8 flex flex-wrap justify-center gap-3">
        <ButtonLink href="/">На главную</ButtonLink>
        <ButtonLink href="/vacancies" variant="secondary">
          Вакансии
        </ButtonLink>
      </div>
    </main>
  );
}
