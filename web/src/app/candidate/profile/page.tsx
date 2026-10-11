"use client";

import { ArrowRight, Download, FileUp, PenLine } from "lucide-react";
import { useRouter } from "next/navigation";
import { useRef, useState } from "react";
import { isApiError, USE_MOCKS } from "@/api/client";
import { getProfilePdf } from "@/api/endpoints";
import { Alert, Button, ButtonLink, ErrorAlert, PageHeader, PageSkeleton, Tabs, tabPanelProps, useToast } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { useProfile } from "@/features/candidate/hooks";
import { OnboardingStrip } from "@/features/candidate/components/Onboarding";
import { ProfileForm, type ProfileFormHandle } from "@/features/candidate/components/ProfileForm";
import { ResumeUpload } from "@/features/candidate/components/ResumeUpload";

type Mode = "form" | "pdf";

function PdfButton() {
  const toast = useToast();
  const router = useRouter();
  const [busy, setBusy] = useState(false);
  const download = async () => {
    if (USE_MOCKS) {
      router.push("/candidate/profile/print");
      return;
    }
    setBusy(true);
    try {
      const blob = await getProfilePdf();
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "profile.pdf";
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      if (isApiError(e) && (e.status === 404 || e.status === 501)) router.push("/candidate/profile/print");
      else toast.error("Не удалось сформировать PDF", isApiError(e) ? (e.detail ?? undefined) : undefined);
    } finally {
      setBusy(false);
    }
  };
  return (
    <Button variant="secondary" loading={busy} icon={<Download className="size-4" aria-hidden />} onClick={download}>
      Скачать PDF-профиль
    </Button>
  );
}

export default function ProfilePage() {
  const profile = useProfile();
  const me = useMe();
  const [mode, setMode] = useState<Mode>("form");
  const [justSaved, setJustSaved] = useState(false);
  const formRef = useRef<ProfileFormHandle>(null);
  const step = me.data?.candidate?.onboarding_step;

  return (
    <div>
      <PageHeader title="Профиль" description="Резюме не определяет категорию — его использует подбор как дополнительный сигнал." actions={<PdfButton />} />
      <OnboardingStrip />
      {justSaved && step === "survey" && (
        <Alert
          tone="success"
          title="Профиль готов"
          className="mb-6"
          action={
            <ButtonLink href="/candidate/survey" size="sm" icon={<ArrowRight className="size-4" aria-hidden />}>
              Дальше: опрос
            </ButtonLink>
          }
        >
          Следующий шаг — короткий опрос о специализации и стеке.
        </Alert>
      )}

      <Tabs<Mode>
        ariaLabel="Способ заполнения"
        idPrefix="profile"
        value={mode}
        onChange={setMode}
        items={[
          {
            value: "form",
            label: (
              <span className="inline-flex items-center gap-2">
                <PenLine className="size-4" aria-hidden /> Заполнить вручную
              </span>
            ),
          },
          {
            value: "pdf",
            label: (
              <span className="inline-flex items-center gap-2">
                <FileUp className="size-4" aria-hidden /> Загрузить PDF-резюме
              </span>
            ),
          },
        ]}
        className="mb-6"
      />

      {profile.isPending ? (
        <PageSkeleton cards={2} />
      ) : profile.isError && !(isApiError(profile.error) && profile.error.status === 404) ? (
        <ErrorAlert error={profile.error} onRetry={() => void profile.refetch()} />
      ) : (
        <>
          <div {...tabPanelProps("profile", "pdf")} hidden={mode !== "pdf"}>
            <ResumeUpload
              onApply={(values) => {
                formRef.current?.apply(values);
                setMode("form");
              }}
            />
          </div>
          <div {...tabPanelProps("profile", "form")} hidden={mode !== "form"}>
            <ProfileForm profile={profile.data} accountEmail={me.data?.email} handleRef={formRef} onSaved={() => setJustSaved(true)} />
          </div>
        </>
      )}
    </div>
  );
}
