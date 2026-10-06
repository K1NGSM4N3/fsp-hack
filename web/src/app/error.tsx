"use client";

/**
 * Plain markup on purpose: this boundary's code ships with every page, so it carries no icons, links or shared
 * components — only the look of the 404 page (`SiteState`).
 */
export default function SiteError({ error, retry }: { error: Error & { digest?: string }; retry: () => void }) {
  return (
    <div className="relative isolate flex min-h-dvh flex-1 flex-col overflow-hidden bg-night bg-[radial-gradient(46rem_46rem_at_100%_-10rem,rgb(64_47_255/0.5),transparent_62%)] text-white">
      <main id="main" className="mx-auto flex w-full max-w-[1240px] flex-1 flex-col justify-center px-4 py-16 sm:px-6 lg:px-8">
        <p aria-hidden className="font-display text-[clamp(5.5rem,22vw,13rem)] leading-[0.85] font-semibold tracking-[-0.06em] text-transparent [-webkit-text-stroke:1.5px_rgb(169_163_255/0.8)]">
          500
        </p>
        <h1 className="mt-8 max-w-2xl text-[clamp(2rem,1.4rem+2vw,3rem)] leading-[1.05] font-semibold tracking-[-0.035em]">Что-то пошло не так</h1>
        <p className="mt-4 max-w-xl text-lg leading-relaxed text-night-muted">Страница не открылась из-за ошибки на нашей стороне. Попробуйте ещё раз через минуту.</p>
        {error.digest && (
          <p className="mt-3 text-sm text-night-faint">
            Код для поддержки: <span className="font-mono text-white">{error.digest}</span>
          </p>
        )}
        <div className="mt-9 flex flex-wrap gap-3">
          <button type="button" onClick={retry} className="inline-flex h-12 items-center rounded-full bg-primary px-6 font-medium hover:bg-primary-hover">
            Попробовать снова
          </button>
          {/* eslint-disable-next-line @next/next/no-html-link-for-pages -- a full load leaves the broken client state behind */}
          <a href="/" className="inline-flex h-12 items-center rounded-full border border-white/25 px-6 font-medium hover:border-white hover:bg-white/5">
            На главную
          </a>
        </div>
      </main>
    </div>
  );
}
