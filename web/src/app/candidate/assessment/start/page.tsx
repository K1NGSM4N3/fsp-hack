"use client";

import { Camera, Clock, Eye, PlayCircle, ShieldCheck, Video, VideoOff } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";
import { isApiError } from "@/api/client";
import type { Grade } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { Alert, Button, ButtonLink, Card, Checkbox, Dialog, EmptyState, ErrorAlert, PageHeader, PageSkeleton, Spinner } from "@/components/ui";
import { useCategory, useStartAssessment } from "@/features/candidate/hooks";
import { pluralize } from "@/lib/format";
import { GRADE_LABEL, GRADES } from "@/lib/labels";
import { proctor } from "../_proctoring/proctor";
import { CameraView, useProctor, useProctoringOptions } from "../_proctoring/ui";

function CameraPanel() {
  const p = useProctor();
  return (
    <Card padding="lg" className="flex flex-col">
      <div className="relative aspect-[4/3] w-full overflow-hidden rounded-control border border-line bg-canvas">
        {p.camera === "ready" ? (
          <CameraView stream={p.stream} label="Предпросмотр камеры" className="size-full" />
        ) : (
          <div className="flex size-full flex-col items-center justify-center gap-3 p-6 text-center">
            {p.camera === "starting" ? (
              <>
                <Spinner className="size-6 text-primary" />
                <p className="text-sm text-ink-muted">Ждём разрешения браузера…</p>
              </>
            ) : p.camera === "denied" ? (
              <>
                <VideoOff className="size-8 text-ink-faint" aria-hidden />
                <p className="max-w-xs text-sm text-ink-muted">
                  Доступ к камере запрещён. Разрешите его в настройках сайта — значок слева от адреса — или пройдите тест без камеры.
                </p>
                <Button variant="secondary" size="sm" onClick={() => void proctor.openCamera()}>
                  Попробовать снова
                </Button>
              </>
            ) : p.camera === "unavailable" ? (
              <>
                <VideoOff className="size-8 text-ink-faint" aria-hidden />
                <p className="max-w-xs text-sm text-ink-muted">Камера не найдена, или браузер не умеет записывать видео. Тест можно пройти без камеры.</p>
              </>
            ) : (
              <>
                <Camera className="size-8 text-ink-faint" aria-hidden />
                <p className="text-sm text-ink-muted">Камера выключена</p>
                <Button size="sm" icon={<Camera className="size-4" aria-hidden />} onClick={() => void proctor.openCamera()}>
                  Включить камеру
                </Button>
                <p className="text-xs text-ink-subtle">Браузер спросит разрешение</p>
              </>
            )}
          </div>
        )}
        {p.camera === "ready" && (
          <span className="absolute top-2 left-2 rounded-md bg-night/70 px-2 py-0.5 text-xs text-white">Предпросмотр · запись ещё не идёт</span>
        )}
      </div>
      <p className="mt-3 text-sm text-ink-muted">{p.camera === "ready" ? "Так вас будет видно на записи. Звук не записывается." : "Посмотрите, как вас видно, прежде чем начать."}</p>
    </Card>
  );
}

