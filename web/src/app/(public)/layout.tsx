import type { ReactNode } from "react";
import { PublicFooter, PublicHeader } from "@/components/layout/PublicChrome";

export default function PublicLayout({ children }: { children: ReactNode }) {
  return (
    <>
      <PublicHeader />
      <main id="main" className="flex flex-1 flex-col">
        {children}
      </main>
      <PublicFooter />
    </>
  );
}
