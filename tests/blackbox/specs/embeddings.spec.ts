import { expect, test } from "../lib/session";
import { DEMO, PASSWORD, uniqueEmail } from "../lib/accounts";
import { confirmEmail, register } from "../lib/flows";

interface PublicVacancy {
  id: string;
  title: string;
  company_name: string;
  specialization: string;
}

interface Related {
  method: "meaning" | "skills";
  items: { vacancy: PublicVacancy; similarity: number }[];
}

interface OwnVacancy {
  id: string;
  warnings: { kind: string; text: string }[];
}

interface Need {
  id: string;
  title: string;
}

interface Feature {
  key: string;
  label_ru: string;
  explanation_ru: string;
}

interface Shortlist {
  items: { breakdown: { features: Feature[] } }[];
}

interface TrustItem {
  id: string;
  reasons: string[];
}

/** The demo seed's vacancy of «Северный Код» and the need of the demo company with the same title. */
const ORIGINAL = "Go-разработчик в команду платежей";
const ORIGINAL_COMPANY = "Северный Код";

/** The same vacancy in other words (docs/validation/paraphrase.md): word pairs barely overlap, so MinHash stays silent. */
const RETOLD = {
  title: "Golang-инженер в процессинг платежей",
  description:
    "В команду, которая обрабатывает платежи, нужен мидл-бэкендер. Пишем на Golang, данные храним в Postgres, события гоняем через Kafka, сервисы общаются по gRPC и живут в контейнерах Docker. Знание k8s и ClickHouse — преимущество. Предстоит делать платёжные API с идемпотентностью, паттерн outbox и шардирование базы. Оплата от 280 до 360 тысяч рублей, можно работать из дома или приходить в московский офис.",
};

/** The embedding worker fills vectors in the background after the seed; the first run of a fresh stack waits for it. */
const EMBEDDED = { timeout: 150_000, intervals: [2_000, 5_000] };

