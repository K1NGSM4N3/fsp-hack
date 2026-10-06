import { NodeLines } from "@/components/brand/Logo";
import { ButtonLink } from "@/components/ui/Button";

export default function NotFound() {
  return (
    <main id="main" className="relative isolate flex flex-1 flex-col items-center justify-center overflow-hidden bg-night px-4 py-24 text-center text-white">
      <div aria-hidden className="pointer-events-none absolute -top-40 left-1/2 -z-10 size-[40rem] -translate-x-1/2 rounded-full bg-primary/35 blur-[150px]" />
      <NodeLines className="pointer-events-none absolute -top-10 right-0 -z-10 h-80 w-[30rem] text-primary-400/40" />
      <p className="font-display text-[clamp(6rem,20vw,12rem)] leading-none font-semibold tracking-[-0.06em] text-transparent [-webkit-text-stroke:1.5px_rgb(169_163_255/0.8)]">404</p>
      <h1 className="mt-6 text-3xl font-semibold tracking-[-0.03em]">Страница не найдена</h1>
      <p className="mt-3 max-w-md text-night-muted">Возможно, ссылка устарела или страница переехала. Начните с главной — там всё по полочкам.</p>
      <div className="mt-10 flex flex-wrap justify-center gap-3">
        <ButtonLink href="/" size="lg">
          На главную
        </ButtonLink>
        <ButtonLink href="/vacancies" variant="secondary" size="lg">
          Вакансии
        </ButtonLink>
      </div>
    </main>
  );
}
