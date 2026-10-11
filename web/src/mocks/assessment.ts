import type { Answer, AssessmentTopic, CodeBlock, Grade, ItemKind, Specialization } from "@/api/types";
import { SPEC_TOPICS } from "@/lib/labels";
import { GRADE_THRESHOLD } from "./catalog";
import { mulberry32, pick, randInt, sample, shuffle, type Rng } from "./rng";
import type { AnswerKey } from "./store";

export interface GeneratedItem {
  kind: ItemKind;
  prompt: string;
  code: CodeBlock | null;
  options: string[];
  input_hint: string | null;
  key: AnswerKey;
  time_limit_secs: number;
}

interface Extra {
  prompt: string;
  code?: string | null;
  lang?: string | null;
  time?: number;
}

interface Template {
  id: string;
  title: string;
  topicBySpec: Partial<Record<Specialization, AssessmentTopic>>;
  /** Calibrated IRT difficulty per radical level (0..2); `a` is shared. */
  b: [number, number, number];
  a: number;
  gen: (rng: Rng, level: number) => GeneratedItem;
}

function codeBlock(extra: Extra): CodeBlock | null {
  return extra.code ? { language: extra.lang ?? "text", source: extra.code } : null;
}

function choice(rng: Rng, correct: string, wrong: string[], extra: Extra): GeneratedItem {
  const options = shuffle(rng, [correct, ...sample(rng, wrong, 3)]);
  return {
    kind: "single_choice",
    prompt: extra.prompt,
    code: codeBlock(extra),
    options,
    input_hint: null,
    key: { type: "choice", value: options.indexOf(correct) },
    time_limit_secs: extra.time ?? 90,
  };
}

function numeric(answer: number, extra: Extra, tolerance = 0): GeneratedItem {
  return {
    kind: "number",
    prompt: extra.prompt,
    code: codeBlock(extra),
    options: [],
    input_hint: tolerance ? "Число, например 0.75" : "Целое число",
    key: { type: "number", value: answer, tolerance },
    time_limit_secs: extra.time ?? 120,
  };
}

function text(accepted: string, hint: string, extra: Extra): GeneratedItem {
  return {
    kind: "text",
    prompt: extra.prompt,
    code: codeBlock(extra),
    options: [],
    input_hint: hint,
    key: { type: "text", value: [accepted] },
    time_limit_secs: extra.time ?? 90,
  };
}

function permutations(rng: Rng, correct: string[], n: number): string[][] {
  const seen = new Set([correct.join(",")]);
  const out: string[][] = [];
  for (let tries = 0; out.length < n && tries < 200; tries++) {
    const p = shuffle(rng, correct);
    const k = p.join(",");
    if (!seen.has(k)) {
      seen.add(k);
      out.push(p);
    }
  }
  return out;
}

/** Event-loop order is computed by a tiny simulator, never stored. */
function eventLoop(rng: Rng, level: number): GeneratedItem {
  type Stmt = { kind: "sync" | "timeout" | "micro" | "queue"; label: string; delay: number; nested: string | null };
  const labels = shuffle(rng, ["A", "B", "C", "D", "E", "F"]);
  let li = 0;
  const next = () => labels[li++] as string;
  const stmts: Stmt[] = [];
  const kinds: Stmt["kind"][] =
    level === 0 ? ["sync", "timeout", "micro"] : level === 1 ? ["sync", "timeout", "micro", "sync", "queue"] : ["sync", "timeout", "micro", "timeout", "sync"];
  for (const kind of shuffle(rng, kinds)) {
    const delay = kind === "timeout" ? (level === 2 ? pick(rng, [0, 10]) : 0) : 0;
    const nested = kind === "timeout" && level === 2 && !stmts.some((s) => s.nested) ? next() : null;
    stmts.push({ kind, label: next(), delay, nested });
  }
  const out: string[] = [];
  const micro: string[] = [];
  const timers: Array<{ delay: number; seq: number; s: Stmt }> = [];
  stmts.forEach((s, seq) => {
    if (s.kind === "sync") out.push(s.label);
    else if (s.kind === "micro" || s.kind === "queue") micro.push(s.label);
    else timers.push({ delay: s.delay, seq, s });
  });
  out.push(...micro.splice(0));
  timers.sort((x, y) => x.delay - y.delay || x.seq - y.seq);
  for (const t of timers) {
    out.push(t.s.label);
    if (t.s.nested) out.push(t.s.nested);
  }
  const code = stmts
    .map((s) => {
      switch (s.kind) {
        case "sync":
          return `console.log("${s.label}");`;
        case "micro":
          return `Promise.resolve().then(() => console.log("${s.label}"));`;
        case "queue":
          return `queueMicrotask(() => console.log("${s.label}"));`;
        case "timeout":
          return s.nested
            ? `setTimeout(() => {\n  console.log("${s.label}");\n  Promise.resolve().then(() => console.log("${s.nested}"));\n}, ${s.delay});`
            : `setTimeout(() => console.log("${s.label}"), ${s.delay});`;
      }
    })
    .join("\n");
  const wrong = permutations(rng, out, 6).map((p) => p.join(" "));
  return choice(rng, out.join(" "), wrong, {
    prompt: "В каком порядке строки появятся в консоли?",
    code,
    lang: "javascript",
    time: 120,
  });
}

