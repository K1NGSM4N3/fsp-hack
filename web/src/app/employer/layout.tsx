"use client";

import { Briefcase, Building2, LayoutDashboard, ListChecks, Search, Send, ShieldCheck, Target, Webhook } from "lucide-react";
import type { ReactNode } from "react";
import { CabinetShell, type NavItem } from "@/components/layout/CabinetShell";

const NAV: NavItem[] = [
  { href: "/employer", label: "Главная", icon: <LayoutDashboard className="size-4.5" />, exact: true, group: "Обзор" },
  { href: "/employer/needs", label: "Потребности и подборки", icon: <Target className="size-4.5" />, group: "Подбор" },
  { href: "/employer/candidates", label: "Поиск по базе", icon: <Search className="size-4.5" />, group: "Подбор" },
  { href: "/employer/invitations", label: "Приглашения", icon: <Send className="size-4.5" />, group: "Подбор" },
  { href: "/employer/vacancies", label: "Вакансии и отклики", icon: <Briefcase className="size-4.5" />, group: "Найм" },
  { href: "/employer/tasks", label: "Регулярные задания", icon: <ListChecks className="size-4.5" />, group: "Найм" },
  { href: "/employer/integrations", label: "Интеграции", icon: <Webhook className="size-4.5" />, group: "Аккаунт" },
  { href: "/employer/company", label: "Компания", icon: <Building2 className="size-4.5" />, group: "Аккаунт" },
  { href: "/employer/trust", label: "Доверие и квота", icon: <ShieldCheck className="size-4.5" />, group: "Аккаунт" },
];

export default function EmployerLayout({ children }: { children: ReactNode }) {
  return (
    <CabinetShell role="employer" nav={NAV}>
      {children}
    </CabinetShell>
  );
}
