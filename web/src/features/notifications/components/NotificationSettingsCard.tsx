"use client";

import { isApiError } from "@/api/client";
import { Card, CardHeader, ErrorAlert, SkeletonCard, Switch, useToast } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { useNotificationSettings, useUpdateNotificationSettings } from "../hooks";

/** In-app items always arrive; here the user picks which of them also come by email. */
export function NotificationSettingsCard() {
  const q = useNotificationSettings();
  const me = useMe();
  const save = useUpdateNotificationSettings();
  const toast = useToast();
  if (q.isPending) return <SkeletonCard lines={4} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const adult = me.data?.candidate?.age.bracket !== "minor";
  const items = q.data.items.filter((s) => s.kind !== "guardian_decision" || !adult);
  return (
    <Card id="notifications" className="scroll-mt-24">
      <CardHeader
        title="Уведомления"
        description="В кабинете под колокольчиком появляется всё. Ниже — что дублировать на почту. Отписаться от вида писем можно и по ссылке в конце любого письма."
      />
      <div className="divide-y divide-line">
        {items.map((s) => (
          <Switch
            key={s.kind}
            className="py-4 first:pt-0 last:pb-0"
            checked={s.email}
            disabled={save.isPending}
            label={s.title}
            description={s.description}
            onChange={(enabled) =>
              save.mutate(
                { email: [{ kind: s.kind, enabled }] },
                {
                  onSuccess: () => toast.success(enabled ? "Письма включены" : "Письма отключены", s.title),
                  onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось сохранить"),
                },
              )
            }
          />
        ))}
      </div>
    </Card>
  );
}
