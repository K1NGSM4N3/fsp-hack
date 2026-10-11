"use client";

import Link from "next/link";
import type { ReactNode } from "react";
import type { ConsentDocument } from "@/api/types";
import { Button, Dialog } from "@/components/ui";

interface ConsentDialogProps {
  doc: ConsentDocument | null;
  onClose: () => void;
  onAccept: () => void;
  loading?: boolean;
  title?: ReactNode;
  lead?: ReactNode;
  acceptLabel?: string;
}

/** The full text of a consent document with the button that grants it. */
export function ConsentDialog({ doc, onClose, onAccept, loading, title, lead, acceptLabel = "Даю согласие" }: ConsentDialogProps) {
  return (
    <Dialog
      open={doc !== null}
      onClose={onClose}
      title={title ?? doc?.title ?? ""}
      description={doc ? `${title ? `${doc.title}, ` : ""}редакция ${doc.version}` : undefined}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Отмена
          </Button>
          <Button loading={loading} onClick={onAccept}>
            {acceptLabel}
          </Button>
        </>
      }
    >
      {doc && (
        <div className="space-y-4">
          {lead && <p className="text-sm text-ink">{lead}</p>}
          <div className="max-h-[45dvh] overflow-y-auto rounded-control border border-line bg-canvas p-4 text-sm leading-relaxed whitespace-pre-line text-ink-muted" data-lenis-prevent>
            {doc.body}
          </div>
          <p className="text-xs text-ink-subtle">
            Отозвать согласие можно в любой момент здесь же, в настройках. Как мы обращаемся с данными —{" "}
            <Link href={`/privacy#${doc.kind}`} target="_blank" className="link-inline">
              в политике обработки
            </Link>
            .
          </p>
        </div>
      )}
    </Dialog>
  );
}