function loopSum(rng: Rng, level: number): GeneratedItem {
  if (level < 2) {
    const a = randInt(rng, 1, 9);
    const b = randInt(rng, 25, 48);
    const k = randInt(rng, 3, 6);
    const m = pick(rng, [2, 3, 4, 5].filter((x) => x !== k));
    let s = 0;
    for (let i = a; i < b; i++) {
      if (level === 0 ? i % k === 0 : i % k === 0 && i % m !== 0) s += i;
    }
    const cond = level === 0 ? `i % ${k} == 0` : `i % ${k} == 0 and i % ${m} != 0`;
    return numeric(s, {
      prompt: "Какое число выведет программа?",
      code: `s = 0\nfor i in range(${a}, ${b}):\n    if ${cond}:\n        s += i\nprint(s)`,
      lang: "python",
    });
  }
  const x0 = randInt(rng, 1, 5);
  const add = randInt(rng, 1, 3);
  const limit = randInt(rng, 200, 900);
  let x = x0;
  let steps = 0;
  while (x < limit) {
    x = x * 2 + add;
    steps++;
  }
  return numeric(steps, {
    prompt: "Какое число выведет программа?",
    code: `x = ${x0}\nsteps = 0\nwhile x < ${limit}:\n    x = x * 2 + ${add}\n    steps += 1\nprint(steps)`,
    lang: "python",
  });
}

function sqlRows(rng: Rng, level: number): GeneratedItem {
  const customers = shuffle(rng, ["anna", "boris", "vera", "gleb", "dina"]).slice(0, 4);
  const rows = Array.from({ length: 8 }, (_, i) => ({ id: i + 1, customer: pick(rng, customers), amount: randInt(rng, 2, 40) * 50 }));
  const table = ["id | customer | amount", "---+----------+-------", ...rows.map((r) => `${String(r.id).padEnd(2)} | ${r.customer.padEnd(8)} | ${r.amount}`)].join("\n");
  const x = pick(rng, [500, 700, 900, 1000]);
  let query: string;
  let answer: number;
  if (level === 0) {
    query = `SELECT COUNT(*) FROM orders WHERE amount > ${x};`;
    answer = rows.filter((r) => r.amount > x).length;
  } else if (level === 1) {
    const y = pick(rng, [1200, 1500, 1800]);
    query = `SELECT customer\nFROM orders\nGROUP BY customer\nHAVING SUM(amount) > ${y};`;
    const sums = new Map<string, number>();
    for (const r of rows) sums.set(r.customer, (sums.get(r.customer) ?? 0) + r.amount);
    answer = [...sums.values()].filter((s) => s > y).length;
  } else {
    query = `SELECT customer, COUNT(*)\nFROM orders\nWHERE amount >= ${x}\nGROUP BY customer\nHAVING COUNT(*) >= 2;`;
    const counts = new Map<string, number>();
    for (const r of rows) if (r.amount >= x) counts.set(r.customer, (counts.get(r.customer) ?? 0) + 1);
    answer = [...counts.values()].filter((c) => c >= 2).length;
  }
  return numeric(answer, {
    prompt: level === 0 ? "Таблица orders приведена ниже. Какое число вернёт запрос?" : "Таблица orders приведена ниже. Сколько строк вернёт запрос?",
    code: `-- orders\n${table.split("\n").map((l) => `-- ${l}`).join("\n")}\n\n${query}`,
    lang: "sql",
    time: 150,
  });
}

const COMPLEXITIES = ["O(1)", "O(log n)", "O(n)", "O(n log n)", "O(n²)", "O(2ⁿ)"];

function bigO(rng: Rng, level: number): GeneratedItem {
  const variants: Array<[string, string]>[] = [
    [
      ["O(n)", "total = 0\nfor x in items:\n    total += x\nfor x in items:\n    total -= x // 2"],
      ["O(n)", "seen = set()\nfor x in items:\n    if x in seen:\n        return True\n    seen.add(x)\nreturn False"],
    ],
    [
      ["O(n²)", "count = 0\nfor i in range(n):\n    for j in range(i, n):\n        count += 1"],
      ["O(log n)", "i = n\nsteps = 0\nwhile i > 1:\n    i //= 2\n    steps += 1"],
    ],
    [
      ["O(n log n)", "for i in range(n):\n    j = n\n    while j > 0:\n        j //= 2"],
      ["O(n log n)", "i = 1\nwhile i < n:\n    for j in range(n):\n        pass\n    i *= 2"],
      ["O(n)", "i = 1\nwhile i < n:\n    for j in range(i):\n        pass\n    i *= 2"],
    ],
  ];
  const [answer, code] = pick(rng, variants[level] ?? variants[0]!);
  return choice(
    rng,
    answer,
    COMPLEXITIES.filter((c) => c !== answer),
    { prompt: "Какова асимптотическая сложность фрагмента по времени?", code, lang: "python" },
  );
}

