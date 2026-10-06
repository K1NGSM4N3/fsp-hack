import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Providers } from "../providers";
import { AdminShell } from "./AdminShell";

export const metadata: Metadata = {
  title: "Кабинет модератора",
  robots: { index: false, follow: false },
};

export default function AdminLayout({ children }: { children: ReactNode }) {
  return (
    <Providers>
      <AdminShell>{children}</AdminShell>
    </Providers>
  );
}