test.describe("смысловая близость текстов (эмбеддинги)", () => {
  test("похожие вакансии: соседи по смыслу описания, сначала той же специализации — на сайте и в кабинете соискателя", async ({ anonymous, login }) => {
    const guest = await anonymous();
    const list = await guest.api.get<{ items: PublicVacancy[] }>("/vacancies?limit=100");
    const go = list.items.find((v) => v.title === ORIGINAL && v.company_name === ORIGINAL_COMPANY);
    expect(go, "the demo seed publishes the Go vacancy").toBeTruthy();
    const id = go?.id ?? "";
    await expect.poll(async () => (await guest.api.get<Related>(`/vacancies/${id}/similar`)).method, EMBEDDED).toBe("meaning");
    // Other specs publish Go vacancies too, so the whole neighbourhood is read, not just the first four.
    const related = await guest.api.get<Related>(`/vacancies/${id}/similar?limit=12`);
    const specs = related.items.map((i) => i.vacancy.specialization);
    const titles = related.items.map((i) => i.vacancy.title);
    expect(related.items.map((i) => i.vacancy.id)).not.toContain(id);
    expect(specs[0], titles.join(" | ")).toBe("backend");
    const firstOther = specs.findIndex((s) => s !== "backend");
    const others = firstOther < 0 ? [] : specs.slice(firstOther);
    expect(others.includes("backend"), `the same specialization first: ${specs.join(", ")}`).toBe(false);
    expect(titles, "the other backend vacancies of the seed").toEqual(expect.arrayContaining(["Python-разработчик маршрутизации", "Senior Java-разработчик антифрода"]));
    const backend = related.items.filter((i) => i.vacancy.specialization === "backend").map((i) => i.similarity);
    expect(backend, "closest first within the specialization").toEqual([...backend].sort((a, b) => b - a));

    await guest.page.goto(`/vacancies/${id}`);
    const block = guest.page.getByTestId("similar-vacancies");
    await expect(guest.page.getByRole("heading", { name: "Похожие вакансии" })).toBeVisible();
    await expect(block).toHaveAttribute("data-method", "meaning");
    await expect(block.getByRole("link")).toHaveCount(4);
    await expect(block.getByRole("link").first()).toHaveAttribute("href", /^\/vacancies\/[0-9a-f-]{36}$/);

    const seeker = await login(DEMO.candidate);
    await seeker.page.goto(`/candidate/vacancies/${id}`);
    const cabinet = seeker.page.getByTestId("similar-vacancies");
    await expect(cabinet.getByRole("link").first()).toHaveAttribute("href", /^\/candidate\/vacancies\/[0-9a-f-]{36}$/);
  });

  test("подборка: в разложении балла — «Близость описаний» по смыслу", async ({ login }) => {
    const { api, page } = await login(DEMO.employer);
    const need = (await api.get<Need[]>("/employer/needs")).find((n) => n.title === ORIGINAL);
    expect(need, "the demo company has the Go need").toBeTruthy();
    const textOf = async () => {
      const shortlist = await api.post<Shortlist>(`/employer/needs/${need?.id}/shortlist`, {}, 200);
      return shortlist.items[0]?.breakdown.features.find((f) => f.key === "text");
    };
    await expect.poll(async () => (await textOf())?.explanation_ru ?? "", EMBEDDED).toMatch(/^Близость описаний по смыслу: \d+ %/);
    expect((await textOf())?.label_ru).toBe("Близость описаний");

    await page.goto(`/employer/needs/${need?.id}`);
    await page.getByRole("button", { name: "Как посчитан балл" }).first().click();
    await expect(page.getByText("Близость описаний", { exact: true }).first()).toBeVisible();
    await expect(page.getByText(/Близость описаний по смыслу: \d+ %/).first()).toBeVisible();
  });

  test("пересказ чужой вакансии: MinHash молчит, отметка по смыслу — работодателю без имени оригинала и модератору с ним", async ({ anonymous, login }, testInfo) => {
    const email = uniqueEmail(testInfo, "retold");
    const guest = await anonymous();
    const since = Date.now();
    await register(guest.api, { email, password: PASSWORD, role: "employer", company_name: `Пересказ ${Date.now().toString(36)}` });
    await confirmEmail(guest.page, email, since);
    const company = await login(email, PASSWORD);
    const created = await company.api.post<OwnVacancy>(
      "/employer/vacancies",
      {
        ...RETOLD,
        specialization: "backend",
        grade: "middle",
        skills: ["go", "postgresql", "kafka"],
        salary: { min: 280_000, max: 360_000 },
        work_format: "remote",
        city: null,
        status: "published",
      },
      201,
    );
    expect(created.warnings.map((w) => w.kind), "not a verbatim copy").not.toContain("copy_of_other");
    await expect
      .poll(async () => (await company.api.get<OwnVacancy>(`/employer/vacancies/${created.id}`)).warnings.map((w) => w.kind), EMBEDDED)
      .toContain("paraphrase_of_other");
    const own = await company.api.get<OwnVacancy>(`/employer/vacancies/${created.id}`);
    const note = own.warnings.find((w) => w.kind === "paraphrase_of_other")?.text ?? "";
    expect(note).toContain("квоту это не снижает");
    expect(note).not.toContain(ORIGINAL_COMPANY);
    expect((await company.api.get<{ penalty: number }>("/employer/trust")).penalty, "a paraphrase alone does not lower the quota").toBe(0);

    await company.page.goto(`/employer/vacancies/${created.id}`);
    await expect(company.page.getByText("Что заметили проверки")).toBeVisible();
    await expect(company.page.getByText(/По смыслу описание очень похоже на вакансию другой компании/)).toBeVisible();

    const moderator = await login(DEMO.moderator);
    const queue = await moderator.api.get<{ items: TrustItem[] }>("/admin/antifake?limit=100");
    const item = queue.items.find((i) => i.id === created.id);
    expect(item, "the vacancy is in the anti-fake queue").toBeTruthy();
    expect(item?.reasons.join(" ")).toMatch(new RegExp(`Похоже на пересказ вакансии «${ORIGINAL}» компании «${ORIGINAL_COMPANY}»: по смыслу тексты совпадают \\(косинус эмбеддингов 0,\\d{3} при пороге 0,\\d{3}\\)`));
    await moderator.page.goto("/admin/antifake");
    await expect(moderator.page.getByText(/Похоже на пересказ вакансии/).first()).toBeVisible();
  });
});