function binarySearch(rng: Rng, level: number): GeneratedItem {
  if (level === 0) {
    const k = randInt(rng, 3, 7);
    const n = 2 ** k - 1;
    return numeric(k, { prompt: `Отсортированный массив из ${n} элементов. Какое максимальное число сравнений сделает бинарный поиск?` });
  }
  if (level === 1) {
    const n = randInt(rng, 100, 5000);
    return numeric(Math.floor(Math.log2(n)) + 1, {
      prompt: `Отсортированный массив из ${n} элементов. Какое максимальное число сравнений сделает бинарный поиск?`,
    });
  }
  const k = randInt(rng, 8, 14);
  return numeric(2 ** k - 1, {
    prompt: `Бинарный поиск сделал не больше ${k} сравнений при любом искомом значении. Какой максимальный размер массива это гарантирует?`,
  });
}

function subnet(rng: Rng, level: number): GeneratedItem {
  if (level === 0) {
    const p = pick(rng, [24, 28, 29, 30]);
    return numeric(2 ** (32 - p) - 2, { prompt: `Сколько адресов можно выдать хостам в подсети /${p}?` });
  }
  if (level === 1) {
    const p = pick(rng, [20, 22, 25, 26, 27]);
    return numeric(2 ** (32 - p) - 2, { prompt: `Сколько адресов можно выдать хостам в подсети 10.${randInt(rng, 0, 200)}.0.0/${p}?` });
  }
  const big = pick(rng, [20, 21, 22]);
  const small = pick(rng, [24, 25, 26]);
  return numeric(2 ** (small - big), { prompt: `Сколько подсетей /${small} помещается в одну подсеть /${big}?` });
}

function chmod(rng: Rng, level: number): GeneratedItem {
  const toSym = (oct: number) =>
    [4, 2, 1].map((b, i) => (oct & b ? "rwx"[i] : "-")).join("");
  if (level === 0) {
    const o = [pick(rng, [7, 6]), pick(rng, [5, 4, 0]), pick(rng, [4, 0])];
    return text(o.join(""), "Три цифры, например 750", { prompt: `Запишите права ${o.map(toSym).join("")} в восьмеричной форме.` });
  }
  const umask = level === 1 ? pick(rng, ["022", "027", "077"]) : pick(rng, ["037", "026", "033"]);
  const base = level === 1 ? 0o666 : 0o777;
  const res = (base & ~parseInt(umask, 8)).toString(8).padStart(3, "0");
  return text(res, "Три цифры, например 640", {
    prompt: level === 1 ? `umask ${umask}. С какими правами (восьмерично) будет создан новый файл?` : `umask ${umask}. С какими правами (восьмерично) будет создан новый каталог?`,
  });
}

function cron(rng: Rng, level: number): GeneratedItem {
  if (level === 0) {
    const h = pick(rng, [2, 3, 4, 6]);
    return numeric(24 / h, { prompt: "Сколько раз в сутки выполнится задача?", code: `0 */${h} * * *`, lang: "bash" });
  }
  const step = pick(rng, [10, 15, 20, 30]);
  const from = randInt(rng, 8, 10);
  const to = randInt(rng, 17, 20);
  const perDay = (60 / step) * (to - from + 1);
  if (level === 1) return numeric(perDay, { prompt: "Сколько раз в сутки выполнится задача?", code: `*/${step} ${from}-${to} * * *`, lang: "bash" });
  return numeric(perDay * 5, { prompt: "Сколько раз за неделю выполнится задача?", code: `*/${step} ${from}-${to} * * 1-5`, lang: "bash" });
}

function mlMetrics(rng: Rng, level: number): GeneratedItem {
  const tp = randInt(rng, 20, 90);
  const fp = randInt(rng, 5, 40);
  const fn = randInt(rng, 5, 40);
  const tn = randInt(rng, 50, 200);
  const r2 = (v: number) => Math.round(v * 100) / 100;
  const matrix = `TP = ${tp}, FP = ${fp}, FN = ${fn}, TN = ${tn}`;
  if (level === 0) return numeric(r2((tp + tn) / (tp + fp + fn + tn)), { prompt: `Матрица ошибок: ${matrix}. Чему равна accuracy? Округлите до сотых.` }, 0.011);
  if (level === 1) {
    const which = pick(rng, ["precision", "recall"] as const);
    const v = which === "precision" ? tp / (tp + fp) : tp / (tp + fn);
    return numeric(r2(v), { prompt: `Матрица ошибок: ${matrix}. Чему равен ${which}? Округлите до сотых.` }, 0.011);
  }
  const p = tp / (tp + fp);
  const r = tp / (tp + fn);
  return numeric(r2((2 * p * r) / (p + r)), { prompt: `Матрица ошибок: ${matrix}. Чему равна F1-мера? Округлите до сотых.`, time: 180 }, 0.011);
}

