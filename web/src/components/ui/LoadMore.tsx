"use client";

import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/cn";
import { Button } from "./Button";

interface Pager {
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  fetchNextPage: () => Promise<unknown>;
}

/**
 * «Показать ещё» under a keyset-paginated list. The running count sits in a polite live region, so a screen reader hears
 * what the click added; when the last page arrives the button gives way to «Показаны все», which takes the focus the
 * button had instead of dropping it to the page.
 */
export function LoadMore({ pager, loaded, noun, className }: { pager: Pager; loaded: number; noun: string; className?: string }) {
  const [clicked, setClicked] = useState(false);
  const done = useRef<HTMLParagraphElement>(null);
  const finished = clicked && !pager.hasNextPage;
  useEffect(() => {
    if (finished) done.current?.focus();
  }, [finished]);
  if (!pager.hasNextPage && !clicked) return null;
  return (
    <div className={cn("mt-4 flex flex-col items-center gap-2", className)}>
      {pager.hasNextPage ? (
        <Button
          variant="secondary"
          loading={pager.isFetchingNextPage}
          onClick={() => {
            setClicked(true);
            void pager.fetchNextPage();
          }}
        >
          Показать ещё
        </Button>
      ) : (
        <p ref={done} tabIndex={-1} className="rounded text-sm text-ink-subtle">
          Показаны все: {loaded}
        </p>
      )}
      <p aria-live="polite" className="sr-only">
        {clicked ? `${noun}: показано ${loaded}` : ""}
      </p>
    </div>
  );
}
