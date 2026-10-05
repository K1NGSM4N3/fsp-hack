"use client";

import { LogOut, Menu, RotateCcw, X } from "lucide-react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useEffect, useState, type ReactNode } from "react";
import { isApiError, USE_MOCKS } from "@/api/client";
import type { Role } from "@/api/types";
import { Logo } from "@/components/brand/Logo";
import { Button, IconButton, PageSkeleton } from "@/components/ui";
import { useLogout, useMe } from "@/features/auth/hooks";
import { cn } from "@/lib/cn";

export interface NavItem {
  href: string;
  label: string;
  icon: ReactNode;
  badge?: number;
  exact?: boolean;
}

interface CabinetShellProps {
  role: Extract<Role, "candidate" | "employer">;
  nav: NavItem[];
  children: ReactNode;
}

const ROLE_HOME: Record<string, string> = { candidate: "/candidate", employer: "/employer" };
const ROLE_LABEL: Record<string, string> = { candidate: "Кандидат", employer: "Работодатель" };

function NavList({ nav, pathname, onNavigate }: { nav: NavItem[]; pathname: string; onNavigate?: () => void }) {
  return (
    <ul className="space-y-1">
      {nav.map((item) => {
        const active = item.exact ? pathname === item.href : pathname === item.href || pathname.startsWith(`${item.href}/`);
        return (
          <li key={item.href}>
            <Link
              href={item.href}
              onClick={onNavigate}
              aria-current={active ? "page" : undefined}
              className={cn(
                "flex h-10 items-center gap-3 rounded-control px-3 text-sm font-medium transition-colors",
                active ? "bg-primary-50 text-primary-strong" : "text-ink-muted hover:bg-canvas hover:text-ink",
              )}
            >
              <span className={cn("shrink-0", active ? "text-primary" : "text-ink-subtle")} aria-hidden>
                {item.icon}
              </span>
              <span className="min-w-0 flex-1 truncate">{item.label}</span>
              {item.badge ? (
                <span className="tabular rounded-full bg-accent-strong px-1.5 py-0.5 font-mono text-[11px] leading-none text-white" aria-label={`новых: ${item.badge}`}>
                  {item.badge}
                </span>
              ) : null}
            </Link>
          </li>
        );
      })}
    </ul>
  );
}

function DemoReset() {
  if (!USE_MOCKS) return null;
  return (
    <div className="rounded-control border border-dashed border-line-strong p-3 text-xs text-ink-muted">
      <p>Демо-режим: данные хранятся в этом браузере.</p>
      <button
        type="button"
        onClick={async () => {
          const m = await import("@/mocks/handlers");
          m.mockReset();
          // eslint-disable-next-line @next/next/no-location-assign-relative-destination -- full reload so no query cache survives the reset
          window.location.href = "/login";
        }}
        className="mt-1.5 inline-flex items-center gap-1 font-medium text-primary hover:underline"
      >
        <RotateCcw className="size-3.5" aria-hidden />
        Сбросить демо-данные
      </button>
    </div>
  );
}

export function CabinetShell({ role, nav, children }: CabinetShellProps) {
  const pathname = usePathname();
  const router = useRouter();
  const me = useMe();
  const logout = useLogout();
  const [drawer, setDrawer] = useState(false);

  const unauthorized = me.isError && isApiError(me.error) && me.error.status === 401;
  const wrongRole = me.data && me.data.role !== role;

  useEffect(() => {
    if (unauthorized) router.replace(`/login?next=${encodeURIComponent(pathname)}`);
    else if (wrongRole && me.data) router.replace(ROLE_HOME[me.data.role] ?? "/");
  }, [unauthorized, wrongRole, me.data, pathname, router]);

  const onLogout = () =>
    logout.mutate(undefined, {
      onSettled: () => router.replace("/login"),
    });

  const displayName = me.data?.company?.name ?? me.data?.email ?? "";

  const userBlock = (
    <div className="flex items-center justify-between gap-2 border-t border-line pt-4">
      <div className="min-w-0">
        <p className="truncate text-sm font-medium text-ink" title={displayName}>
          {displayName || "…"}
        </p>
        <p className="text-xs text-ink-subtle">{ROLE_LABEL[role]}</p>
      </div>
      <IconButton label="Выйти" onClick={onLogout} disabled={logout.isPending}>
        <LogOut className="size-4.5" aria-hidden />
      </IconButton>
    </div>
  );

  return (
    <div className="flex min-h-dvh flex-1 bg-canvas print:bg-white">
      <aside className="sticky top-0 hidden h-dvh w-64 shrink-0 flex-col border-r border-line bg-white px-4 py-5 lg:flex print:hidden">
        <Logo href={ROLE_HOME[role]} className="mb-8 px-2" />
        <nav aria-label="Разделы кабинета" className="min-h-0 flex-1 overflow-y-auto">
          <NavList nav={nav} pathname={pathname} />
        </nav>
        <div className="space-y-4">
          <DemoReset />
          {userBlock}
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="sticky top-0 z-30 flex h-14 items-center justify-between border-b border-line bg-white/95 px-4 backdrop-blur lg:hidden print:hidden">
          <Logo href={ROLE_HOME[role]} />
          <IconButton label="Открыть меню" onClick={() => setDrawer(true)} aria-expanded={drawer}>
            <Menu className="size-5" aria-hidden />
          </IconButton>
        </header>

        {drawer && (
          <div className="fixed inset-0 z-40 lg:hidden" role="dialog" aria-modal="true" aria-label="Меню">
            <button type="button" aria-label="Закрыть меню" className="absolute inset-0 bg-ink/40" onClick={() => setDrawer(false)} />
            <div className="absolute inset-y-0 right-0 flex w-[min(20rem,85vw)] animate-fade-in flex-col bg-white px-4 py-4 shadow-pop">
              <div className="mb-6 flex items-center justify-between">
                <span className="font-mono text-sm font-semibold text-ink-subtle">{ROLE_LABEL[role]}</span>
                <IconButton label="Закрыть меню" onClick={() => setDrawer(false)}>
                  <X className="size-5" aria-hidden />
                </IconButton>
              </div>
              <nav aria-label="Разделы кабинета" className="min-h-0 flex-1 overflow-y-auto">
                <NavList nav={nav} pathname={pathname} onNavigate={() => setDrawer(false)} />
              </nav>
              <div className="space-y-4">
                <DemoReset />
                {userBlock}
              </div>
            </div>
          </div>
        )}

        <main id="main" className="mx-auto w-full max-w-6xl flex-1 px-4 py-6 sm:px-6 sm:py-8 lg:px-10 print:max-w-none print:p-0">
          {me.isPending || unauthorized || wrongRole ? (
            <PageSkeleton />
          ) : me.isError ? (
            <div className="rounded-card border border-line bg-white p-6">
              <p className="font-medium text-ink">Не удалось загрузить кабинет</p>
              <p className="mt-1 text-sm text-ink-muted">Проверьте подключение и попробуйте ещё раз.</p>
              <Button className="mt-4" variant="secondary" onClick={() => void me.refetch()}>
                Повторить
              </Button>
            </div>
          ) : (
            children
          )}
        </main>
      </div>
    </div>
  );
}