function dlParams(rng: Rng, level: number): GeneratedItem {
  if (level === 0) {
    const n = pick(rng, [16, 32, 64, 128]);
    const m = pick(rng, [8, 10, 16, 32]);
    return numeric(n * m + m, { prompt: `Сколько обучаемых параметров у полносвязного слоя ${n} → ${m} со смещением?` });
  }
  if (level === 1) {
    const k = pick(rng, [3, 5]);
    const cin = pick(rng, [3, 16, 32]);
    const cout = pick(rng, [16, 32, 64]);
    return numeric(k * k * cin * cout + cout, {
      prompt: `Сколько обучаемых параметров у свёрточного слоя Conv2d(in=${cin}, out=${cout}, kernel=${k}×${k}) со смещением?`,
    });
  }
  const a = pick(rng, [64, 128]);
  const b = pick(rng, [32, 64]);
  const c = pick(rng, [2, 10]);
  return numeric(a * b + b + b * c + c, {
    prompt: "Сколько обучаемых параметров у сети?",
    code: `nn.Sequential(\n    nn.Linear(${a}, ${b}),\n    nn.ReLU(),\n    nn.Linear(${b}, ${c}),\n)`,
    lang: "python",
  });
}

function stats(rng: Rng, level: number): GeneratedItem {
  const xs = Array.from({ length: level === 1 ? 6 : 5 }, () => randInt(rng, 1, 20));
  if (level === 0) {
    const s = xs.reduce((a, b) => a + b, 0);
    const fix = (5 - (s % 5)) % 5;
    xs[0] = (xs[0] ?? 0) + fix;
    return numeric((s + fix) / 5, { prompt: `Найдите среднее значение выборки: ${xs.join(", ")}.` });
  }
  if (level === 1) {
    const sorted = [...xs].sort((a, b) => a - b);
    return numeric(((sorted[2] ?? 0) + (sorted[3] ?? 0)) / 2, { prompt: `Найдите медиану выборки: ${xs.join(", ")}.` }, 0.01);
  }
  const mean = randInt(rng, 5, 12);
  const ds = pick(rng, [[-2, -1, 0, 1, 2], [-3, 0, 0, 0, 3], [-4, -2, 0, 2, 4]]);
  const vals = ds.map((d) => mean + d);
  const variance = ds.reduce((a, d) => a + d * d, 0) / ds.length;
  return numeric(variance, { prompt: `Найдите дисперсию (генеральную, деление на n) выборки: ${vals.join(", ")}.` }, 0.01);
}

function reactState(rng: Rng, level: number): GeneratedItem {
  const c0 = randInt(rng, 0, 5);
  const body =
    level === 0
      ? ["setCount(c => c + 1);", "setCount(c => c + 1);", "setCount(c => c + 1);"]
      : level === 1
        ? ["setCount(count + 1);", "setCount(count + 1);", "setCount(count + 1);"]
        : ["setCount(c => c + 1);", "setCount(count + 5);", "setCount(c => c * 2);"];
  const answer = level === 0 ? c0 + 3 : level === 1 ? c0 + 1 : (c0 + 5) * 2;
  return numeric(answer, {
    prompt: "Какое значение count отобразится после одного клика по кнопке?",
    code: `function Counter() {\n  const [count, setCount] = useState(${c0});\n  const onClick = () => {\n    ${body.join("\n    ")}\n  };\n  return <button onClick={onClick}>{count}</button>;\n}`,
    lang: "javascript",
  });
}

function cssSpecificity(rng: Rng, level: number): GeneratedItem {
  const sets: Array<[string, string[]]> = [
    ["#menu a", ["nav a", ".menu a", "a:hover"]],
    ["div#card .title", [".card .title span", "div.card .title", ".card.active .title"]],
    [".list li:not(#skip) a", ["ul.list li a.link", ".list .item a", "ul li.item a"]],
  ];
  const [correct, wrong] = sets[level] ?? sets[0]!;
  return choice(rng, correct, wrong, {
    prompt: "Все селекторы задают color одному и тому же элементу. Какое правило применится (у какого селектора выше специфичность)?",
    time: 90,
  });
}

function k8sRollout(rng: Rng, level: number): GeneratedItem {
  const n = pick(rng, [4, 6, 8, 10]);
  if (level === 0) {
    const s = randInt(rng, 1, 3);
    return numeric(n + s, { prompt: `Deployment: replicas: ${n}, maxSurge: ${s}, maxUnavailable: 0. Сколько подов максимум может существовать во время rolling update?` });
  }
  const pct = pick(rng, [25, 30, 50]);
  if (level === 1) {
    return numeric(n - Math.floor((n * pct) / 100), {
      prompt: `Deployment: replicas: ${n}, maxUnavailable: ${pct}%. Сколько подов гарантированно будут доступны во время rolling update?`,
    });
  }
  return numeric(n + Math.ceil((n * pct) / 100), {
    prompt: `Deployment: replicas: ${n}, maxSurge: ${pct}%, maxUnavailable: ${pct}%. Сколько подов максимум может существовать во время rolling update?`,
  });
}

