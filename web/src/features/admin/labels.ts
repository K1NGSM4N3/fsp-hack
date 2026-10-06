import type { ModerationAction, ModerationTarget, QueueFilter, ReviewDecision, TrustAction, TrustItemKind, VideoState } from "@/api/admin";
import type { BadgeTone } from "@/components/ui";

export const DECISION_LABEL: Record<ReviewDecision, string> = {
  confirm: "Подтвердить результат",
  annul: "Аннулировать результат",
  retest: "Назначить пересдачу с видеозаписью",
};

export const DECISION_DONE: Record<ReviewDecision, { text: string; tone: BadgeTone }> = {
  confirm: { text: "Подтверждён", tone: "success" },
  annul: { text: "Аннулирован", tone: "danger" },
  retest: { text: "Пересдача с видео", tone: "warning" },
};

/** What a decision does, said before the moderator presses the button. */
export const DECISION_EFFECT: Record<ReviewDecision, string> = {
  confirm: "Отметка «требует подтверждения» снимается. Работодатели увидят у категории «Результат подтверждён».",
  annul:
    "Категория, которую дал этот тест, снимается, и возвращается прежняя — или категории не будет. Навыки, которые подтвердил только этот тест, теряют отметку «по тесту». Тест считается неудачной попыткой: этот грейд закрыт на 30 дней от решения.",
  retest:
    "Результат аннулируется так же, но пересдать этот тест можно сразу и вне обычных сроков — только с включённой камерой. Тест на этот грейд и выше без камеры не начнётся, пока не будет пройден тест с видеозаписью.",
};

export const QUEUE_FILTER_LABEL: Record<QueueFilter, string> = {
  needs_review: "Требуют подтверждения",
  remarks: "С замечаниями",
  video: "Все с видеозаписью",
};

export const VIDEO_LABEL: Record<VideoState, { text: string; tone: BadgeTone }> = {
  kept: { text: "Есть запись", tone: "primary" },
  purged: { text: "Удалена по сроку", tone: "muted" },
  declined: { text: "Без камеры", tone: "neutral" },
  off: { text: "Прокторинг выключен", tone: "muted" },
};

export const ITEM_KIND_LABEL: Record<TrustItemKind, string> = {
  company: "Компания",
  vacancy: "Вакансия",
  invitation: "Приглашение",
  complaint: "Жалоба",
};

export const TRUST_ACTION_LABEL: Record<TrustAction, string> = {
  dismiss: "Снять: нарушений нет",
  unpublish: "Снять вакансию с публикации",
  freeze: "Приостановить приглашения",
  restore: "Вернуть",
};

/** What the action does to this kind of item; the company reads the comment on its trust page. */
export function trustActionEffect(action: TrustAction, kind: TrustItemKind): string {
  switch (action) {
    case "dismiss":
      return kind === "complaint"
        ? "Жалоба закрывается как необоснованная и перестаёт снижать квоту компании."
        : kind === "company"
          ? "Компания уходит из очереди и вернётся при новой жалобе или новой отметке. Квота не меняется."
          : "Отметка снимается и больше не снижает квоту компании.";
    case "unpublish":
      return "Вакансия закрывается и пропадает из публичного списка; компания не сможет опубликовать её снова, пока вы её не вернёте. Жалобы на эту вакансию закрываются как подтверждённые.";
    case "freeze":
      return "Квота приглашений компании — 0 на выбранный срок. Жалобы на компанию и её приглашения закрываются как подтверждённые.";
    case "restore":
      return kind === "company"
        ? "Приостановка снимается досрочно, квота возвращается по обычному правилу."
        : "Вакансия снова публикуется, её отметки перестают снижать квоту.";
  }
}

export const ACTION_LABEL: Record<ModerationAction, string> = {
  confirm_result: "Подтвердил результат",
  annul_result: "Аннулировал результат",
  order_retest: "Назначил пересдачу с видео",
  view_recording: "Посмотрел запись теста",
  dismiss: "Снял отметку или жалобу",
  unpublish_vacancy: "Снял вакансию",
  freeze_invitations: "Приостановил приглашения",
  restore: "Вернул",
};

export const ACTION_TONE: Record<ModerationAction, BadgeTone> = {
  confirm_result: "success",
  annul_result: "danger",
  order_retest: "warning",
  view_recording: "muted",
  dismiss: "neutral",
  unpublish_vacancy: "danger",
  freeze_invitations: "danger",
  restore: "primary",
};

export const TARGET_LABEL: Record<ModerationTarget, string> = {
  session: "Тест",
  vacancy: "Вакансия",
  invitation: "Приглашение",
  company: "Компания",
  complaint: "Жалоба",
};

export const EVENT_LABEL: Record<string, string> = {
  tab_hidden: "Ушёл со вкладки",
  window_blur: "Переключился на другое окно",
  fullscreen_exit: "Вышел из полноэкранного режима",
  paste: "Вставил текст в ответ",
  camera_lost: "Камера отключилась",
  recording_failed: "Запись остановилась",
};

/** A signed decimal the Russian way: «−2,41». */
export function lz(v: number | null | undefined, digits = 2): string {
  return typeof v === "number" ? v.toFixed(digits).replace("-", "−").replace(".", ",") : "—";
}
