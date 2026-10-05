//! Security templates: vulnerability classes in code, entropy, XOR, encodings, ports,
//! packet filtering, sanitiser bypasses.

use catalog::Specialization::{Backend, Devops, Security};

use super::util::{chance, pick, pick_n, plural_ru, py_replace, range, shuffled};
use super::{C_CHOICE, C_OPEN, Draft, Level, Rng, Source, Template, Topic};

pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        id: "sec.vulnerability_class",
        title: "Класс уязвимости в коде",
        topic: Topic::Appsec,
        specializations: &[Security, Backend],
        skills: &["owasp", "appsec"],
        levels: &[
            Level::new(1, 1.0, -0.9, C_CHOICE, 90),
            Level::new(2, 1.3, 0.0, C_CHOICE, 120),
            Level::new(3, 1.5, 1.0, C_CHOICE, 150),
        ],
        source: Source::Computed(vulnerability_class),
    },
    Template {
        id: "sec.password_entropy",
        title: "Энтропия паролей",
        topic: Topic::Cryptography,
        specializations: &[Security],
        skills: &["cryptography"],
        levels: &[
            Level::new(1, 1.0, -0.4, C_OPEN, 90),
            Level::new(2, 1.2, 0.3, C_OPEN, 120),
            Level::new(3, 1.4, 1.1, C_OPEN, 150),
        ],
        source: Source::Computed(password_entropy),
    },
    Template {
        id: "sec.xor_cipher",
        title: "XOR-шифрование",
        topic: Topic::Cryptography,
        specializations: &[Security],
        skills: &["cryptography"],
        levels: &[
            Level::new(1, 1.0, -0.5, C_CHOICE, 90),
            Level::new(2, 1.2, 0.3, C_CHOICE, 90),
            Level::new(3, 1.4, 1.2, C_CHOICE, 150),
        ],
        source: Source::Computed(xor_cipher),
    },
    Template {
        id: "sec.base64",
        title: "Base64 — кодирование, а не шифрование",
        topic: Topic::Cryptography,
        specializations: &[Security],
        skills: &["cryptography", "http"],
        levels: &[
            Level::new(1, 0.9, -0.8, C_OPEN, 60),
            Level::new(2, 1.1, 0.0, C_CHOICE, 120),
        ],
        source: Source::Computed(base64_item),
    },
    Template {
        id: "sec.service_ports",
        title: "Стандартные порты сервисов",
        topic: Topic::NetworkSecurity,
        specializations: &[Security, Devops],
        skills: &["network_security", "networking"],
        levels: &[
            Level::new(1, 0.9, -1.3, C_CHOICE, 60),
            Level::new(2, 1.0, -0.4, C_CHOICE, 60),
            Level::new(3, 1.2, 0.6, C_CHOICE, 60),
        ],
        source: Source::Computed(service_ports),
    },
    Template {
        id: "sec.firewall_rules",
        title: "Порядок правил iptables",
        topic: Topic::NetworkSecurity,
        specializations: &[Security, Devops],
        skills: &["network_security", "linux"],
        levels: &[
            Level::new(1, 1.0, -0.5, C_CHOICE, 90),
            Level::new(2, 1.3, 0.3, C_CHOICE, 120),
            Level::new(3, 1.5, 1.0, C_CHOICE, 150),
        ],
        source: Source::Computed(firewall),
    },
    Template {
        id: "sec.filter_bypass",
        title: "Обход наивной фильтрации",
        topic: Topic::Appsec,
        specializations: &[Security],
        skills: &["appsec", "pentest"],
        levels: &[
            Level::new(1, 1.2, 0.3, C_CHOICE, 120),
            Level::new(2, 1.4, 1.1, C_CHOICE, 150),
        ],
        source: Source::Computed(filter_bypass),
    },
];

#[derive(Clone, Copy, PartialEq)]
enum Vuln {
    Sqli,
    Xss,
    CommandInjection,
    PathTraversal,
    Ssrf,
    OpenRedirect,
    Idor,
    Deserialization,
}

impl Vuln {
    const ALL: [Vuln; 8] = [
        Vuln::Sqli,
        Vuln::Xss,
        Vuln::CommandInjection,
        Vuln::PathTraversal,
        Vuln::Ssrf,
        Vuln::OpenRedirect,
        Vuln::Idor,
        Vuln::Deserialization,
    ];

    fn label(self) -> &'static str {
        match self {
            Vuln::Sqli => "SQL-инъекция",
            Vuln::Xss => "XSS (межсайтовый скриптинг)",
            Vuln::CommandInjection => "Внедрение команд ОС",
            Vuln::PathTraversal => "Path traversal (выход за пределы каталога)",
            Vuln::Ssrf => "SSRF (подделка запросов со стороны сервера)",
            Vuln::OpenRedirect => "Открытый редирект",
            Vuln::Idor => "IDOR (доступ к чужим объектам по идентификатору)",
            Vuln::Deserialization => "Небезопасная десериализация",
        }
    }
}

struct Names {
    route: &'static str,
    param: &'static str,
    table: &'static str,
    link: &'static str,
    next: &'static str,
    allowed: &'static str,
}

