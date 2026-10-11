import type { ReactNode } from "react";
import { Providers } from "../../providers";

export default function NotificationsLayout({ children }: { children: ReactNode }) {
  return <Providers>{children}</Providers>;
}
