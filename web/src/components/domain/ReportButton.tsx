"use client";

import { Flag } from "lucide-react";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { ReportInput, ReportReason } from "@/api/types";
import { Button, ChipSelect, Dialog, Field, Textarea, useToast } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { useCreateReport } from "@/features/reports/hooks";

const REASONS: { value: ReportReason; label: string }[] = [
  { value: "fake_vacancy", label: "Фиктивное предложение" },
  { value: "salary_mismatch", label: "Зарплата не соответствует" },
  { value: "spam", label: "Спам" },
  { value: "abuse", label: "Некорректное поведение" },
  { value: "other", label: "Другое" },
];

export function ReportButton({ targetKind, targetId }: { targetKind: ReportInput["target_kind"]; targetId: string }) {
  const me = useMe();
  const toast = useToast();
  const report = useCreateReport();
  const [open, setOpen] = useState(false);
  const [reason, setReason] = useState<ReportReason[]>([]);
  const [comment, setComment] = useState("");
  if (!me.data) return null;
  const picked = reason[0];
  const close = () => {
    setOpen(false);
    setReason([]);
    setComment("");
  };
  return (
    <>
      <Button variant="ghost" size="sm" icon={<Flag className="size-4" aria-hidden />} onClick={() => setOpen(true)}>
        Пожаловаться
      </Button>
      <Dialog
        open={open}
        onClose={close}
        title="Пожаловаться"
        description="Жалобы помогают находить фиктивные вакансии и недобросовестных работодателей. Компания не узнает, кто пожаловался."
        footer={
          <>
            <Button variant="ghost" onClick={close}>
              Отмена
            </Button>
            <Button
              variant="danger"
              disabled={!picked}
              loading={report.isPending}
              onClick={() =>
                picked &&
                report.mutate(
                  { target_kind: targetKind, target_id: targetId, reason: picked, comment: comment.trim() || null },
                  {
                    onSuccess: () => {
                      close();
                      toast.success("Жалоба отправлена");
                    },
                    onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось отправить жалобу"),
                  },
                )
              }
            >
              Отправить
            </Button>
          </>
        }
      >
        <div className="space-y-5">
          <Field label="Что не так" required>
            <ChipSelect<ReportReason> single ariaLabel="Причина жалобы" options={REASONS} value={reason} onChange={setReason} />
          </Field>
          <Field label="Подробности" optional>
            <Textarea rows={3} value={comment} onChange={(e) => setComment(e.target.value)} maxLength={2000} />
          </Field>
        </div>
      </Dialog>
    </>
  );
}
