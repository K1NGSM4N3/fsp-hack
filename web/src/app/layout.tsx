import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";
import { JetBrains_Mono, Onest } from "next/font/google";
import "./globals.css";
import { MotionFlag } from "@/components/motion/MotionFlag";
import { Providers } from "./providers";

const onest = Onest({ subsets: ["latin", "cyrillic"], variable: "--font-onest", display: "swap" });
const jetbrains = JetBrains_Mono({ subsets: ["latin", "cyrillic"], variable: "--font-jetbrains", display: "swap" });

export const metadata: Metadata = {
  title: { default: "ФСП Таланты — найм, в котором первым пишет работодатель", template: "%s · ФСП Таланты" },
  description:
    "Платформа подбора ИТ-специалистов Федерации спортивного программирования России. Уровень подтверждает адаптивный тест, работодатель приходит сам — с зарплатной вилкой в первом сообщении.",
};

export const viewport: Viewport = {
  themeColor: "#402FFF",
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="ru" className={`${onest.variable} ${jetbrains.variable} h-full antialiased`} suppressHydrationWarning>
      <head>
        <MotionFlag />
      </head>
      <body className="flex min-h-full flex-col">
        <a href="#main" className="sr-only z-[60] rounded-control bg-primary px-4 py-2 text-white focus:not-sr-only focus:fixed focus:top-3 focus:left-3">
          Перейти к содержимому
        </a>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