function sloBudget(_rng: Rng, level: number): GeneratedItem {
  const cfg: Array<[number, number]> = [
    [99, 30],
    [99.9, 30],
    [99.95, 7],
  ];
  const [slo, days] = cfg[level] ?? cfg[0]!;
  const minutes = Math.round(days * 24 * 60 * (1 - slo / 100) * 100) / 100;
  return numeric(minutes, { prompt: `SLO доступности ${slo} % за ${days} дней. Сколько минут простоя допускает бюджет ошибок? Округлите до сотых.` }, 0.06);
}

function boundary(rng: Rng, level: number): GeneratedItem {
  const a = randInt(rng, 1, 18);
  const b = a + randInt(rng, 20, 60);
  if (level === 0) {
    const mid = Math.round((a + b) / 2);
    return choice(rng, String(mid), [String(a), String(b), String(a - 1), String(b + 1)], {
      prompt: `Поле принимает целые числа от ${a} до ${b} включительно. Какое значение НЕ является граничным?`,
    });
  }
  if (level === 1) {
    return choice(
      rng,
      `${a - 1}, ${a}, ${b}, ${b + 1}`,
      [`${a}, ${b}`, `${a}, ${Math.round((a + b) / 2)}, ${b}`, `${a - 1}, ${b + 1}`, `${a + 1}, ${b - 1}`],
      { prompt: `Поле принимает целые от ${a} до ${b} включительно. Какой минимальный набор значений для анализа граничных значений (по две точки на границу)?` },
    );
  }
  return numeric(5, {
    prompt:
      "Скидка зависит от возраста: 0–17 — детская, 18–64 — без скидки, 65–120 — пенсионная. Значения вне 0–120 — ошибка ввода. Сколько классов эквивалентности, включая невалидные?",
  });
}

interface Fact {
  q: string;
  correct: string;
  wrong: string[];
  code?: string;
  lang?: string;
}

function facts(levels: Fact[][]) {
  return (rng: Rng, level: number): GeneratedItem => {
    const f = pick(rng, levels[level] ?? levels[0]!);
    return choice(rng, f.correct, f.wrong, { prompt: f.q, code: f.code ?? null, lang: f.lang ?? null });
  };
}

const httpFacts = facts([
  [
    { q: "Клиент запросил несуществующий ресурс. Какой статус вернуть?", correct: "404 Not Found", wrong: ["400 Bad Request", "410 Gone", "500 Internal Server Error", "403 Forbidden"] },
    { q: "Ресурс успешно создан. Какой статус вернуть?", correct: "201 Created", wrong: ["200 OK", "202 Accepted", "204 No Content", "302 Found"] },
  ],
  [
    { q: "Пользователь вошёл в систему, но не имеет прав на ресурс. Какой статус?", correct: "403 Forbidden", wrong: ["401 Unauthorized", "404 Not Found", "400 Bad Request", "409 Conflict"] },
    { q: "JSON синтаксически верен, но поле email не прошло валидацию. Какой статус уместнее?", correct: "422 Unprocessable Content", wrong: ["400 Bad Request", "409 Conflict", "415 Unsupported Media Type", "406 Not Acceptable"] },
  ],
  [
    { q: "Запрос с If-Match, ETag ресурса уже изменился. Какой статус?", correct: "412 Precondition Failed", wrong: ["409 Conflict", "304 Not Modified", "428 Precondition Required", "400 Bad Request"] },
    { q: "Клиент превысил лимит запросов. Какой статус и заголовок?", correct: "429 и Retry-After", wrong: ["503 и Retry-After", "429 и Location", "403 и WWW-Authenticate", "420 и Retry-After"] },
  ],
]);

const dockerFacts = facts([
  [
    { q: "Какая инструкция Dockerfile задаёт команду по умолчанию при запуске контейнера?", correct: "CMD", wrong: ["RUN", "FROM", "COPY", "WORKDIR"] },
  ],
  [
    {
      q: "Код меняется в каждом коммите, зависимости — редко. Какой порядок инструкций лучше использует кэш слоёв?",
      correct: "COPY requirements.txt → RUN pip install → COPY . .",
      wrong: ["COPY . . → RUN pip install", "RUN pip install → COPY requirements.txt → COPY . .", "COPY . . → COPY requirements.txt → RUN pip install"],
    },
  ],
  [
    {
      q: "Зачем в многоэтапной сборке (multi-stage) финальный образ строят FROM минимального базового образа?",
      correct: "Чтобы в образ не попали компиляторы и исходники — меньше размер и поверхность атаки",
      wrong: ["Чтобы ускорить RUN-инструкции", "Чтобы кэш слоёв не использовался", "Чтобы контейнер запускался от root"],
    },
  ],
]);

