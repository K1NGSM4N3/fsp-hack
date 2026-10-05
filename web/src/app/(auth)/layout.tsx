import type { ReactNode } from "react";
import { BadgeCheck, Banknote, ShieldCheck, Trophy } from "lucide-react";
import { Logo, NodeLines } from "@/components/brand/Logo";

export default function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div className="grid min-h-dvh flex-1 lg:grid-cols-[1fr_minmax(0,32rem)]">
      <div className="flex flex-col bg-white">
        <header className="flex h-16 items-center px-4 sm:px-8">
          <Logo />
        </header>
        <main id="main" className="flex flex-1 items-start justify-center px-4 pt-6 pb-12 sm:px-8 sm:pt-12">
          <div className="w-full max-w-md">{children}</div>
        </main>
      </div>
      <aside className="relative hidden overflow-hidden bg-primary text-white lg:flex lg:flex-col lg:justify-center lg:px-12">
        <NodeLines className="pointer-events-none absolute -top-10 -right-20 h-96 w-[34rem] text-white/25" />
        <NodeLines className="pointer-events-none absolute -bottom-24 -left-24 h-96 w-[34rem] rotate-180 text-white/15" />
        <div className="relative">
          <p className="font-mono text-xs tracking-[0.12em] text-primary-100 uppercase">ФСП Карьера</p>
          <p className="mt-4 font-mono text-3xl leading-tight font-semibold tracking-tight">Категория по тесту. Вилка — в первом сообщении.</p>
          <ul className="mt-8 space-y-4 text-primary-100">
            <li className="flex gap-3">
              <BadgeCheck className="size-5 shrink-0 text-white" aria-hidden /> Адаптивный тест подтверждает специализацию и грейд
            </li>
            <li className="flex gap-3">
              <Banknote className="size-5 shrink-0 text-white" aria-hidden /> Работодатель указывает зарплату до начала общения
            </li>
            <li className="flex gap-3">
              <Trophy className="size-5 shrink-0 text-white" aria-hidden /> Достижения ФСП поднимают внутри категории
            </li>
            <li className="flex gap-3">
              <ShieldCheck className="size-5 shrink-0 text-white" aria-hidden /> Контакты — только после вашего согласия
            </li>
          </ul>
        </div>
      </aside>
    </div>
  );
}
