import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Providers } from "../providers";
import { EmployerShell } from "./EmployerShell";

export const metadata: Metadata = { robots: { index: false, follow: false } };

export default function EmployerLayout({ children }: { children: ReactNode }) {
  return (
    <Providers>
      <EmployerShell>{children}</EmployerShell>
    </Providers>
  );
}