fn snippet(v: Vuln, level: u8, python: bool, n: &Names) -> (String, &'static str) {
    let Names {
        route,
        param,
        table,
        link,
        next,
        allowed,
    } = n;
    match (v, level, python) {
        (Vuln::Sqli, 3, _) => (
            format!(
                "@app.get(\"/{route}\")\n@login_required\ndef list_{table}():\n    order = request.args.get(\"sort\", \"created_at\")\n    cur.execute(\n        f\"SELECT id, title FROM {table} WHERE owner_id = %s ORDER BY {{order}}\",\n        (current_user.id,),\n    )\n    return jsonify(cur.fetchall())\n"
            ),
            "python",
        ),
        (Vuln::Sqli, _, true) => (
            format!(
                "@app.get(\"/{route}\")\ndef find():\n    {param} = request.args[\"{param}\"]\n    cur.execute(f\"SELECT * FROM {table} WHERE {param} = '{{{param}}}'\")\n    return jsonify(cur.fetchall())\n"
            ),
            "python",
        ),
        (Vuln::Sqli, _, false) => (
            format!(
                "app.get('/{route}', async (req, res) => {{\n  const rows = await db.query(\n    `SELECT * FROM {table} WHERE {param} = '${{req.query.{param}}}'`\n  );\n  res.json(rows);\n}});\n"
            ),
            "javascript",
        ),
        (Vuln::Xss, 3, _) => (
            format!(
                "const params = new URLSearchParams(location.search);\nconst {param} = params.get('{param}') ?? '';\ndocument.querySelector('#greeting').innerHTML = `Привет, ${{{param}}}!`;\n"
            ),
            "javascript",
        ),
        (Vuln::Xss, _, true) => (
            format!(
                "@app.get(\"/{route}\")\ndef page():\n    {param} = request.args.get(\"{param}\", \"\")\n    return f\"<h1>Результаты по запросу: {{{param}}}</h1>\"\n"
            ),
            "python",
        ),
        (Vuln::Xss, _, false) => (
            format!(
                "app.get('/{route}', (req, res) => {{\n  res.send(`<p>Результаты по запросу: ${{req.query.{param}}}</p>`);\n}});\n"
            ),
            "javascript",
        ),
        (Vuln::CommandInjection, _, true) => (
            format!(
                "@app.post(\"/{route}\")\ndef check():\n    {param} = request.form[\"{param}\"]\n    output = os.popen(f\"ping -c 1 {{{param}}}\").read()\n    return output\n"
            ),
            "python",
        ),
        (Vuln::CommandInjection, _, false) => (
            format!(
                "app.post('/{route}', (req, res) => {{\n  exec(`ping -c 1 ${{req.body.{param}}}`, (err, stdout) => {{\n    res.send(stdout);\n  }});\n}});\n"
            ),
            "javascript",
        ),
        (Vuln::PathTraversal, 3, _) => (
            format!(
                "@app.get(\"/{route}\")\ndef download():\n    name = request.args[\"{param}\"].replace(\"../\", \"\")\n    return send_file(os.path.join(UPLOAD_DIR, name))\n"
            ),
            "python",
        ),
        (Vuln::PathTraversal, _, true) => (
            format!(
                "@app.get(\"/{route}\")\ndef download():\n    name = request.args[\"{param}\"]\n    return send_file(os.path.join(UPLOAD_DIR, name))\n"
            ),
            "python",
        ),
        (Vuln::PathTraversal, _, false) => (
            format!(
                "app.get('/{route}', (req, res) => {{\n  const file = path.join(__dirname, 'uploads', req.query.{param});\n  res.sendFile(file);\n}});\n"
            ),
            "javascript",
        ),
        (Vuln::Ssrf, 3, _) => (
            format!(
                "@app.post(\"/{route}\")\ndef preview():\n    url = request.json[\"{link}\"]\n    if not url.startswith(\"https://{allowed}\"):\n        abort(400)\n    resp = requests.get(url, timeout=5)\n    return resp.content\n"
            ),
            "python",
        ),
        (Vuln::Ssrf, _, true) => (
            format!(
                "@app.post(\"/{route}\")\ndef preview():\n    url = request.json[\"{link}\"]\n    resp = requests.get(url, timeout=5)\n    return {{\"title\": extract_title(resp.text)}}\n"
            ),
            "python",
        ),
        (Vuln::Ssrf, _, false) => (
            format!(
                "app.post('/{route}', async (req, res) => {{\n  const r = await fetch(req.body.{link});\n  res.json({{ status: r.status, body: await r.text() }});\n}});\n"
            ),
            "javascript",
        ),
        (Vuln::OpenRedirect, _, true) => (
            format!(
                "@app.get(\"/{route}\")\ndef after_login():\n    target = request.args.get(\"{next}\", \"/\")\n    return redirect(target)\n"
            ),
            "python",
        ),
        (Vuln::OpenRedirect, _, false) => (
            format!(
                "app.get('/{route}', (req, res) => {{\n  res.redirect(req.query.{next} || '/');\n}});\n"
            ),
            "javascript",
        ),
        (Vuln::Idor, _, _) => (
            format!(
                "@app.get(\"/api/{table}/<int:item_id>\")\n@login_required\ndef get_item(item_id):\n    item = {}.query.get_or_404(item_id)\n    return jsonify(item.to_dict())\n",
                capitalize(table)
            ),
            "python",
        ),
        (Vuln::Deserialization, _, _) => (
            format!(
                "@app.get(\"/{route}\")\ndef restore():\n    raw = base64.b64decode(request.cookies[\"state\"])\n    state = pickle.loads(raw)\n    return render_template(\"{route}.html\", **state)\n"
            ),
            "python",
        ),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn vulnerability_class(level: u8, rng: &mut Rng) -> Draft {
    let pool: &[Vuln] = match level {
        1 => &[Vuln::Sqli, Vuln::Xss, Vuln::CommandInjection],
        2 => &[Vuln::PathTraversal, Vuln::Ssrf, Vuln::OpenRedirect],
        _ => &[
            Vuln::Idor,
            Vuln::Sqli,
            Vuln::Xss,
            Vuln::PathTraversal,
            Vuln::Ssrf,
            Vuln::Deserialization,
        ],
    };
    let vuln = *pick(rng, pool);
    let names = Names {
        route: pick(
            rng,
            &[
                "search", "reports", "export", "profile", "tools", "files", "lookup", "status",
                "share", "archive", "media", "billing",
            ],
        ),
        param: pick(
            rng,
            &[
                "name", "query", "term", "target", "doc", "host", "q", "value", "input", "keyword",
            ],
        ),
        table: pick(
            rng,
            &[
                "invoices",
                "orders",
                "documents",
                "tickets",
                "contracts",
                "payments",
                "reports",
                "messages",
            ],
        ),
        link: pick(rng, &["url", "image_url", "feed_url", "source", "callback"]),
        next: pick(
            rng,
            &["next", "return_to", "redirect_uri", "back_url", "dest"],
        ),
        allowed: pick(
            rng,
            &[
                "img.example.ru",
                "cdn.example.ru",
                "static.example.ru",
                "media.example.ru",
            ],
        ),
    };
    let python = chance(rng, 0.6);
    let (code, language) = snippet(vuln, level, python, &names);
    let near: &[Vuln] = match vuln {
        Vuln::Sqli => &[Vuln::CommandInjection, Vuln::Xss, Vuln::Idor],
        Vuln::Xss => &[Vuln::Sqli, Vuln::OpenRedirect, Vuln::Ssrf],
        Vuln::CommandInjection => &[Vuln::Sqli, Vuln::PathTraversal, Vuln::Ssrf],
        Vuln::PathTraversal => &[Vuln::Idor, Vuln::CommandInjection, Vuln::Ssrf],
        Vuln::Ssrf => &[Vuln::OpenRedirect, Vuln::PathTraversal, Vuln::Xss],
        Vuln::OpenRedirect => &[Vuln::Ssrf, Vuln::Xss, Vuln::PathTraversal],
        Vuln::Idor => &[Vuln::Sqli, Vuln::PathTraversal, Vuln::Deserialization],
        Vuln::Deserialization => &[Vuln::CommandInjection, Vuln::Xss, Vuln::Idor],
    };
    let mut distractors: Vec<String> = pick_n(rng, near, 2)
        .iter()
        .map(|v| v.label().to_owned())
        .collect();
    distractors.extend(
        shuffled(rng, Vuln::ALL.to_vec())
            .into_iter()
            .filter(|v| *v != vuln)
            .map(|v| v.label().to_owned()),
    );
    let note = if vuln == Vuln::Idor {
        " Декоратор `@login_required` проверяет только, что пользователь вошёл в систему."
    } else {
        ""
    };
    let question = *pick(
        rng,
        &[
            "Какая уязвимость есть в этом фрагменте?",
            "К какому классу относится уязвимость в этом коде?",
            "Код ниже попал на ревью. Какую уязвимость в нём нужно отметить?",
            "Какая уязвимость присутствует в этом коде?",
        ],
    );
    Draft::choice(format!("{question}{note}"), vuln.label(), distractors).code(language, code)
}

/// Fictional organisations for scenario wording; generic nouns, deliberately not real brands.
const COMPANIES: &[&str] = &[
    "«Вектор»",
    "«Орбита»",
    "«Кедр»",
    "«Пульсар»",
    "«Гранит»",
    "«Сигма»",
    "«Меридиан»",
    "«Альтаир»",
    "«Бриз»",
    "«Ладога»",
    "«Зенит»",
    "«Комета»",
    "«Тайга»",
    "«Фарватер»",
    "«Импульс»",
    "«Сфера»",
];

/// (who issues the secrets, nominative and genitive of one secret).
const SECRET_ISSUERS: &[(&str, &str, &str)] = &[
    (
        "менеджер паролей генерирует пароли для сотрудников",
        "пароль",
        "пароля",
    ),
    (
        "сервис аутентификации выдаёт временные пароли",
        "пароль",
        "пароля",
    ),
    (
        "скрипт развёртывания создаёт пароли для баз данных",
        "пароль",
        "пароля",
    ),
    (
        "сервис сброса доступа отправляет токены восстановления",
        "токен",
        "токена",
    ),
    (
        "система SSO выдаёт одноразовые токены входа",
        "токен",
        "токена",
    ),
    (
        "платформа выпускает API-ключи для интеграций",
        "ключ",
        "ключа",
    ),
    (
        "бэкенд мобильного приложения выпускает ключи устройств",
        "ключ",
        "ключа",
    ),
    ("интернет-магазин рассылает коды приглашения", "код", "кода"),
    (
        "CI-пайплайн создаёт секреты для тестовых стендов",
        "секрет",
        "секрета",
    ),
    (
        "сервис вебхуков генерирует секреты для подписи запросов",
        "секрет",
        "секрета",
    ),
];

fn entropy_question(rng: &mut Rng, nom: &str, gen_: &str) -> String {
    match range(rng, 0, 2) {
        0 => format!("Какова энтропия одного {gen_} в битах?"),
        1 => format!("Сколько бит энтропии содержит один {nom}?"),
        _ => format!("Оцените энтропию одного {gen_} в битах."),
    }
}

fn password_entropy(level: u8, rng: &mut Rng) -> Draft {
    let company = *pick(rng, COMPANIES);
    match level {
        1 => {
            let (text, n) = *pick(
                rng,
                &[
                    ("цифр (10 символов)", 10u32),
                    ("шестнадцатеричных цифр 0–9a–f (16 символов)", 16),
                    ("строчных латинских букв (26 символов)", 26),
                    ("заглавных латинских букв (26 символов)", 26),
                    ("алфавита Base32: A–Z и 2–7 (32 символа)", 32),
                    ("строчных латинских букв и цифр (36 символов)", 36),
                    ("заглавных латинских букв и цифр (36 символов)", 36),
                    ("строчных и заглавных латинских букв (52 символа)", 52),
                    ("алфавита Base58 (58 символов)", 58),
                    ("латинских букв обоих регистров и цифр (62 символа)", 62),
                    ("алфавита Base64: A–Z, a–z, 0–9, + и / (64 символа)", 64),
                ],
            );
            let (issuer, nom, gen_) = *pick(rng, SECRET_ISSUERS);
            let len = range(rng, 6, 32);
            let len_text = plural_ru(len, "символ", "символа", "символов");
            let question = entropy_question(rng, nom, gen_);
            entropy_draft(
                format!(
                    "В компании {company} {issuer}. Каждый {nom} — строка длиной {len_text}, каждый символ выбирается случайно и независимо из {text}. {question}"
                ),
                len as f64 * f64::from(n).log2(),
            )
        }
        2 => {
            let (text, n) = *pick(
                rng,
                &[
                    ("строчные и заглавные латинские буквы", 52u32),
                    ("строчные и заглавные латинские буквы и цифры", 62),
                    ("все 94 печатных символа ASCII без пробела", 94),
                    ("строчные латинские буквы, цифры и 10 спецсимволов", 46),
                    ("заглавные латинские буквы и цифры", 36),
                    ("строчные латинские буквы и цифры", 36),
                    ("строчные и заглавные латинские буквы, цифры, `-` и `_`", 64),
                    ("заглавные латинские буквы, цифры и 4 спецсимвола", 40),
                    ("строчные латинские буквы и 6 спецсимволов", 32),
                ],
            );
            let (issuer, nom, gen_) = *pick(rng, SECRET_ISSUERS);
            let len = range(rng, 8, 30);
            let len_text = plural_ru(len, "символ", "символа", "символов");
            let question = entropy_question(rng, nom, gen_);
            entropy_draft(
                format!(
                    "В компании {company} {issuer}. Каждый {nom} имеет длину {len_text}: каждый символ выбирается случайно и независимо, алфавит — {text}. {question}"
                ),
                len as f64 * f64::from(n).log2(),
            )
        }
        _ => {
            let dict = *pick(rng, &[1296i64, 2048, 4096, 7776, 8192, 10000]);
            let n = *pick(rng, &[26i64, 36, 52, 62, 94]);
            let purpose = *pick(
                rng,
                &[
                    "мастер-пароля менеджера паролей",
                    "ключа шифрования резервных копий",
                    "входа в корпоративный VPN",
                    "шифрования ноутбуков",
                    "защиты закрытых SSH-ключей",
                    "доступа к хранилищу секретов",
                ],
            );
            let len = range(rng, 10, 24);
            let target = len as f64 * (n as f64).log2();
            // Alphabet and dictionary can be powers of a common base (36 and 7776 are powers of 6),
            // making the exact ratio an integer; the epsilon keeps float noise from adding a word.
            let words = (target / (dict as f64).log2() - 1e-9).ceil() as i64;
            let dict_text = plural_ru(dict, "слово", "слова", "слов");
            let alphabet = plural_ru(n, "символ", "символа", "символов");
            let len_text = plural_ru(len, "символ", "символа", "символов");
            let question = if chance(rng, 0.5) {
                "Сколько слов нужно минимум"
            } else {
                "Какое наименьшее число слов нужно"
            };
            Draft::integer(
                format!(
                    "В компании {company} для {purpose} вместо случайных паролей вводят парольные фразы: слова выбираются случайно и независимо из словаря на {dict_text}. \
                     {question}, чтобы энтропия фразы была не меньше, чем у случайного пароля длиной {len_text} из алфавита в {alphabet}?"
                ),
                words,
            )
        }
    }
}

fn entropy_draft(prompt: String, bits: f64) -> Draft {
    Draft::number(prompt, (bits * 10.0).round() / 10.0, 1.0)
        .hint("Число бит, допустимая погрешность — 1 бит")
}

fn hex(b: u8) -> String {
    format!("0x{b:02X}")
}

const WORDS: &[&str] = &[
    "admin", "token", "login", "flags", "debug", "proxy", "cache", "shell", "agent", "patch",
    "root", "user", "pass", "salt", "hash", "seed", "port", "host", "sudo", "data",
];

fn xor_cipher(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 => {
            let p = range(rng, 1, 255) as u8;
            let k = range(rng, 1, 255) as u8;
            let answer = p ^ k;
            let distractors = [
                p.wrapping_add(k),
                p & k,
                p | k,
                !answer,
                answer ^ 0x10,
                answer ^ 0x01,
            ];
            Draft::choice(
                format!(
                    "Байт открытого текста {} зашифровали операцией XOR с ключом {}. Каким будет байт шифротекста?",
                    hex(p),
                    hex(k)
                ),
                hex(answer),
                distractors.iter().map(|d| hex(*d)),
            )
        }
        2 => {
            let ch = *pick(
                rng,
                &[
                    'A', 'Z', 'a', 'z', '0', '9', '{', '[', '<', '%', '#', 'f', 'P', 'G', 'M', 'H',
                    'R', 's', 'x', 'k',
                ],
            );
            let p = ch as u8;
            let k = range(rng, 1, 255) as u8;
            let c = p ^ k;
            let distractors = [c.wrapping_sub(p), c.wrapping_add(p), !k, k ^ 0x20, c, p];
            Draft::choice(
                format!(
                    "Сообщение зашифровано однобайтовым XOR-ключом. Известно, что первый символ открытого текста — `{ch}` ({}), а первый байт шифротекста — {}. Каков ключ?",
                    hex(p),
                    hex(c)
                ),
                hex(k),
                distractors.iter().map(|d| hex(*d)),
            )
        }
        _ => {
            let len = *pick(rng, &[4usize, 5]);
            let words: Vec<&str> = WORDS.iter().copied().filter(|w| w.len() == len).collect();
            let word = *pick(rng, &words);
            let key = [range(rng, 1, 255) as u8, range(rng, 1, 255) as u8];
            let cipher: Vec<String> = word
                .bytes()
                .enumerate()
                .map(|(i, b)| format!("{:02X}", b ^ key[i % 2]))
                .collect();
            let first = word.as_bytes()[0];
            let mut others: Vec<&str> = words.iter().copied().filter(|w| *w != word).collect();
            others.sort_by_key(|w| u8::from(w.as_bytes()[0] != first));
            let same_start: Vec<&str> = others
                .iter()
                .copied()
                .take_while(|w| w.as_bytes()[0] == first)
                .collect();
            let mut distractors: Vec<String> = same_start.iter().map(|w| (*w).to_owned()).collect();
            distractors.extend(shuffled(rng, others).into_iter().map(str::to_owned));
            Draft::choice(
                format!(
                    "ASCII-строку зашифровали повторяющимся XOR-ключом из двух байт `{:02X} {:02X}` (первый байт ключа — к первому символу, второй — ко второму и так далее по кругу). \
                     Шифротекст: `{}`. Какая строка была зашифрована?",
                    key[0],
                    key[1],
                    cipher.join(" ")
                ),
                word,
                distractors,
            )
        }
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

const LOGINS: &[&str] = &[
    "admin", "root", "ivanov", "deploy", "backup", "svc_api", "guest", "operator", "petrova",
    "jenkins", "monitor", "billing", "support", "sidorov", "devops", "reporter",
];
const SECRETS: &[&str] = &[
    "qwerty",
    "P@ssw0rd",
    "letmein",
    "summer2025",
    "hunter2",
    "Secret!1",
    "changeme",
    "welcome1",
    "admin123",
    "Passw0rd!",
    "zaq12wsx",
    "Winter#24",
    "1q2w3e4r",
    "dragon77",
];

/// Genitive phrases for what gets encoded; all of them are raw bytes, so length is given in bytes.
const ENCODED: &[&str] = &[
    "строки",
    "бинарного ключа",
    "случайной соли",
    "цифровой подписи",
    "вложения",
    "сессионного токена",
    "идентификатора",
    "фрагмента файла",
    "зашифрованного блока",
    "одноразового значения (nonce)",
];

fn base64_item(level: u8, rng: &mut Rng) -> Draft {
    if level == 1 {
        let n = range(rng, 1, 64);
        let size = if chance(rng, 0.5) {
            plural_ru(n, "байт", "байта", "байт")
        } else {
            plural_ru(n, "символ ASCII", "символа ASCII", "символов ASCII")
        };
        let what = *pick(rng, ENCODED);
        let company = *pick(rng, COMPANIES);
        let channel = *pick(
            rng,
            &[
                "в JSON-поле ответа API",
                "в HTTP-заголовке",
                "в теле письма (MIME)",
                "в поле XML-документа",
                "в значении cookie",
                "в конфигурационном файле",
            ],
        );
        return Draft::integer(
            format!(
                "Сервис компании {company} передаёт данные {channel}, кодируя их стандартным Base64 (алфавит `A–Z a–z 0–9 + /`, с дополнением `=`). \
                 Сколько символов (включая `=` в конце) займёт Base64-представление {what} длиной {size}?"
            ),
            (n + 2) / 3 * 4,
        );
    }
    let logins = pick_n(rng, LOGINS, 4);
    let secret = *pick(rng, SECRETS);
    let creds = format!("{}:{secret}", logins[0]);
    let encoded = base64(creds.as_bytes());
    let distractors = vec![
        format!("{}:{secret}", logins[1]),
        format!("{}:{secret}", logins[2]),
        format!("{}:{secret}", logins[3]),
        "Без ключа сервера расшифровать нельзя".to_owned(),
    ];
    let place = *pick(
        rng,
        &[
            "В перехваченном HTTP-запросе",
            "В логе обратного прокси",
            "В HAR-файле, приложенном к баг-репорту,",
            "В дампе трафика из Wireshark",
        ],
    );
    Draft::choice(
        format!(
            "{place} есть заголовок `Authorization: Basic {encoded}`. Какие учётные данные он передаёт?"
        ),
        creds,
        distractors,
    )
}

const PORTS: &[(u8, &str, &str)] = &[
    (1, "22/tcp", "SSH"),
    (1, "21/tcp", "FTP"),
    (1, "25/tcp", "SMTP"),
    (1, "53/udp", "DNS"),
    (1, "80/tcp", "HTTP"),
    (1, "443/tcp", "HTTPS"),
    (1, "3306/tcp", "MySQL"),
    (1, "5432/tcp", "PostgreSQL"),
    (1, "3389/tcp", "RDP"),
    (2, "23/tcp", "Telnet"),
    (2, "110/tcp", "POP3"),
    (2, "143/tcp", "IMAP"),
    (2, "445/tcp", "SMB"),
    (2, "389/tcp", "LDAP"),
    (2, "6379/tcp", "Redis"),
    (2, "27017/tcp", "MongoDB"),
    (2, "9200/tcp", "Elasticsearch (HTTP API)"),
    (2, "1433/tcp", "Microsoft SQL Server"),
    (2, "5672/tcp", "RabbitMQ (AMQP)"),
    (2, "11211/tcp", "Memcached"),
    (3, "88/tcp", "Kerberos"),
    (3, "161/udp", "SNMP"),
    (3, "636/tcp", "LDAPS"),
    (3, "2375/tcp", "Docker Engine API без TLS"),
    (3, "2379/tcp", "etcd (клиентский API)"),
    (3, "6443/tcp", "Kubernetes API server"),
    (3, "10250/tcp", "Kubelet API"),
    (3, "9092/tcp", "Kafka"),
    (3, "5601/tcp", "Kibana"),
    (3, "9100/tcp", "Prometheus node_exporter"),
];

fn service_ports(level: u8, rng: &mut Rng) -> Draft {
    let same: Vec<&(u8, &str, &str)> = PORTS.iter().filter(|p| p.0 == level).collect();
    let target = *pick(rng, &same);
    let mut others: Vec<&(u8, &str, &str)> =
        same.iter().copied().filter(|p| p.1 != target.1).collect();
    others = shuffled(rng, others);
    let host = format!(
        "10.{}.{}.{}",
        range(rng, 0, 255),
        range(rng, 0, 255),
        range(rng, 2, 254)
    );
    if chance(rng, 0.5) {
        let observed = match range(rng, 0, 3) {
            0 => format!(
                "Сканер (nmap) показал на хосте `{host}` открытый порт `{}`.",
                target.1
            ),
            1 => format!(
                "В выводе `ss -lntu` на сервере `{host}` виден слушающий порт `{}`.",
                target.1
            ),
            2 => format!(
                "IDS зафиксировала входящие подключения к `{host}` на порт `{}`.",
                target.1
            ),
            _ => format!(
                "В отчёте об инвентаризации для `{host}` указан открытый порт `{}`.",
                target.1
            ),
        };
        let question = *pick(
            rng,
            &[
                "Какой сервис обычно работает на нём по умолчанию?",
                "Какой сервис по умолчанию использует этот порт?",
                "Какой сервис, скорее всего, его слушает?",
            ],
        );
        Draft::choice(
            format!("{observed} {question}"),
            target.2,
            others.iter().map(|p| p.2.to_owned()),
        )
    } else {
        let question = match range(rng, 0, 3) {
            0 => format!("Какой порт по умолчанию использует {}?", target.2),
            1 => format!("На каком порту по умолчанию слушает {}?", target.2),
            2 => format!(
                "На хосте `{host}` поднимают сервис {} с настройками по умолчанию. Какой порт нужно открыть в межсетевом экране?",
                target.2
            ),
            _ => format!(
                "Аудитор проверяет, доступен ли снаружи сервис {} на хосте `{host}`. Какой порт он проверит в первую очередь (порт по умолчанию)?",
                target.2
            ),
        };
        Draft::choice(question, target.1, others.iter().map(|p| p.1.to_owned()))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Accept,
    Drop,
    Reject,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Action::Accept => "ACCEPT",
            Action::Drop => "DROP",
            Action::Reject => "REJECT",
        }
    }
}

#[derive(Clone, Copy)]
struct Rule {
    tcp: Option<bool>,
    source: Option<(u32, u32)>,
    ports: Option<(u16, u16)>,
    established: bool,
    action: Action,
}

#[derive(Clone, Copy)]
struct Packet {
    tcp: bool,
    source: u32,
    port: u16,
    established: bool,
}

fn ipv4(v: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        v >> 24,
        (v >> 16) & 255,
        (v >> 8) & 255,
        v & 255
    )
}

fn net_mask(prefix: u32) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

impl Rule {
    fn matches(&self, p: &Packet) -> bool {
        self.tcp.is_none_or(|t| t == p.tcp)
            && self
                .source
                .is_none_or(|(net, prefix)| p.source & net_mask(prefix) == net)
            && self
                .ports
                .is_none_or(|(lo, hi)| (lo..=hi).contains(&p.port))
            && (!self.established || p.established)
    }

    fn render(&self) -> String {
        let mut s = "iptables -A INPUT".to_owned();
        if let Some(tcp) = self.tcp {
            s.push_str(if tcp { " -p tcp" } else { " -p udp" });
        }
        if let Some((net, prefix)) = self.source {
            s.push_str(&format!(" -s {}/{prefix}", ipv4(net)));
        }
        if let Some((lo, hi)) = self.ports {
            if lo == hi {
                s.push_str(&format!(" --dport {lo}"));
            } else {
                s.push_str(&format!(" --dport {lo}:{hi}"));
            }
        }
        if self.established {
            s.push_str(" -m conntrack --ctstate ESTABLISHED,RELATED");
        }
        s.push_str(&format!(" -j {}", self.action.name()));
        s
    }
}

const OFFICE: u32 = (10 << 24) | (20 << 16);
const ADMINS: u32 = (10 << 24) | (20 << 16) | (5 << 8);
const PARTNER: u32 = (192 << 24) | (168 << 16) | (77 << 8);

fn firewall(level: u8, rng: &mut Rng) -> Draft {
    let rule = |tcp, source, ports, action| Rule {
        tcp: Some(tcp),
        source,
        ports,
        established: false,
        action,
    };
    let (rules, policy): (Vec<Rule>, Action) = match level {
        1 => (
            shuffled(
                rng,
                vec![
                    rule(true, Some((ADMINS, 24)), Some((22, 22)), Action::Accept),
                    rule(true, None, Some((443, 443)), Action::Accept),
                    rule(
                        true,
                        Some((PARTNER, 24)),
                        Some((5432, 5432)),
                        Action::Accept,
                    ),
                ],
            )
            .into_iter()
            .chain([Rule {
                tcp: None,
                source: None,
                ports: None,
                established: false,
                action: Action::Drop,
            }])
            .collect(),
            Action::Accept,
        ),
        2 => (
            shuffled(
                rng,
                vec![
                    rule(true, Some((OFFICE, 16)), Some((22, 22)), Action::Drop),
                    rule(true, Some((ADMINS, 24)), Some((22, 22)), Action::Accept),
                    rule(true, None, Some((8000, 8100)), Action::Reject),
                    rule(true, Some((OFFICE, 16)), Some((8000, 8100)), Action::Accept),
                    rule(true, None, Some((443, 443)), Action::Accept),
                ],
            ),
            Action::Drop,
        ),
        _ => {
            let mut rs = vec![
                Rule {
                    tcp: None,
                    source: None,
                    ports: None,
                    established: true,
                    action: Action::Accept,
                },
                rule(true, Some((ADMINS, 24)), Some((22, 22)), Action::Accept),
                rule(false, None, Some((53, 53)), Action::Accept),
                rule(true, None, Some((443, 443)), Action::Accept),
                rule(true, Some((OFFICE, 16)), Some((5432, 5432)), Action::Reject),
            ];
            let tail = shuffled(rng, rs.split_off(1));
            rs.extend(tail);
            (rs, Action::Drop)
        }
    };
    let ports: Vec<u16> = vec![22, 443, 5432, 8080, 53, 3306];
    let sources = [
        ADMINS | range(rng, 1, 254) as u32,
        OFFICE | ((range(rng, 6, 200) as u32) << 8) | range(rng, 1, 254) as u32,
        PARTNER | range(rng, 1, 254) as u32,
        (203 << 24) | (range(rng, 0, 255) as u32) << 8 | range(rng, 1, 254) as u32,
    ];
    let packet = Packet {
        tcp: !(level == 3 && chance(rng, 0.2)),
        source: *pick(rng, &sources),
        port: *pick(rng, &ports),
        established: level == 3 && chance(rng, 0.3),
    };
    let matched = rules.iter().position(|r| r.matches(&packet));
    let option = |i: Option<usize>| match i {
        Some(i) => format!("{} по правилу {}", rules[i].action.name(), i + 1),
        None => format!("{} по политике цепочки", policy.name()),
    };
    let answer = option(matched);
    let mut distractors: Vec<String> = (0..rules.len())
        .filter(|i| Some(*i) != matched)
        .map(|i| option(Some(i)))
        .collect();
    if matched.is_some() {
        distractors.push(option(None));
    }
    let distractors = shuffled(rng, distractors);
    let script = format!(
        "iptables -P INPUT {}\n{}\n",
        policy.name(),
        rules
            .iter()
            .map(Rule::render)
            .collect::<Vec<_>>()
            .join("\n")
    );
    let kind = if packet.established {
        "ответный пакет уже установленного соединения"
    } else {
        "пакет нового соединения"
    };
    let proto = if packet.tcp { "TCP" } else { "UDP" };
    Draft::choice(
        format!(
            "На сервере настроена цепочка INPUT (правила нумеруются сверху вниз, срабатывает первое подходящее). Что произойдёт с входящим пакетом: {kind}, {proto}, источник {}, порт назначения {}?",
            ipv4(packet.source),
            packet.port
        ),
        answer,
        distractors,
    )
    .code("bash", script)
}

struct Payload {
    language: &'static str,
    code: String,
    input: String,
    steps: Vec<(&'static str, &'static str)>,
    sink: String,
}

fn nest(token: &str, depth: usize, split: usize) -> String {
    let mut s = token.to_owned();
    for _ in 0..depth {
        s = format!("{}{}{}", &token[..split], s, &token[split..]);
    }
    s
}

fn sanitize(input: &str, steps: &[(&str, &str)]) -> String {
    steps.iter().fold(input.to_owned(), |acc, (from, to)| {
        py_replace(&acc, from, to)
    })
}

fn sanitize_fixpoint(input: &str, steps: &[(&str, &str)]) -> String {
    let mut current = input.to_owned();
    for _ in 0..32 {
        let next = sanitize(&current, steps);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

fn filter_bypass(level: u8, rng: &mut Rng) -> Draft {
    let traversal = chance(rng, 0.5);
    let payload = if traversal {
        // `nest("../", 1, 1)` is `..././` and `nest("../", 1, 2)` is `....//`: a single
        // non-overlapping `.replace("../", "")` collapses each unit back to `../`.
        let unit = nest("../", 1, range(rng, 1, 2) as usize);
        let hops = range(rng, 2, 3) as usize;
        let target = *pick(
            rng,
            &[
                "etc/app.conf",
                "data/report.csv",
                "var/log/service.log",
                "config/settings.yaml",
                "logs/audit.log",
                "backup/db.sql",
                "data/users.csv",
            ],
        );
        let func = *pick(
            rng,
            &[
                "read_report",
                "load_template",
                "get_attachment",
                "open_invoice",
                "read_log",
                "export_file",
                "show_document",
            ],
        );
        let param = *pick(
            rng,
            &["name", "filename", "path", "doc", "file_name", "item"],
        );
        let base = *pick(
            rng,
            &[
                "/srv/reports",
                "/var/app/uploads",
                "/opt/data/files",
                "/srv/templates",
                "/var/lib/app/attachments",
                "/home/app/storage",
            ],
        );
        let mut steps = vec![("../", "")];
        if level == 2 {
            steps.push(("..\\", ""));
        }
        let input = format!("{}{target}", unit.repeat(hops));
        Payload {
            language: "python",
            code: format!(
                "def {func}({param}):\n    safe = {param}{}\n    with open(os.path.join(\"{base}\", safe)) as f:\n        return f.read()\n",
                steps
                    .iter()
                    .map(|(a, b)| format!(".replace({a:?}, {b:?})"))
                    .collect::<String>()
            ),
            input,
            steps,
            sink: "Какой путь (аргумент `safe`) будет передан в `os.path.join`?".to_owned(),
        }
    } else {
        let open = nest("<script>", 1, range(rng, 2, 6) as usize);
        let close = nest("</script>", 1, range(rng, 2, 7) as usize);
        let steps = vec![("<script>", ""), ("</script>", "")];
        let payload_body = if level == 2 {
            *pick(
                rng,
                &[
                    "alert(document.cookie)",
                    "alert(document.domain)",
                    "console.log(document.cookie)",
                ],
            )
        } else {
            *pick(
                rng,
                &[
                    "alert(1)",
                    "alert(42)",
                    "alert('xss')",
                    "confirm(1)",
                    "print()",
                ],
            )
        };
        let func = *pick(
            rng,
            &[
                "clean_comment",
                "sanitize_bio",
                "strip_scripts",
                "clean_review",
                "safe_title",
                "clean_message",
            ],
        );
        let param = *pick(rng, &["text", "comment", "body", "bio", "message", "value"]);
        let input = format!("{open}{payload_body}{close}");
        Payload {
            language: "python",
            code: format!(
                "def {func}({param}):\n    return {param}{}\n",
                steps
                    .iter()
                    .map(|(a, b)| format!(".replace({a:?}, {b:?})"))
                    .collect::<String>()
            ),
            input,
            steps,
            sink: format!("Что вернёт функция `{func}` для этого ввода?"),
        }
    };
    let answer = sanitize(&payload.input, &payload.steps);
    let mut reversed = payload.steps.clone();
    reversed.reverse();
    let candidates = vec![
        sanitize_fixpoint(&payload.input, &payload.steps),
        payload.input.clone(),
        sanitize(&payload.input, &reversed),
        sanitize(&payload.input, &payload.steps[..1]),
        payload
            .steps
            .iter()
            .fold(payload.input.clone(), |acc, (a, b)| acc.replacen(a, b, 1)),
        sanitize(&answer, &payload.steps),
    ];
    let mut distractors: Vec<String> = candidates.into_iter().filter(|c| *c != answer).collect();
    distractors.push(format!("{answer}/"));
    distractors.push(answer.chars().skip(1).collect());
    let fmt = |s: &str| format!("`{s}`");
    let source = *pick(
        rng,
        &[
            "Пользователь передал строку",
            "На вход функции пришла строка",
            "Тестировщик безопасности отправил строку",
        ],
    );
    Draft::choice(
        format!("{source} `{}`. {}", payload.input, payload.sink),
        fmt(&answer),
        distractors.iter().map(|d| fmt(d)),
    )
    .code(payload.language, payload.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encoding() {
        assert_eq!(base64(b"admin:qwerty"), "YWRtaW46cXdlcnR5");
        assert_eq!(base64(b"a"), "YQ==");
        assert_eq!(base64(b"ab"), "YWI=");
        assert_eq!(base64(b""), "");
    }

    #[test]
    fn sanitizer_bypass() {
        let steps = [("../", "")];
        assert_eq!(
            sanitize("....//....//etc/passwd", &steps),
            "../../etc/passwd"
        );
        assert_eq!(
            sanitize_fixpoint("....//....//etc/passwd", &steps),
            "etc/passwd"
        );
        assert_eq!(nest("../", 1, 2), "....//");
        assert_eq!(
            sanitize(&nest("<script>", 1, 3), &[("<script>", "")]),
            "<script>"
        );
    }

    #[test]
    fn firewall_first_match() {
        let drop_office = Rule {
            tcp: Some(true),
            source: Some((OFFICE, 16)),
            ports: Some((22, 22)),
            established: false,
            action: Action::Drop,
        };
        let allow_admins = Rule {
            tcp: Some(true),
            source: Some((ADMINS, 24)),
            ports: Some((22, 22)),
            established: false,
            action: Action::Accept,
        };
        let packet = Packet {
            tcp: true,
            source: ADMINS | 7,
            port: 22,
            established: false,
        };
        let rules = [drop_office, allow_admins];
        assert_eq!(rules.iter().position(|r| r.matches(&packet)), Some(0));
        assert!(!drop_office.matches(&Packet {
            source: PARTNER | 1,
            ..packet
        }));
    }
}
