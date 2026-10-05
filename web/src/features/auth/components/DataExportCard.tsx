"use client";

import { Download } from "lucide-react";
import { isApiError } from "@/api/client";
import { Button, Card, CardHeader, useToast } from "@/components/ui";
import { useExportMyData } from "@/features/auth/hooks";

/** Right of access (152-ФЗ, ст. 14): one JSON file with everything stored about the account. */
export function DataExportCard({ description }: { description: string }) {
  const exportData = useExportMyData();
  const toast = useToast();
  return (
    <Card>
      <CardHeader title="Ваши данные" description={description} />
      <Button
        variant="secondary"
        icon={<Download className="size-4" aria-hidden />}
        loading={exportData.isPending}
        onClick={() =>
          exportData.mutate(undefined, {
            onSuccess: () => toast.success("Файл с вашими данными сохранён"),
            onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось выгрузить данные"),
          })
        }
      >
        Скачать мои данные
      </Button>
    </Card>
  );
}
