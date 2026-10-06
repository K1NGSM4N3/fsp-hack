"use client";

import { ArrowUpRight, LogOut, Menu, RotateCcw, X } from "lucide-react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useEffect, useState, type ReactNode } from "react";
import { isApiError, USE_MOCKS } from "@/api/client";
import type { Role } from "@/api/types";
import { Logo } from "@/components/brand/Logo";
import { Button, IconButton, PageSkeleton } from "@/components/ui";
import { useLogout, useMe } from "@/features/auth/hooks";
import { NotificationBell, NotificationWatcher } from "@/features/notifications/components/NotificationBell";
import { cn } from "@/lib/cn";
import { initials } from "@/lib/format";

export interface NavItem {
  href: string;
  label: string;
  icon: ReactNode;
  badge?: number;
  exact?: boolean;
  group?: string;
}

interface CabinetShellProps {
  role: Extract<Role, "candidate" | "employer">;
  nav: NavItem[];
  children: ReactNode;
}

const ROLE_HOME: Record<string, string> = { candidate: "/candidate", employer: "/employer" };
const ROLE_LABEL: Record<string, string> = { candidate: "Соискатель", employer: "Компания" };
const ROLE_CABINET: Record<string, string> = { candidate: "Кабинет соискателя", employer: "Кабинет компании" };

function isActive(item: NavItem, pathname: string) {
  return item.exact ? pathname === item.href : pathname === item.href || pathname.startsWith(`${item.href}/`);
}

