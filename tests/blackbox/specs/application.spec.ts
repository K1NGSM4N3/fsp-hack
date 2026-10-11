import { expect, test } from "../lib/session";
import { EMPLOYERS, SEEKERS } from "../lib/accounts";
import { expectSearchableSeeker } from "../lib/seeker";

/** ТЗ S-6: the classic path still works next to invitations — a seeker applies to a vacancy. */
test("S-6: соискатель откликается на вакансию, компания видит отклик и контакты", async ({ login }) => {
  const employer = await login(EMPLOYERS.application);
  const run = Date.now().toString(36);
  const vacancy = await employer.api.post<{ id: string; title: string }>(
    "/employer/vacancies",
    {
      title: `Go-разработчик сервиса записи к врачу ${run}`,
      description: `Сервис онлайн-записи к врачу, ${run}: API на Go, PostgreSQL, очереди на Kafka. Команда из шести человек, релизы каждую неделю.`,
      specialization: "backend",
      grade: "middle",
      skills: ["go", "postgresql", "kafka"],
      salary: { min: 250_000, max: 320_000 },
      work_format: "remote",
      city: null,
      status: "published",
    },
    201,
  );

  const seeker = await login(SEEKERS.application);
  await expectSearchableSeeker(seeker);
  const code = seeker.me.candidate?.public_code ?? "";
  const page = seeker.page;
  await page.goto(`/candidate/vacancies/${vacancy.id}`);
  await expect(page.getByRole("heading", { name: vacancy.title })).toBeVisible();
  await page.getByRole("textbox", { name: "Сопроводительное письмо" }).fill("Писал похожий сервис бронирования, интересна задача с очередями.");
  await page.getByRole("button", { name: "Откликнуться" }).click();
  await expect(page.getByText("Отклик отправлен")).toBeVisible();
  await expect(page.getByText(/Вы откликнулись/)).toBeVisible();

  await employer.page.goto(`/employer/vacancies/${vacancy.id}`);
  await employer.page.getByRole("tab", { name: /Отклики/ }).click();
  await expect(employer.page.getByText(`Кандидат ${code}`)).toBeVisible();
  const contacts = await employer.api.get<{ full_name: string | null }>(`/employer/candidates/${seeker.me.id}/contacts`);
  expect(contacts.full_name, "an application discloses contacts like an accepted invitation").toBeTruthy();

  const access = await seeker.api.get<{ company_name: string | null; basis: string }[]>("/candidate/contact-access");
  const company = await employer.api.get<{ name: string }>("/employer/company");
  expect(access).toContainEqual(expect.objectContaining({ company_name: company.name, basis: "application" }));
});
