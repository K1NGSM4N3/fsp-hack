import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";
import { JetBrains_Mono, Onest } from "next/font/google";
import "./globals.css";
import { MotionFlag } from "@/components/motion/MotionFlag";

const onest = Onest({ subsets: ["latin", "cyrillic"], variable: "--font-onest", display: "swap" });
// Mono is only for eyebrows, codes and sums: no preload, so it never competes with Onest, CSS and JS for the first
// bytes. Listing the weights in use makes Google serve a narrower variable file (52 KB instead of 68 KB for 3 subsets).
const jetbrains = JetBrains_Mono({ weight: ["400", "500", "600", "700"], subsets: ["latin", "cyrillic"], variable: "--font-jetbrains", display: "swap", preload: false });

export const metadata: Metadata = {
  title: { default: "ФСП Таланты: компании пишут первыми и сразу называют зарплату", template: "%s · ФСП Таланты" },
  description:
    "Платформа подбора ИТ-специалистов Федерации спортивного программирования России. Вы один раз подтверждаете уровень тестом, а компании пишут вам сами и сразу называют зарплату.",
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
        {children}
      </body>
    </html>
  );
}
