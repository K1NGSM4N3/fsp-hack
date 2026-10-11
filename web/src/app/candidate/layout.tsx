import type { Metadata } from "next";
import type { ReactNode } from "react";
import { Providers } from "../providers";
import { CandidateShell } from "./CandidateShell";

export const metadata: Metadata = { robots: { index: false, follow: false } };

export default function CandidateLayout({ children }: { children: ReactNode }) {
  return (
    <Providers>
      <CandidateShell>{children}</CandidateShell>
    </Providers>
  );
}