const secVulnFacts = facts([
  [
    {
      q: "Какая уязвимость в этом коде?",
      code: 'query = "SELECT * FROM users WHERE login = \'" + login + "\'"\ncursor.execute(query)',
      lang: "python",
      correct: "SQL-инъекция",
      wrong: ["XSS", "SSRF", "CSRF", "Path traversal"],
    },
  ],
  [
    {
      q: "Какая уязвимость в этом обработчике?",
      code: 'app.get("/search", (req, res) => {\n  res.send("<p>Вы искали: " + req.query.q + "</p>");\n});',
      lang: "javascript",
      correct: "Отражённая XSS",
      wrong: ["SQL-инъекция", "SSRF", "Небезопасная десериализация", "Open redirect"],
    },
    {
      q: "Какая уязвимость в этом обработчике?",
      code: 'app.get("/preview", async (req, res) => {\n  const page = await fetch(req.query.url);\n  res.send(await page.text());\n});',
      lang: "javascript",
      correct: "SSRF",
      wrong: ["XSS", "CSRF", "SQL-инъекция", "Path traversal"],
    },
  ],
  [
    {
      q: "Какая уязвимость в этом коде?",
      code: 'const file = path.join("/srv/files", req.query.name);\nres.sendFile(file);',
      lang: "javascript",
      correct: "Path traversal",
      wrong: ["SSRF", "XSS", "Open redirect", "IDOR"],
    },
    {
      q: "Какая уязвимость в этом коде?",
      code: "data = pickle.loads(request.body)\nprocess(data)",
      lang: "python",
      correct: "Небезопасная десериализация",
      wrong: ["SQL-инъекция", "SSRF", "Path traversal", "CSRF"],
    },
  ],
]);

const cryptoFacts = facts([
  [{ q: "Что из перечисленного — криптографическая хеш-функция?", correct: "SHA-256", wrong: ["AES-256", "RSA", "TLS 1.3", "Base64"] }],
  [{ q: "Что подходит для хранения паролей пользователей?", correct: "Argon2id с солью", wrong: ["SHA-256 без соли", "MD5 с солью", "AES с ключом в конфиге", "Base64"] }],
  [
    {
      q: "Почему режим шифрования ECB считается небезопасным?",
      correct: "Одинаковые блоки открытого текста дают одинаковые блоки шифротекста",
      wrong: ["Он требует слишком длинный ключ", "Он не поддерживает AES", "Он медленнее CBC в 10 раз"],
    },
  ],
]);

const qaProcessFacts = facts([
  [{ q: "Как называется быстрая проверка основных функций новой сборки перед глубоким тестированием?", correct: "Smoke-тестирование", wrong: ["Регрессионное тестирование", "Нагрузочное тестирование", "Альфа-тестирование"] }],
  [{ q: "Опечатка в юридическом тексте на главной странице перед релизом. Какая комбинация оценки дефекта уместна?", correct: "Низкая серьёзность, высокий приоритет", wrong: ["Высокая серьёзность, низкий приоритет", "Блокер, высокий приоритет", "Низкая серьёзность, низкий приоритет"] }],
  [{ q: "Что из перечисленного — техника тест-дизайна на основе таблиц принятия решений?", correct: "Покрытие всех комбинаций условий и действий", wrong: ["Анализ граничных значений", "Исследовательское тестирование", "Попарное тестирование только по двум параметрам"] }],
]);

const mlClassicFacts = facts([
  [{ q: "Ошибка на обучении низкая, на валидации — высокая. Что это?", correct: "Переобучение", wrong: ["Недообучение", "Утечка данных", "Дисбаланс классов"] }],
  [{ q: "Какая регуляризация чаще приводит к разреженным весам (часть весов равна нулю)?", correct: "L1", wrong: ["L2", "Dropout", "Batch normalization"] }],
  [{ q: "Чем бустинг отличается от бэггинга?", correct: "Модели обучаются последовательно, каждая исправляет ошибки предыдущих", wrong: ["Модели обучаются параллельно на бутстрап-выборках", "Используется только одна глубокая модель", "Признаки отбираются случайно без обучения"] }],
]);

const secProcessFacts = facts([
  [{ q: "Что первым делом сделать при подтверждённой компрометации учётной записи сотрудника?", correct: "Заблокировать учётную запись и отозвать сессии", wrong: ["Удалить журналы событий", "Сообщить всем сотрудникам пароль администратора", "Переустановить ОС на всех серверах"] }],
  [{ q: "Что означает принцип наименьших привилегий?", correct: "Выдавать только права, необходимые для задачи", wrong: ["Выдавать права администратора всем разработчикам", "Хранить пароли в одном месте", "Использовать один ключ для всех сервисов"] }],
  [{ q: "Что оценивает CVSS?", correct: "Серьёзность уязвимости", wrong: ["Вероятность фишинга", "Скорость реагирования SOC", "Зрелость процессов ИБ"] }],
]);