function Step() {
  const router = useRouter();
  const grade = useSearchParams().get("grade") as Grade | null;
  const category = useCategory();
  const options = useProctoringOptions();
  const start = useStartAssessment();
  const p = useProctor();
  const [consent, setConsent] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);
  const [docOpen, setDocOpen] = useState(false);
  const [error, setError] = useState<{ title: string; detail: string | null } | null>(null);

  useEffect(() => () => proctor.releasePreview(), []);

  if (category.isPending || options.isPending) return <PageSkeleton cards={2} />;
  if (category.isError) return <ErrorAlert error={category.error} onRetry={() => void category.refetch()} />;
  if (options.isError) return <ErrorAlert error={options.error} onRetry={() => void options.refetch()} />;

  const c = category.data;
  const spec = c.specialization;
  const valid = grade && GRADES.includes(grade) && c.available.some((o) => o.grade === grade);
  if (!spec || !grade || !valid) {
    return (
      <div className="mx-auto max-w-3xl">
        <PageHeader back={{ href: "/candidate/assessment", label: "Тестирование" }} title="Тест недоступен" />
        <EmptyState
          title="Этот тест сейчас недоступен"
          description="Выберите грейд на странице тестирования."
          action={<ButtonLink href="/candidate/assessment">К тестированию</ButtonLink>}
        />
      </div>
    );
  }

  const o = options.data;
  const doc = o.mode === "optional" ? (o.consent ?? null) : null;
  const go = (camera: boolean) => {
    setError(null);
    if (fullscreen && document.fullscreenEnabled && !document.fullscreenElement) {
      void document.documentElement.requestFullscreen().catch(() => undefined);
    }
    start.mutate(
      { specialization: spec, grade, proctoring: doc ? { camera, consent_document_id: camera ? doc.id : null } : null },
      {
        onSuccess: (s) => {
          proctor.begin(s.id, camera && Boolean(doc));
          if (!camera) proctor.releasePreview();
          router.push(`/candidate/assessment/${s.id}`);
        },
        onError: (e) => {
          if (document.fullscreenElement) void document.exitFullscreen().catch(() => undefined);
          setError(isApiError(e) ? { title: e.title, detail: e.detail } : { title: "Не удалось начать тест", detail: null });
        },
      },
    );
  };

  const header = (
    <PageHeader
      back={{ href: "/candidate/assessment", label: "Тестирование" }}
      eyebrow={<CategoryBadge category={{ specialization: spec, grade }} size="sm" />}
      title={doc ? "Видеозапись во время теста" : `Тест на ${GRADE_LABEL[grade]}`}
      description={doc ? "Камера помогает подтвердить, что тест пройден самостоятельно. Это по желанию: без камеры тест засчитывается так же." : undefined}
    />
  );

  if (!doc) {
    return (
      <div className="mx-auto max-w-3xl">
        {header}
        <Card padding="lg">
          {error && (
            <Alert tone="danger" title={error.title} className="mb-5">
              {error.detail}
            </Alert>
          )}
          <Button size="lg" icon={<PlayCircle className="size-5" aria-hidden />} loading={start.isPending} onClick={() => go(false)}>
            Начать тест
          </Button>
        </Card>
      </div>
    );
  }

  const days = pluralize(o.retention_days, ["день", "дня", "дней"]);
  const points = [
    { icon: <Video className="size-4" aria-hidden />, title: "Что записываем", text: "Только видео с камеры — без звука и без экрана, в низком качестве. От первого задания до последнего." },
    {
      icon: <Eye className="size-4" aria-hidden />,
      title: "Кто увидит",
      text: "Вы и администратор — только если результат придётся перепроверить. Работодатели видео не получают: им видна лишь отметка «Тест с видеозаписью».",
    },
    { icon: <Clock className="size-4" aria-hidden />, title: "Сколько храним", text: `${days} в зашифрованном виде, потом удаляем сами. Удалить раньше — отозвать согласие в настройках.` },
  ];
  const ready = p.camera === "ready";
  const blocker = !ready ? "Сначала включите камеру" : !consent ? "Отметьте согласие на видеозапись" : null;

  return (
    <div className="mx-auto max-w-5xl">
      {header}
      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.15fr)]">
        <CameraPanel />
        <Card padding="lg">
          <ul className="space-y-4">
            {points.map((pt) => (
              <li key={pt.title} className="flex gap-3">
                <span className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-md bg-primary-50 text-primary">{pt.icon}</span>
                <span className="text-sm">
                  <span className="block font-medium text-ink">{pt.title}</span>
                  <span className="text-ink-muted">{pt.text}</span>
                </span>
              </li>
            ))}
          </ul>
          <p className="mt-4 flex gap-2 rounded-control bg-canvas p-3 text-xs text-ink-muted">
            <ShieldCheck className="size-4 shrink-0 text-ink-subtle" aria-hidden />
            В любом тесте, с камерой и без, мы отмечаем уходы со вкладки и вставку текста в ответ. Работодатель видит только их число.
          </p>

          <div className="mt-5 space-y-3 border-t border-line pt-5">
            <Checkbox
              checked={consent}
              onChange={(e) => setConsent(e.target.checked)}
              label="Согласен(на) на видеозапись во время теста"
              description={
                <>
                  <button type="button" className="link-inline rounded" onClick={() => setDocOpen(true)}>
                    Текст согласия
                  </button>
                  {" · "}
                  <Link href="/privacy#proctoring_video" target="_blank" className="link-inline">
                    как мы храним запись
                  </Link>
                </>
              }
            />
            <Checkbox checked={fullscreen} onChange={(e) => setFullscreen(e.target.checked)} label="Открыть тест на весь экран — меньше отвлекает" />
          </div>

          {error && (
            <Alert tone="danger" title={error.title} className="mt-5">
              {error.detail}
            </Alert>
          )}

          <div className="mt-6 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
            <Button size="lg" icon={<Video className="size-5" aria-hidden />} disabled={Boolean(blocker)} loading={start.isPending} onClick={() => go(true)}>
              Начать с видеозаписью
            </Button>
            <Button size="lg" variant="secondary" disabled={start.isPending} onClick={() => go(false)}>
              Продолжить без камеры
            </Button>
          </div>
          <p className="mt-3 text-xs text-ink-subtle">
            {blocker ? `${blocker}. ` : ""}Без камеры у результата будет отметка «без видеозаписи» — её увидят работодатели.
          </p>
        </Card>
      </div>

      <Dialog
        open={docOpen}
        onClose={() => setDocOpen(false)}
        title={doc.title}
        description={`Версия ${doc.version}`}
        footer={
          <Button
            onClick={() => {
              setConsent(true);
              setDocOpen(false);
            }}
          >
            Согласен(на)
          </Button>
        }
      >
        <p className="text-sm leading-relaxed whitespace-pre-line text-ink-muted">{doc.body}</p>
      </Dialog>
    </div>
  );
}

export default function ProctoringStepPage() {
  return (
    <Suspense fallback={<PageSkeleton cards={2} />}>
      <Step />
    </Suspense>
  );
}
