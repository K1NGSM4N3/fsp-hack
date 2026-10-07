import { expect, test } from "../lib/session";
import { DEMO, SEEKERS } from "../lib/accounts";
import { invitationBody } from "../lib/flows";
import { expectSearchableSeeker } from "../lib/seeker";

interface SentInvitation {
  id: string;
  candidate_id: string;
  status: string;
}

/** ТЗ S-5: the company reaches out to a specific person; contacts open only after their consent. */
test("S-5: приглашение конкретному соискателю → он принимает → компания видит контакты", async ({ login }) => {
  const seeker = await login(SEEKERS.invitation);
  await expectSearchableSeeker(seeker);
  const candidateId = seeker.me.id;
  const profile = await seeker.api.get<{ full_name: string; contacts: { phone?: string | null; email?: string | null } }>("/candidate/profile");

  const employer = await login(DEMO.employer);
  const open = await employer.api.get<{ items: SentInvitation[] }>("/employer/invitations?status=sent,viewed&limit=100");
  for (const inv of open.items.filter((i) => i.candidate_id === candidateId)) {
    await employer.api.post(`/employer/invitations/${inv.id}/withdraw`, {}, 204);
  }
  await employer.api.problem("GET", `/employer/candidates/${candidateId}/contacts`, 403, "contacts_not_granted");

  const title = `Go-разработчик в команду платежей ${Date.now().toString(36)}`;
  const page = employer.page;
  await page.goto(`/employer/candidates/${candidateId}`);
  await expect(page.getByText(/Анонимный профиль/)).toBeVisible();
  await expect(page.getByText("Контакты скрыты")).toBeVisible();
  await page.getByRole("button", { name: "Пригласить" }).first().click();
  const dialog = page.getByRole("dialog", { name: "Приглашение кандидату" });
  await expect(dialog).toBeVisible();
  await dialog.getByRole("textbox", { name: "Роль" }).fill(title);
  await dialog.getByRole("textbox", { name: "от" }).fill("280000");
  await dialog.getByRole("textbox", { name: "до" }).fill("340000");
  await dialog.getByRole("textbox", { name: "Сообщение кандидату" }).fill("Здравствуйте! Видели ваш результат теста — хотим познакомиться.");
  const contact = dialog.getByRole("textbox", { name: "Способ связи" });
  if (!(await contact.inputValue())) await contact.fill("@severny_kod_hr");
  await dialog.getByRole("button", { name: "Отправить приглашение" }).click();
  await expect(page.getByText("Приглашение отправлено").first()).toBeVisible();

  const again = await employer.api.problem("POST", "/employer/invitations", 409, "invitation_already_active", { data: invitationBody(candidateId, title) });
  expect(again.title).toBeTruthy();

  const inbox = seeker.page;
  await inbox.goto("/candidate/invitations");
  await inbox.getByRole("link", { name: new RegExp(title) }).first().click();
  await expect(inbox.getByText("Как связаться с компанией")).toBeVisible();
  await expect(inbox.getByText(/280\s000/).first()).toBeVisible();
  await inbox.getByRole("button", { name: "Принять", exact: true }).click();
  const accept = inbox.getByRole("dialog", { name: "Принять приглашение?" });
  await accept.getByRole("button", { name: "Принять", exact: true }).click();
  await expect(inbox.getByText("Вы приняли приглашение")).toBeVisible();

  await page.goto(`/employer/candidates/${candidateId}`);
  await expect(page.getByText("Кандидат принял приглашение")).toBeVisible();
  await page.getByRole("button", { name: "Показать контакты" }).click();
  await expect(page.getByText(profile.full_name)).toBeVisible();
  if (profile.contacts.phone) await expect(page.getByText(profile.contacts.phone)).toBeVisible();
  const contacts = await employer.api.get<{ full_name: string }>(`/employer/candidates/${candidateId}/contacts`);
  expect(contacts.full_name).toBe(profile.full_name);

  const accepted = await employer.api.get<{ items: SentInvitation[]; by_status: { status: string; count: number }[] }>("/employer/invitations?status=accepted&limit=100");
  expect(accepted.items.every((i) => i.status === "accepted")).toBe(true);
  expect(accepted.items.some((i) => i.candidate_id === candidateId)).toBe(true);
  expect(accepted.by_status.some((c) => c.status === "accepted" && c.count >= 1)).toBe(true);
  const mine = accepted.items.find((i) => i.candidate_id === candidateId);
  const first = await seeker.api.get<{ items: { id: string }[]; next_cursor: string | null }>("/candidate/invitations?limit=1");
  expect(first.items[0]?.id, "the seeker's inbox pages newest first").toBe(mine?.id);
  await employer.api.problem("GET", "/employer/invitations?status=lost", 400, "invalid_query");
  await seeker.api.problem("GET", "/candidate/invitations?cursor=nope", 400, "invalid_query");
});