const closureFacts = facts([
  [{ q: "Что выведет код?", code: "for (let i = 0; i < 3; i++) {\n  setTimeout(() => console.log(i));\n}", lang: "javascript", correct: "0 1 2", wrong: ["3 3 3", "0 0 0", "undefined ×3"] }],
  [{ q: "Что выведет код?", code: "for (var i = 0; i < 3; i++) {\n  setTimeout(() => console.log(i));\n}", lang: "javascript", correct: "3 3 3", wrong: ["0 1 2", "2 2 2", "undefined ×3"] }],
  [{ q: "Что выведет код?", code: "const fns = [];\nfor (var i = 0; i < 3; i++) {\n  fns.push(((j) => () => j)(i));\n}\nconsole.log(fns.map(f => f()).join(\" \"));", lang: "javascript", correct: "0 1 2", wrong: ["3 3 3", "2 2 2", "undefined undefined undefined"] }],
]);

const TEMPLATES: Template[] = [
  { id: "event_loop", title: "Порядок выполнения в event loop", topicBySpec: { backend: "languages", frontend: "javascript" }, b: [-1.1, 0.1, 1.2], a: 1.4, gen: eventLoop },
  { id: "loop_sum", title: "Результат работы цикла", topicBySpec: { backend: "languages", qa: "test_automation" }, b: [-1.4, -0.3, 0.6], a: 1.2, gen: loopSum },
  { id: "sql_rows", title: "Результат SQL-запроса", topicBySpec: { backend: "databases", qa: "databases", ml: "data_tools" }, b: [-1.2, 0, 0.9], a: 1.5, gen: sqlRows },
  { id: "big_o", title: "Сложность алгоритма", topicBySpec: { backend: "algorithms", frontend: "algorithms", ml: "algorithms" }, b: [-1.3, -0.2, 0.8], a: 1.3, gen: bigO },
  { id: "binary_search", title: "Бинарный поиск", topicBySpec: { backend: "algorithms", frontend: "algorithms", ml: "algorithms" }, b: [-0.6, 0.3, 1.1], a: 1.2, gen: binarySearch },
  { id: "http", title: "Коды ответа HTTP", topicBySpec: { backend: "web", frontend: "web", qa: "web", security: "web" }, b: [-1.5, -0.4, 0.7], a: 1.1, gen: httpFacts },
  { id: "subnet", title: "Адресация в сетях", topicBySpec: { devops: "networking", security: "network_security" }, b: [-0.8, 0.2, 1.1], a: 1.4, gen: subnet },
  { id: "docker", title: "Сборка контейнеров", topicBySpec: { backend: "architecture", devops: "containers" }, b: [-1.2, -0.1, 0.9], a: 1.1, gen: dockerFacts },
  { id: "sec_vuln", title: "Уязвимость в коде", topicBySpec: { backend: "architecture", security: "appsec" }, b: [-1.3, -0.1, 1], a: 1.3, gen: secVulnFacts },
  { id: "closure", title: "Замыкания", topicBySpec: { frontend: "javascript" }, b: [-1.2, -0.2, 0.9], a: 1.3, gen: closureFacts },
  { id: "css", title: "Специфичность селекторов", topicBySpec: { frontend: "layout" }, b: [-1, 0, 1], a: 1.2, gen: cssSpecificity },
  { id: "react_state", title: "Обновление состояния", topicBySpec: { frontend: "frameworks" }, b: [-0.9, 0.2, 1.2], a: 1.5, gen: reactState },
  { id: "chmod", title: "Права доступа", topicBySpec: { devops: "linux", security: "linux" }, b: [-1, 0.2, 1], a: 1.3, gen: chmod },
  { id: "cron", title: "Расписание cron", topicBySpec: { devops: "tools" }, b: [-0.9, 0.1, 1], a: 1.2, gen: cron },
  { id: "k8s", title: "Rolling update", topicBySpec: { devops: "containers" }, b: [-1, 0.1, 1.1], a: 1.3, gen: k8sRollout },
  { id: "slo", title: "Бюджет ошибок", topicBySpec: { devops: "observability", backend: "architecture" }, b: [-0.9, 0.2, 1.1], a: 1.2, gen: sloBudget },
  { id: "boundary", title: "Граничные значения", topicBySpec: { qa: "test_design" }, b: [-1.4, -0.2, 0.8], a: 1.3, gen: boundary },
  { id: "qa_process", title: "Процесс тестирования", topicBySpec: { qa: "qa_process" }, b: [-1.3, -0.1, 0.9], a: 1.1, gen: qaProcessFacts },
  { id: "ml_metrics", title: "Метрики классификации", topicBySpec: { ml: "ml_metrics" }, b: [-1.2, -0.1, 0.9], a: 1.5, gen: mlMetrics },
  { id: "dl_params", title: "Параметры нейросети", topicBySpec: { ml: "modeling" }, b: [-0.9, 0.2, 1.1], a: 1.4, gen: dlParams },
  { id: "stats", title: "Описательная статистика", topicBySpec: { ml: "statistics" }, b: [-1.5, -0.4, 0.7], a: 1.2, gen: stats },
  { id: "ml_classic", title: "Обучение моделей", topicBySpec: { ml: "modeling" }, b: [-1.3, -0.1, 0.9], a: 1.2, gen: mlClassicFacts },
  { id: "crypto", title: "Криптография", topicBySpec: { security: "cryptography" }, b: [-1.4, -0.3, 0.6], a: 1.2, gen: cryptoFacts },
  { id: "sec_process", title: "Реагирование на инциденты", topicBySpec: { security: "appsec" }, b: [-1.3, -0.2, 0.8], a: 1.1, gen: secProcessFacts },
];

