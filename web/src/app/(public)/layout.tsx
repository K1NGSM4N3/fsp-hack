import type { ReactNode } from "react";
import { SmoothScroll } from "@/components/motion/SmoothScroll";
import { JsonLd } from "@/components/seo/JsonLd";
import { SiteFooter } from "@/components/site/SiteFooter";
import { SiteHeader } from "@/components/site/SiteHeader";
import { siteGraph } from "@/lib/jsonld";

export default function PublicLayout({ children }: { children: ReactNode }) {
  return (
    <SmoothScroll>
      <JsonLd data={siteGraph()} />
      <SiteHeader />
      <main id="main" className="flex flex-1 flex-col">
        {children}
      </main>
      <SiteFooter />
    </SmoothScroll>
  );
}
