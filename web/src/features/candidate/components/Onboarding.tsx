"use client";

import { ArrowRight } from "lucide-react";
import type { OnboardingStep } from "@/api/types";
import { ButtonLink, Card, Stepper, type Step } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { useAssessments, useInvitationSummary, usePrivacy } from "@/features/candidate/hooks";
import { pluralize } from "@/lib/format";

const ORDER: OnboardingStep[] = ["profile", "survey", "assessment", "done"];

export function onboardingSteps(step: OnboardingStep): Step[] {
  const idx = ORDER.indexOf(step);
  const status = (i: number): Step["status"] => (i < idx || step === "done" ? "done" : i === idx ? "current" : "upcoming");
  return [
    { key: "profile", label: "Профиль", hint: "форма или PDF", href: "/candidate/profile", status: status(0) },
    { key: "survey", label: "Опрос", hint: "стек и грейд", href: "/candidate/survey", status: status(1) },
    { key: "assessment", label: "Тест", hint: "адаптивный", href: "/candidate/assessment", status: status(2) },
    { key: "category", label: "Категория", hint: "видна компаниям", href: "/candidate", status: step === "done" ? "done" : "upcoming" },
  ];
}

/** A candidate who already has a category is past onboarding even if older data lacks a survey. */
export function useOnboardingStep(): OnboardingStep | null {
  const candidate = useMe().data?.candidate;
  if (!candidate) return null;
  return candidate.category ? "done" : candidate.onboarding_step;
}

/** Compact stepper shown on profile, survey and test pages until the category is assigned. */
export function OnboardingStrip() {
  const step = useOnboardingStep();
  if (!step || step === "done") return null;
  return (
    <Card padding="sm" className="mb-6">
      <Stepper steps={onboardingSteps(step)} ariaLabel="Путь к категории" />
    </Card>
  );
}

/** Whether companies can find the profile right now, and what to do if not. */
export function useVisibility(): { visible: boolean; text: string } | null {
  const me = useMe();
  const privacy = usePrivacy();
  const age = me.data?.candidate?.age;
  if (!privacy.data) return null;
  if (age?.bracket === "minor" && age.guardian !== "granted") return { visible: false, text: "Компании пока не видят профиль: ждём согласия законного представителя." };
  if (!privacy.data.visible) return { visible: false, text: "Профиль скрыт от компаний — включить показ можно в настройках." };
  if (privacy.data.search_status === "closed") return { visible: false, text: "Поиск закрыт: компании не находят вас в подборках. Открыть его можно в настройках." };
  return { visible: true, text: "Видна компаниям — по ней вас находят." };
}

interface Next {
  eyebrow: string;
  title: string;
  text: string;
  href: string;
  cta: string;
}

function useNextStep(): Next | null {
  const step = useOnboardingStep();
  const summary = useInvitationSummary();
  const assessments = useAssessments();
  const visibility = useVisibility();
  if (!step) return null;
  const running = assessments.data?.find((s) => s.status === "in_progress");
  if (running) {
    return {
      eyebrow: "Тест не закончен",
      title: "Вернитесь к тесту",
      text: "Тест закроется и засчитается как неудачная попытка, если не ответить в течение 30 минут после окончания времени на задание.",
      href: `/candidate/assessment/${running.id}`,
      cta: "Продолжить тест",
    };
  }
  if (step === "profile") {
    return {
      eyebrow: "Шаг 1 из 3 · профиль",
      title: "Заполните профиль",
      text: "Форма или PDF-резюме — поля распознаются сами. Контакты компании увидят, только если вы примете приглашение.",
      href: "/candidate/profile",
      cta: "Заполнить профиль",
    };
  }
  if (step === "survey" || step === "assessment") {
    return {
      eyebrow: "Шаг 2 из 3 · тест",
      title: "Подтвердите грейд тестом",
      text:
        step === "survey"
          ? "Сначала три минуты на опрос: специализация, стек и грейд, который хотите подтвердить. Потом адаптивный тест на 10–25 минут — категорию даёт только он."
          : "Опрос пройден, грейд для теста выбран. Адаптивный тест займёт 10–25 минут — категорию даёт только он.",
      href: step === "survey" ? "/candidate/survey" : "/candidate/assessment",
      cta: step === "survey" ? "Пройти опрос и тест" : "Пройти тест",
    };
  }
  const fresh = summary.data?.by_status.find((c) => c.status === "sent")?.count ?? 0;
  if (fresh > 0) {
    return {
      eyebrow: "Шаг 3 из 3 · приглашения",
      title: `${pluralize(fresh, ["новое приглашение ждёт", "новых приглашения ждут", "новых приглашений ждут"])} ответа`,
      text: "Компания уже назвала вилку и задачу. Контакты она получит, только если вы примете приглашение.",
      href: "/candidate/invitations",
      cta: "Открыть приглашения",
    };
  }
  if (visibility && !visibility.visible) {
    return {
      eyebrow: "Шаг 3 из 3 · приглашения",
      title: "Компании пока не видят профиль",
      text: visibility.text,
      href: "/candidate/settings#visibility",
      cta: "Открыть настройки показа",
    };
  }
  return {
    eyebrow: "Шаг 3 из 3 · приглашения",
    title: "Ждите приглашений",
    text: "Компании находят вас по подтверждённой категории и пишут первыми, сразу с вилкой. Пока ждёте — посмотрите вакансии и откликнитесь сами.",
    href: "/candidate/vacancies",
    cta: "Смотреть вакансии",
  };
}

/** The one thing to do next, with the one button on the home page that does it. */
export function NextStepCard() {
  const step = useOnboardingStep();
  const next = useNextStep();
  if (!step || !next) return null;
  return (
    <Card padding="lg" className="mb-6 border-primary-200" aria-labelledby="next-step-title">
      <div className="flex flex-col gap-1">
        <span className="font-mono text-xs tracking-wider text-primary uppercase">{next.eyebrow}</span>
        <h2 id="next-step-title" className="text-xl font-semibold tracking-tight text-ink">
          {next.title}
        </h2>
        <p className="max-w-2xl text-sm text-ink-muted">{next.text}</p>
      </div>
      {step !== "done" && (
        <div className="mt-6">
          <Stepper steps={onboardingSteps(step)} ariaLabel="Путь к категории" />
        </div>
      )}
      <div className="mt-6">
        <ButtonLink href={next.href} icon={<ArrowRight className="size-4" aria-hidden />}>
          {next.cta}
        </ButtonLink>
      </div>
    </Card>
  );
}
