"use client";

import { Briefcase, Building2, LayoutDashboard, ListChecks, Search, Send, Target } from "lucide-react";
import type { ReactNode } from "react";
import { CabinetShell, type NavItem } from "@/components/layout/CabinetShell";

const NAV: NavItem[] = [
  { href: "/employer", label: "Главная", icon: <LayoutDashboard className="size-4.5" />, exact: true },
  { href: "/employer/needs", label: "Потребности и подборки", icon: <Target className="size-4.5" /> },
  { href: "/employer/candidates", label: "Поиск по базе", icon: <Search className="size-4.5" /> },
  { href: "/employer/invitations", label: "Приглашения", icon: <Send className="size-4.5" /> },
  { href: "/employer/vacancies", label: "Вакансии и отклики", icon: <Briefcase className="size-4.5" /> },
  { href: "/employer/tasks", label: "Регулярные задания", icon: <ListChecks className="size-4.5" /> },
  { href: "/employer/company", label: "Компания", icon: <Building2 className="size-4.5" /> },
];

export default function EmployerLayout({ children }: { children: ReactNode }) {
  return (
    <CabinetShell role="employer" nav={NAV}>
      {children}
    </CabinetShell>
  );
}
