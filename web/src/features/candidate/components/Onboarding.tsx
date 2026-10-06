"use client";

import { ArrowRight } from "lucide-react";
import type { OnboardingStep } from "@/api/types";
import { ButtonLink, Card, Stepper, type Step } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";

const ORDER: OnboardingStep[] = ["profile", "survey", "assessment", "done"];

const NEXT: Record<Exclude<OnboardingStep, "done">, { title: string; text: string; href: string; cta: string }> = {
  profile: {
    title: "Начните с профиля",
    text: "Заполните форму или загрузите PDF-резюме — поля распознаются автоматически. Контакты не видны компаниям до вашего согласия.",
    href: "/candidate/profile",
    cta: "Заполнить профиль",
  },
  survey: {
    title: "Пройдите короткий опрос",
    text: "Направление, специализация, стек и грейд, который хотите подтвердить. Около 3 минут.",
    href: "/candidate/survey",
    cta: "Перейти к опросу",
  },
  assessment: {
    title: "Подтвердите грейд тестом",
    text: "Адаптивный тест подбирает задания под ваш уровень. Категорию даёт только тест — не резюме.",
    href: "/candidate/assessment",
    cta: "Перейти к тесту",
  },
};

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

export function OnboardingCard() {
  const step = useOnboardingStep();
  if (!step || step === "done") return null;
  const next = NEXT[step];
  const left = 4 - ORDER.indexOf(step);
  return (
    <Card padding="lg" className="mb-6 border-primary-200">
      <div className="mb-6 flex flex-col gap-1">
        <span className="font-mono text-xs tracking-wider text-primary uppercase">Путь к категории · осталось шагов: {left}</span>
        <h2 className="text-xl font-semibold tracking-tight text-ink">{next.title}</h2>
        <p className="max-w-2xl text-sm text-ink-muted">{next.text}</p>
      </div>
      <Stepper steps={onboardingSteps(step)} ariaLabel="Путь к категории" />
      <div className="mt-6">
        <ButtonLink href={next.href} icon={<ArrowRight className="size-4" aria-hidden />}>
          {next.cta}
        </ButtonLink>
      </div>
    </Card>
  );
}
