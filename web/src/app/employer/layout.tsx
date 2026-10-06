import type { ReactNode } from "react";
import { Providers } from "../providers";
import { EmployerShell } from "./EmployerShell";

export default function EmployerLayout({ children }: { children: ReactNode }) {
  return (
    <Providers>
      <EmployerShell>{children}</EmployerShell>
    </Providers>
  );
}
