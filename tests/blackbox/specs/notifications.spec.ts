import { expect, test } from "../lib/session";
import { BASE } from "../lib/env";
import { EMPLOYERS, SEEKERS } from "../lib/accounts";
import { invitationBody } from "../lib/flows";
import { header, waitForLetter } from "../lib/mail";
import { expectSearchableSeeker } from "../lib/seeker";

interface Notification {
  id: string;
  kind: string;
  title: string;
  link: string | null;
  read_at: string | null;
}

interface Setting {
  kind: string;
  in_app: boolean;
  email: boolean;
}

/**
 * Notifications in the cabinet and by email. The stack holds a letter for seconds instead of
 * minutes (deploy/compose.blackbox.yml), so letters arrive while the test waits.
 */
test("приглашение → письмо соискателю и значок; принятие → уведомление и письмо компании без ПДн; отписка в один клик", async ({ login, request }) => {
  const seeker = await login(SEEKERS.notifications);
  await expectSearchableSeeker(seeker);
  const employer = await login(EMPLOYERS.notifications);
  const company = (await employer.api.get<{ name: string }>("/employer/company")).name;
  const profile = await seeker.api.get<{ full_name: string; contacts: { phone?: string | null; email?: string | null; telegram?: string | null } }>("/candidate/profile");
  const pii = [profile.full_name, profile.contacts.phone, profile.contacts.telegram, SEEKERS.notifications].filter((x): x is string => Boolean(x));
  const unread = (await seeker.api.get<{ count: number }>("/me/notifications/unread-count")).count;

  const title = `Rust-разработчик в команду платежей ${Date.now().toString(36)}`;
  const since = Date.now();
  const sent = await employer.api.post<{ id: string; candidate_code: string }>(
    "/employer/invitations",
    invitationBody(seeker.me.id, title, { salary: { min: 287_000, max: 341_000 } }),
    201,
  );

  const letter = await waitForLetter(SEEKERS.notifications, { since, subject: new RegExp(`${company} приглашает`) });
  expect(letter.subject).toContain(title);
  expect(letter.text).toContain("287 000 – 341 000 ₽");
  expect(letter.text).toContain("Ответить можно до");
  expect(letter.text).toContain(`${BASE}/candidate/invitations/${sent.id}`);
  const unsubscribe = header(letter, "List-Unsubscribe")?.replace(/^<|>$/g, "") ?? "";
  expect(unsubscribe).toContain("/api/v1/notifications/unsubscribe?token=");
  expect(header(letter, "List-Unsubscribe-Post")).toBe("List-Unsubscribe=One-Click");

  expect((await seeker.api.get<{ count: number }>("/me/notifications/unread-count")).count).toBe(unread + 1);
  const page = seeker.page;
  await page.goto("/candidate");
  await expect(page.getByTestId("notification-badge").filter({ visible: true })).toHaveText(/\d+/);
  await page.getByTestId("notification-bell").filter({ visible: true }).click();
  const bell = page.getByRole("dialog", { name: "Уведомления" });
  await bell.locator(`a[href="/candidate/invitations/${sent.id}"]`).first().click();
  await expect(page).toHaveURL(new RegExp(`/candidate/invitations/${sent.id}$`));
  await expect
    .poll(async () => (await seeker.api.get<{ items: Notification[] }>("/me/notifications")).items.find((n) => n.link === `/candidate/invitations/${sent.id}`)?.read_at)
    .toBeTruthy();

  const accepted = Date.now();
  await seeker.api.post(`/candidate/invitations/${sent.id}/accept`, {}, 200);
  const feed = await employer.api.get<{ items: Notification[] }>("/me/notifications");
  const item = feed.items.find((n) => n.kind === "invitation_accepted" && n.link === `/employer/candidates/${seeker.me.id}`);
  expect(item?.title).toBe(`Кандидат ${sent.candidate_code} принял приглашение — контакты в кабинете`);

  const companyLetter = await waitForLetter(EMPLOYERS.notifications, {
    since: accepted,
    where: (l) => l.subject.includes(`Кандидат ${sent.candidate_code} принял приглашение`) || /новых? уведомлени/.test(l.subject),
  });
  expect(companyLetter.text).toContain(`Кандидат ${sent.candidate_code} принял приглашение`);
  for (const leak of pii) {
    expect(companyLetter.subject + companyLetter.text + companyLetter.html, `no «${leak}» in the company's letter`).not.toContain(leak);
  }

  const shown = await request.get(unsubscribe, { maxRedirects: 0 });
  expect(shown.status(), "opening the link only shows the page: mail scanners open links too").toBe(303);
  const off = await request.post(unsubscribe, { form: { "List-Unsubscribe": "One-Click" } });
  expect(off.status()).toBe(200);
  expect(((await off.json()) as { kind: string }).kind).toBe("invitation_received");
  const settings = await seeker.api.get<{ items: Setting[] }>("/me/notification-settings");
  for (const s of settings.items) {
    expect(s.in_app, s.kind).toBe(true);
    expect(s.email, s.kind).toBe(s.kind !== "invitation_received");
  }
});
