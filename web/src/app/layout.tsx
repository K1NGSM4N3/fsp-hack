import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";
import { Inter, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import { Providers } from "./providers";

const inter = Inter({ subsets: ["latin", "cyrillic"], variable: "--font-inter", display: "swap" });
const jetbrains = JetBrains_Mono({ subsets: ["latin", "cyrillic"], variable: "--font-jetbrains", display: "swap" });

export const metadata: Metadata = {
  title: { default: "ФСП Карьера — подбор ИТ-специалистов", template: "%s · ФСП Карьера" },
  description:
    "Работодатель сам приходит к кандидату с зарплатной вилкой. Категория подтверждена адаптивным тестом, достижения Федерации спортивного программирования повышают позицию.",
};

export const viewport: Viewport = {
  themeColor: "#402FFF",
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="ru" className={`${inter.variable} ${jetbrains.variable} h-full antialiased`}>
      <body className="flex min-h-full flex-col">
        <a href="#main" className="sr-only z-50 rounded-control bg-primary px-4 py-2 text-white focus:not-sr-only focus:fixed focus:top-3 focus:left-3">
          Перейти к содержимому
        </a>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
