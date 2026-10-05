"use client";

import { Briefcase, ClipboardList, GraduationCap, LayoutDashboard, ListChecks, Mail, Settings, UserRound } from "lucide-react";
import type { ReactNode } from "react";
import { CabinetShell, type NavItem } from "@/components/layout/CabinetShell";
import { useReceivedInvitations } from "@/features/candidate/hooks";

export default function CandidateLayout({ children }: { children: ReactNode }) {
  const invitations = useReceivedInvitations();
  const fresh = invitations.data?.filter((i) => i.status === "sent").length ?? 0;
  const nav: NavItem[] = [
    { href: "/candidate", label: "Главная", icon: <LayoutDashboard className="size-4.5" />, exact: true, group: "Обзор" },
    { href: "/candidate/profile", label: "Профиль", icon: <UserRound className="size-4.5" />, group: "Категория" },
    { href: "/candidate/survey", label: "Опрос", icon: <ClipboardList className="size-4.5" />, group: "Категория" },
    { href: "/candidate/assessment", label: "Тестирование", icon: <GraduationCap className="size-4.5" />, group: "Категория" },
    { href: "/candidate/invitations", label: "Приглашения", icon: <Mail className="size-4.5" />, badge: fresh, group: "Работа" },
    { href: "/candidate/vacancies", label: "Вакансии и отклики", icon: <Briefcase className="size-4.5" />, group: "Работа" },
    { href: "/candidate/tasks", label: "Задания", icon: <ListChecks className="size-4.5" />, group: "Работа" },
    { href: "/candidate/settings", label: "Настройки", icon: <Settings className="size-4.5" />, group: "Аккаунт" },
  ];
  return (
    <CabinetShell role="candidate" nav={nav}>
      {children}
    </CabinetShell>
  );
}
