import { MousePointerClick } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { BusinessHero } from "@/components/site/business/BusinessHero";
import { FirstVacancy } from "@/components/site/business/FirstVacancy";
import { InboxCompare } from "@/components/site/business/InboxCompare";
import { KineticLines } from "@/components/site/business/KineticLines";
import { ProductTour } from "@/components/site/business/ProductTour";
import { RulesReceipt } from "@/components/site/business/RulesReceipt";
import { Container, Eyebrow, Section, TextLink } from "@/components/site/primitives";

export const metadata: Metadata = {
  title: { absolute: "Компаниям · ФСП Таланты" },
  description:
    "Вставьте текст вакансии и получите подборку ИТ-специалистов, чей грейд подтверждён тестом. У каждого написано, чем подходит и чего не хватает. Зарплата в первом сообщении, контакты после согласия.",
};

export default function BusinessPage() {
  return (
    <MotionRoot>
      <BusinessHero />

      <Section tone="canvas" id="compare" className="scroll-mt-24">
        <Container>
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-[1.1fr_0.9fr] lg:items-end lg:gap-16">
            <div>
              <div data-reveal="fade">
                <Eyebrow>Входящие</Eyebrow>
              </div>
              <h2 data-split className="display-2c mt-5 text-balance text-ink">
                Сдвиньте шторку и сравните два понедельника
              </h2>
            </div>
            <p data-reveal className="text-lg leading-relaxed text-ink-muted">
              Слева то, что приходит после публикации вакансии на обычной площадке. Справа ваша подборка: только люди, сдавшие тест на нужный грейд, и у каждого написано, чем подходит и чего не хватает.
            </p>
          </div>
          <div data-reveal className="mt-12 sm:mt-14">
            <InboxCompare />
          </div>
          <p className="mt-4 flex items-center gap-2 text-sm text-ink-subtle">
            <MousePointerClick className="size-4 shrink-0" aria-hidden />
            Тяните шторку, нажмите в нужном месте или двигайте ползунок стрелками на клавиатуре
          </p>
        </Container>
      </Section>

      <section id="tour" className="relative overflow-x-clip bg-white">
        <ProductTour />
      </section>

      <Section tone="night" className="overflow-x-clip">
        <KineticLines />
      </Section>

      <Section tone="canvas" id="rules" className="overflow-x-clip">
        <Container className="grid grid-cols-1 gap-14 lg:grid-cols-[1fr_minmax(0,26rem)] lg:gap-20">
          <div className="lg:sticky lg:top-32 lg:self-start">
            <div data-reveal="fade">
              <Eyebrow>Условия контакта</Eyebrow>
            </div>
            <h2 data-split className="display-2c mt-5 text-balance text-ink">
              Правила для всех компаний
            </h2>
            <p data-reveal className="mt-6 max-w-xl text-lg leading-relaxed text-ink-muted">
              Они одинаковые для стартапа и для банка. Специалисты знают их заранее, поэтому спокойно открывают приглашения и отвечают по делу.
            </p>
            <div data-reveal className="mt-10 max-w-xl border-t border-line-strong pt-6">
              <p className="font-semibold text-ink">Обычные вакансии тоже работают</p>
              <p className="mt-2 leading-relaxed text-ink-muted">
                Опубликуйте вакансию, и специалисты смогут откликнуться сами. Контакты откроются вместе с откликом, без отдельного приглашения.
              </p>
              <TextLink href="/vacancies" className="mt-4">
                Посмотреть вакансии
              </TextLink>
            </div>
          </div>
          <RulesReceipt />
        </Container>
      </Section>

      <section className="relative isolate overflow-hidden bg-primary py-24 text-white sm:py-32 lg:py-40">
        <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
          <div className="bg-grid-night mask-radial absolute inset-0 opacity-70" />
          <div className="absolute -top-[22rem] left-1/2 size-[56rem] -translate-x-1/2 rounded-full bg-[radial-gradient(circle,rgb(169_163_255/0.45),transparent_62%)]" />
        </div>
        <FirstVacancy />
      </section>
    </MotionRoot>
  );
}
