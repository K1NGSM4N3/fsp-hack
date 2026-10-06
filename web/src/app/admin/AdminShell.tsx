"use client";

import { LayoutDashboard, ScrollText, ShieldAlert, Video } from "lucide-react";
import type { ReactNode } from "react";
import { CabinetShell, type NavItem } from "@/components/layout/CabinetShell";
import { useOverview } from "@/features/admin/hooks";

export function AdminShell({ children }: { children: ReactNode }) {
  const o = useOverview().data;
  const antifake = o ? o.antifake.companies + o.antifake.vacancies + o.antifake.invitations + o.antifake.complaints : 0;
  const nav: NavItem[] = [
    { href: "/admin", label: "Обзор", icon: <LayoutDashboard className="size-4.5" />, exact: true, group: "Модерация" },
    { href: "/admin/proctoring", label: "Прокторинг", icon: <Video className="size-4.5" />, badge: o?.proctoring.needs_review, group: "Модерация" },
    { href: "/admin/antifake", label: "Антифейк и жалобы", icon: <ShieldAlert className="size-4.5" />, badge: antifake, group: "Модерация" },
    { href: "/admin/log", label: "Журнал", icon: <ScrollText className="size-4.5" />, group: "Аудит" },
  ];
  return (
    <CabinetShell role="admin" nav={nav}>
      {children}
    </CabinetShell>
  );
}