function NavList({ nav, pathname, onNavigate }: { nav: NavItem[]; pathname: string; onNavigate?: () => void }) {
  const groups: { title: string | undefined; items: NavItem[] }[] = [];
  for (const item of nav) {
    const last = groups.at(-1);
    if (last && last.title === item.group) last.items.push(item);
    else groups.push({ title: item.group, items: [item] });
  }
  return (
    <div className="space-y-6">
      {groups.map((g, gi) => (
        <div key={g.title ?? gi}>
          {g.title && <p className="mb-2 px-3 font-mono text-xs tracking-[0.12em] text-night-faint uppercase">{g.title}</p>}
          <ul className="space-y-0.5">
            {g.items.map((item) => {
              const active = isActive(item, pathname);
              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    onClick={onNavigate}
                    aria-current={active ? "page" : undefined}
                    className={cn(
                      "group relative flex h-10 items-center gap-3 rounded-[10px] px-3 text-sm font-medium transition-colors",
                      active ? "bg-white/[0.08] text-white" : "text-night-muted hover:bg-white/[0.04] hover:text-white",
                    )}
                  >
                    <span aria-hidden className={cn("absolute top-2 bottom-2 left-0 w-[3px] rounded-full bg-primary-400 transition-opacity", active ? "opacity-100" : "opacity-0")} />
                    <span className={cn("shrink-0 transition-colors", active ? "text-primary-300" : "text-night-faint group-hover:text-night-muted")} aria-hidden>
                      {item.icon}
                    </span>
                    <span className="min-w-0 flex-1 truncate">{item.label}</span>
                    {item.badge ? (
                      <span className="tabular rounded-full bg-accent px-1.5 py-0.5 font-mono text-xs leading-none text-white" aria-label={`новых: ${item.badge}`}>
                        {item.badge}
                      </span>
                    ) : null}
                  </Link>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </div>
  );
}

function DemoReset() {
  if (!USE_MOCKS) return null;
  return (
    <div className="rounded-[12px] border border-dashed border-night-line p-3 text-xs text-night-muted">
      <p>Демо-режим: данные хранятся в этом браузере.</p>
      <button
        type="button"
        onClick={async () => {
          const m = await import("@/mocks/handlers");
          m.mockReset();
          // eslint-disable-next-line @next/next/no-location-assign-relative-destination -- full reload so no query cache survives the reset
          window.location.href = "/login";
        }}
        className="mt-1.5 inline-flex items-center gap-1 font-medium text-primary-300 hover:underline"
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
  const current = nav.find((item) => isActive(item, pathname));

  const userBlock = (
    <div className="flex items-center gap-3 rounded-[14px] border border-night-line bg-white/[0.03] p-2.5">
      <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-primary font-mono text-xs font-semibold text-white" aria-hidden>
        {displayName ? initials(displayName.split("@")[0] ?? displayName) : "…"}
      </span>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-white" title={displayName}>
          {displayName || "…"}
        </p>
        <p className="text-xs text-night-faint">{ROLE_LABEL[role]}</p>
      </div>
      <IconButton label="Выйти" onClick={onLogout} disabled={logout.isPending} className="text-night-muted hover:bg-white/10 hover:text-white">
        <LogOut className="size-4.5" aria-hidden />
      </IconButton>
    </div>
  );

  return (
    <div className="flex min-h-dvh flex-1 bg-canvas print:bg-white">
      <div className="relative hidden w-[17rem] shrink-0 bg-night lg:block print:hidden">
        <aside className="sticky top-0 flex h-dvh flex-col overflow-hidden px-4 py-5">
          <div aria-hidden className="pointer-events-none absolute -top-32 -left-24 size-72 rounded-full bg-primary/30 blur-[90px]" />
          <div className="relative mb-8 px-2">
            <Logo href={ROLE_HOME[role]} tone="dark" />
            <p className="mt-4 font-mono text-xs tracking-[0.12em] text-night-faint uppercase">{ROLE_CABINET[role]}</p>
          </div>
          <nav aria-label="Разделы кабинета" className="scrollbar-thin relative min-h-0 flex-1 overflow-y-auto">
            <NavList nav={nav} pathname={pathname} />
          </nav>
          <div className="relative space-y-3 pt-4">
            <DemoReset />
            <Link href="/" className="flex min-h-10 items-center justify-between rounded-[10px] px-3 text-xs text-night-faint transition-colors hover:text-white">
              Сайт ФСП Таланты
              <ArrowUpRight className="size-3.5" aria-hidden />
            </Link>
            {userBlock}
          </div>
        </aside>
      </div>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="sticky top-0 z-30 flex h-14 items-center justify-between border-b border-line bg-white/90 px-4 backdrop-blur-xl lg:hidden print:hidden">
          <Logo href={ROLE_HOME[role]} />
          <div className="flex items-center gap-1">
            <NotificationBell role={role} />
            <IconButton label="Открыть меню" onClick={() => setDrawer(true)} aria-expanded={drawer}>
              <Menu className="size-5" aria-hidden />
            </IconButton>
          </div>
        </header>
        <NotificationWatcher role={role} />
        <div className="sticky top-0 z-20 hidden h-14 items-center justify-between border-b border-line bg-white/80 px-10 backdrop-blur-xl lg:flex print:hidden">
          <p className="flex min-w-0 items-center gap-2 text-sm">
            <span className="text-ink-subtle">{ROLE_CABINET[role]}</span>
            <span aria-hidden className="text-line-strong">/</span>
            <span className="truncate font-medium text-ink">{current?.label ?? (pathname.endsWith("/notifications") ? "Уведомления" : "Раздел")}</span>
          </p>
          <div className="flex items-center gap-3">
            <span className="inline-flex items-center gap-2 rounded-full border border-line bg-white px-3 py-1 text-xs text-ink-muted">
              <span className="size-1.5 rounded-full bg-success" aria-hidden />
              {me.data?.role === "candidate" && me.data.candidate?.category ? "Категория подтверждена" : ROLE_LABEL[role]}
            </span>
            <NotificationBell role={role} />
          </div>
        </div>

        {drawer && (
          <div className="fixed inset-0 z-40 lg:hidden" role="dialog" aria-modal="true" aria-label="Меню">
            <button type="button" aria-label="Закрыть меню" className="absolute inset-0 animate-fade-in bg-night/50 backdrop-blur-sm" onClick={() => setDrawer(false)} />
            <div className="absolute inset-y-0 right-0 flex w-[min(20rem,88vw)] animate-[toast-in_420ms_var(--ease-out-expo)_both] flex-col bg-night px-4 py-4 shadow-pop">
              <div className="mb-6 flex items-center justify-between">
                <span className="font-mono text-xs tracking-[0.12em] text-night-faint uppercase">{ROLE_CABINET[role]}</span>
                <IconButton label="Закрыть меню" onClick={() => setDrawer(false)} className="text-night-muted hover:bg-white/10 hover:text-white">
                  <X className="size-5" aria-hidden />
                </IconButton>
              </div>
              <nav aria-label="Разделы кабинета" className="min-h-0 flex-1 overflow-y-auto">
                <NavList nav={nav} pathname={pathname} onNavigate={() => setDrawer(false)} />
              </nav>
              <div className="space-y-3 pt-4">
                <DemoReset />
                {userBlock}
              </div>
            </div>
          </div>
        )}

        <main id="main" className="mx-auto w-full max-w-[1200px] flex-1 px-4 py-6 sm:px-6 sm:py-8 lg:px-10 lg:py-10 print:max-w-none print:p-0">
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
            <div key={pathname} className="animate-rise">
              {children}
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