const GRID = Array.from({ length: 81 }, (_, i) => -4 + i * 0.1);
const D = 1.7;

function p3pl(theta: number, a: number, b: number, c: number) {
  return c + (1 - c) / (1 + Math.exp(-D * a * (theta - b)));
}

export function priorFor(grade: Grade): number[] {
  const mu = GRADE_THRESHOLD[grade];
  const raw = GRID.map((t) => Math.exp(-0.5 * (t - mu) ** 2));
  const sum = raw.reduce((x, y) => x + y, 0);
  return raw.map((v) => v / sum);
}

export function updatePosterior(post: number[], correct: boolean, a: number, b: number, c: number): number[] {
  const next = post.map((w, i) => {
    const p = p3pl(GRID[i] as number, a, b, c);
    return w * (correct ? p : 1 - p);
  });
  const sum = next.reduce((x, y) => x + y, 0) || 1;
  return next.map((v) => v / sum);
}

export function posteriorStats(post: number[]) {
  const mean = post.reduce((acc, w, i) => acc + w * (GRID[i] as number), 0);
  const variance = post.reduce((acc, w, i) => acc + w * ((GRID[i] as number) - mean) ** 2, 0);
  return { theta: Math.round(mean * 100) / 100, se: Math.round(Math.sqrt(variance) * 100) / 100 };
}

export function probAbove(post: number[], threshold: number): number {
  return post.reduce((acc, w, i) => acc + ((GRID[i] as number) >= threshold - 1e-9 ? w : 0), 0);
}

function information(theta: number, a: number, b: number, c: number) {
  const p = p3pl(theta, a, b, c);
  return (D * a) ** 2 * ((p - c) ** 2 / (1 - c) ** 2) * ((1 - p) / p);
}

export interface PickedItem {
  template_id: string;
  title: string;
  level: number;
  topic: AssessmentTopic;
  a: number;
  b: number;
  c: number;
  item: GeneratedItem;
}

/**
 * Max-information selection with topic quotas and "randomesque" exposure control:
 * pick randomly among the 3 most informative candidates, so two candidates with
 * the same θ path still see different items.
 */
export function selectNext(spec: Specialization, theta: number, used: Array<{ template_id: string; topic: AssessmentTopic }>, seed: number): PickedItem | null {
  const rng = mulberry32(seed);
  const topics = SPEC_TOPICS[spec];
  const usage = new Map<AssessmentTopic, number>();
  for (const u of used) usage.set(u.topic, (usage.get(u.topic) ?? 0) + 1);
  const templates = TEMPLATES.filter((t) => t.topicBySpec[spec] && used.filter((u) => u.template_id === t.id).length < 2);
  const coveredTopics = topics.filter((tp) => templates.some((t) => t.topicBySpec[spec] === tp));
  const minUse = Math.min(...coveredTopics.map((tp) => usage.get(tp) ?? 0));
  const balanced = templates.filter((t) => (usage.get(t.topicBySpec[spec] as AssessmentTopic) ?? 0) === minUse);
  const pool = balanced.length ? balanced : templates;
  if (!pool.length) return null;
  const candidates = pool.flatMap((t) =>
    [0, 1, 2].map((level) => {
      const b = t.b[level] as number;
      return { t, level, b, info: information(theta, t.a, b, 0.2) };
    }),
  );
  candidates.sort((x, y) => y.info - x.info);
  const chosen = pick(rng, candidates.slice(0, 3));
  const item = chosen.t.gen(rng, chosen.level);
  return {
    template_id: chosen.t.id,
    title: chosen.t.title,
    level: chosen.level,
    topic: chosen.t.topicBySpec[spec] as AssessmentTopic,
    a: chosen.t.a,
    b: chosen.b,
    c: item.kind === "single_choice" ? 1 / Math.max(2, item.options.length) : 0,
    item,
  };
}

export function checkAnswer(key: AnswerKey, answer: Answer | null): boolean {
  if (!answer) return false;
  if (key.type === "choice") return answer.type === "choice" && answer.value === key.value;
  if (key.type === "number") {
    const n = answer.type === "number" ? answer.value : answer.type === "text" ? Number(answer.value.trim().replace(",", ".")) : NaN;
    return Number.isFinite(n) && Math.abs(n - key.value) <= key.tolerance + 1e-9;
  }
  if (answer.type !== "text") return false;
  const norm = (x: string) => x.trim().toLowerCase().replace(/\s+/g, " ");
  return key.value.some((v) => norm(v) === norm(answer.value));
}

export const MOCK_ITEMS_MIN = 6;
export const MOCK_ITEMS_MAX = 12;
