import { ShieldCheck } from "lucide-react";
import { Badge } from "@/components/ui";

/**
 * A neutral fact for candidates: the company signs in with an email on its website's domain.
 * Its absence says nothing bad, so nothing is shown then.
 */
export function DomainBadge({ domain, className }: { domain?: string | null; className?: string }) {
  if (!domain) return null;
  return (
    <Badge
      tone="muted"
      className={className}
      icon={<ShieldCheck className="size-3.5 text-primary" aria-hidden />}
      title={`Компания входит на платформу с почты на домене ${domain} — на том же домене её сайт`}
    >
      Домен подтверждён
    </Badge>
  );
}
