//! DevOps templates: containers, scheduling, networking, nginx, cron, permissions, monitoring.

use catalog::Specialization::{Backend, Devops, Security};

use super::util::{
    PEOPLE, chance, fmt_number, join, md_table, pick, pick_n, plural_ru, range, shuffled,
};
use super::{C_CHOICE, C_MULTI5, C_OPEN, Draft, Level, Rng, Source, Template, Topic};

pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        id: "docker.layer_cache",
        title: "Кэш слоёв Docker",
        topic: Topic::Containers,
        specializations: &[Devops, Backend],
        skills: &["docker", "ci_cd"],
        levels: &[
            Level::new(1, 1.0, -0.6, C_CHOICE, 90),
            Level::new(2, 1.3, 0.2, C_CHOICE, 120),
            Level::new(3, 1.4, 0.9, C_CHOICE, 120),
        ],
        source: Source::Computed(layer_cache),
    },
    Template {
        id: "k8s.resource_fit",
        title: "Размещение подов по requests",
        topic: Topic::Containers,
        specializations: &[Devops],
        skills: &["kubernetes"],
        levels: &[
            Level::new(1, 1.0, -0.4, C_OPEN, 120),
            Level::new(2, 1.3, 0.4, C_OPEN, 150),
            Level::new(3, 1.5, 1.1, C_OPEN, 180),
        ],
        source: Source::Computed(resource_fit),
    },
    Template {
        id: "net.cidr",
        title: "Подсети IPv4 и CIDR",
        topic: Topic::Networking,
        specializations: &[Devops, Security],
        skills: &["networking"],
        levels: &[
            Level::new(1, 1.0, -0.9, C_OPEN, 90),
            Level::new(2, 1.3, 0.0, C_CHOICE, 120),
            Level::new(3, 1.5, 0.8, C_MULTI5, 150),
        ],
        source: Source::Computed(cidr),
    },
    Template {
        id: "nginx.location_match",
        title: "Выбор location в nginx",
        topic: Topic::Networking,
        specializations: &[Devops],
        skills: &["nginx", "http"],
        levels: &[
            Level::new(1, 1.0, -0.3, C_CHOICE, 120),
            Level::new(2, 1.3, 0.5, C_CHOICE, 150),
            Level::new(3, 1.5, 1.3, C_CHOICE, 150),
        ],
        source: Source::Computed(nginx_location),
    },
    Template {
        id: "cron.next_run",
        title: "Следующий запуск по cron",
        topic: Topic::Linux,
        specializations: &[Devops],
        skills: &["linux", "bash"],
        levels: &[
            Level::new(1, 1.0, -0.7, C_CHOICE, 90),
            Level::new(2, 1.2, 0.1, C_CHOICE, 120),
            Level::new(3, 1.4, 0.9, C_CHOICE, 150),
        ],
        source: Source::Computed(cron_next),
    },
    Template {
        id: "linux.permissions",
        title: "Права доступа в Linux",
        topic: Topic::Linux,
        specializations: &[Devops, Security],
        skills: &["linux", "bash"],
        levels: &[
            Level::new(1, 1.0, -1.0, C_OPEN, 60),
            Level::new(2, 1.2, -0.2, C_OPEN, 90),
            Level::new(3, 1.4, 0.7, C_CHOICE, 90),
        ],
        source: Source::Computed(permissions),
    },
    Template {
        id: "prom.counter_increase",
        title: "Счётчики Prometheus и сбросы",
        topic: Topic::Observability,
        specializations: &[Devops],
        skills: &["prometheus"],
        levels: &[
            Level::new(1, 0.9, -0.8, C_OPEN, 90),
            Level::new(2, 1.2, 0.2, C_OPEN, 120),
            Level::new(3, 1.4, 1.0, C_OPEN, 150),
        ],
        source: Source::Computed(counter_increase),
    },
    Template {
        id: "sre.error_budget",
        title: "SLO, бюджет ошибок и доступность",
        topic: Topic::Observability,
        specializations: &[Devops],
        skills: &["prometheus", "system_design"],
        levels: &[
            Level::new(1, 1.0, -0.5, C_OPEN, 90),
            Level::new(2, 1.2, 0.3, C_OPEN, 120),
            Level::new(3, 1.4, 1.1, C_OPEN, 150),
        ],
        source: Source::Computed(error_budget),
    },
];

const SERVICES: &[&str] = &[
    "checkout",
    "payments",
    "auth",
    "search",
    "catalog",
    "orders",
    "notifications",
    "billing",
    "profile",
    "delivery",
    "inventory",
    "recommendations",
    "media",
    "chat",
    "gateway",
    "reports",
    "booking",
    "pricing",
    "cart",
    "loyalty",
];

#[derive(Clone)]
enum Instr {
    From(&'static str),
    Workdir(&'static str),
    Copy {
        sources: Vec<&'static str>,
        dest: &'static str,
    },
    Run(&'static str),
    Cmd(&'static str),
}

impl Instr {
    fn render(&self) -> String {
        match self {
            Instr::From(s) => format!("FROM {s}"),
            Instr::Workdir(s) => format!("WORKDIR {s}"),
            Instr::Copy { sources, dest } => format!("COPY {} {dest}", sources.join(" ")),
            Instr::Run(s) => format!("RUN {s}"),
            Instr::Cmd(s) => format!("CMD {s}"),
        }
    }

    /// Whether changing `file` invalidates this instruction's cache key.
    fn depends_on(&self, file: &str, ignored: &[&str]) -> bool {
        let Instr::Copy { sources, .. } = self else {
            return false;
        };
        if ignored.iter().any(|pattern| matches_ignore(pattern, file)) {
            return false;
        }
        sources.iter().any(|src| match *src {
            "." => true,
            dir if dir.ends_with('/') => file.starts_with(dir),
            glob if glob.ends_with("*.json") => {
                let prefix = glob.trim_end_matches("*.json");
                file.starts_with(prefix) && file.ends_with(".json") && !file.contains('/')
            }
            exact => file == exact,
        })
    }
}

fn matches_ignore(pattern: &str, file: &str) -> bool {
    if let Some(ext) = pattern.strip_prefix("*.") {
        !file.contains('/') && file.ends_with(&format!(".{ext}"))
    } else if pattern.ends_with('/') {
        file.starts_with(pattern)
    } else {
        file == pattern
    }
}

struct Stack {
    bases: &'static [&'static str],
    deps: &'static [&'static str],
    dep_copy: &'static [&'static str],
    install: &'static str,
    build: &'static str,
    cmds: &'static [&'static str],
    /// Two top-level source directories with the files inside them.
    dirs: [(&'static str, &'static [&'static str]); 2],
    /// Sources outside `dirs`: reached only by `COPY . .`.
    loose: &'static [&'static str],
}

impl Stack {
    fn sources(&self) -> Vec<&'static str> {
        self.dirs
            .iter()
            .flat_map(|(_, files)| files.iter().copied())
            .chain(self.loose.iter().copied())
            .collect()
    }
}

const PYTHON_BASES: &[&str] = &["python:3.11-slim", "python:3.12-slim", "python:3.13-slim"];

const STACKS: &[Stack] = &[
    Stack {
        bases: PYTHON_BASES,
        deps: &["requirements.txt"],
        dep_copy: &["requirements.txt"],
        install: "pip install --no-cache-dir -r requirements.txt",
        build: "python -m compileall -q app",
        cmds: &[
            "[\"python\", \"-m\", \"app\"]",
            "[\"uvicorn\", \"app.main:app\", \"--host\", \"0.0.0.0\"]",
        ],
        dirs: [
            (
                "app/",
                &[
                    "app/main.py",
                    "app/api/routes.py",
                    "app/models/user.py",
                    "app/services/billing.py",
                ],
            ),
            ("config/", &["config/settings.yaml", "config/logging.yaml"]),
        ],
        loose: &[],
    },
    Stack {
        bases: PYTHON_BASES,
        deps: &["pyproject.toml", "poetry.lock"],
        dep_copy: &["pyproject.toml", "poetry.lock"],
        install: "pip install --no-cache-dir poetry && poetry install --no-root",
        build: "python -m compileall -q service",
        cmds: &["[\"poetry\", \"run\", \"python\", \"-m\", \"service\"]"],
        dirs: [
            (
                "service/",
                &[
                    "service/__main__.py",
                    "service/handlers/orders.py",
                    "service/db/session.py",
                ],
            ),
            (
                "migrations/",
                &["migrations/env.py", "migrations/versions/0001_init.py"],
            ),
        ],
        loose: &["alembic.ini"],
    },
    Stack {
        bases: &["node:18-alpine", "node:20-alpine", "node:22-alpine"],
        deps: &["package.json", "package-lock.json"],
        dep_copy: &["package*.json"],
        install: "npm ci",
        build: "npm run build",
        cmds: &["[\"node\", \"dist/server.js\"]", "[\"npm\", \"start\"]"],
        dirs: [
            (
                "src/",
                &[
                    "src/server.ts",
                    "src/routes/users.ts",
                    "src/services/auth.ts",
                ],
            ),
            ("public/", &["public/index.html", "public/robots.txt"]),
        ],
        loose: &[],
    },
    Stack {
        bases: &["node:18-slim", "node:20-slim", "node:22-slim"],
        deps: &["package.json", "yarn.lock"],
        dep_copy: &["package.json", "yarn.lock"],
        install: "yarn install --frozen-lockfile",
        build: "yarn build",
        cmds: &["[\"node\", \"dist/index.js\"]", "[\"yarn\", \"start\"]"],
        dirs: [
            (
                "src/",
                &["src/index.ts", "src/api/orders.ts", "src/lib/cache.ts"],
            ),
            ("prisma/", &["prisma/schema.prisma", "prisma/seed.ts"]),
        ],
        loose: &[],
    },
    Stack {
        bases: &["node:20-alpine", "node:22-alpine"],
        deps: &["package.json", "pnpm-lock.yaml"],
        dep_copy: &["package.json", "pnpm-lock.yaml"],
        install: "corepack enable && pnpm install --frozen-lockfile",
        build: "pnpm run build",
        cmds: &["[\"node\", \"build/main.js\"]"],
        dirs: [
            (
                "src/",
                &[
                    "src/main.ts",
                    "src/modules/users.ts",
                    "src/common/logger.ts",
                ],
            ),
            ("views/", &["views/index.hbs", "views/layout.hbs"]),
        ],
        loose: &[],
    },
    Stack {
        bases: &["golang:1.21", "golang:1.22", "golang:1.23"],
        deps: &["go.mod", "go.sum"],
        dep_copy: &["go.mod", "go.sum"],
        install: "go mod download",
        build: "go build -o /bin/app ./cmd/app",
        cmds: &["[\"/bin/app\"]"],
        dirs: [
            ("cmd/", &["cmd/app/main.go", "cmd/app/flags.go"]),
            (
                "internal/",
                &[
                    "internal/db/db.go",
                    "internal/http/handler.go",
                    "internal/config/config.go",
                ],
            ),
        ],
        loose: &["migrations/001_init.sql"],
    },
    Stack {
        bases: &["composer:2.6", "composer:2.7"],
        deps: &["composer.json", "composer.lock"],
        dep_copy: &["composer.json", "composer.lock"],
        install: "composer install --no-dev --no-scripts --no-autoloader",
        build: "composer dump-autoload --optimize",
        cmds: &["[\"php\", \"-S\", \"0.0.0.0:8080\", \"-t\", \"public\"]"],
        dirs: [
            (
                "src/",
                &[
                    "src/Controller/UserController.php",
                    "src/Service/Mailer.php",
                ],
            ),
            ("public/", &["public/index.php", "public/favicon.ico"]),
        ],
        loose: &["config/services.yaml"],
    },
    Stack {
        bases: &["ruby:3.2-slim", "ruby:3.3-slim"],
        deps: &["Gemfile", "Gemfile.lock"],
        dep_copy: &["Gemfile", "Gemfile.lock"],
        install: "bundle install --jobs 4",
        build: "bundle exec bootsnap precompile --gemfile app/",
        cmds: &["[\"bundle\", \"exec\", \"puma\", \"-C\", \"config/puma.rb\"]"],
        dirs: [
            (
                "app/",
                &["app/models/user.rb", "app/controllers/orders_controller.rb"],
            ),
            ("config/", &["config/routes.rb", "config/puma.rb"]),
        ],
        loose: &["lib/tasks/cleanup.rake"],
    },
];

const WORKDIRS: &[&str] = &[
    "/app",
    "/src",
    "/srv/app",
    "/usr/src/app",
    "/opt/app",
    "/workspace",
];

/// Extra `.dockerignore` entries that never match any stack source.
const IGNORE_EXTRA: &[&str] = &[
    ".env",
    "*.log",
    ".github/",
    ".idea/",
    ".vscode/",
    "coverage/",
    "tmp/",
];
const IGNORED_MD: &[&str] = &["README.md", "CHANGELOG.md", "CONTRIBUTING.md"];
const IGNORED_DOCS: &[&str] = &[
    "docs/architecture.md",
    "docs/runbook.md",
    "docs/adr/0007-cache.md",
    "docs/diagrams/flow.png",
];
const IGNORED_TESTS: &[&str] = &[
    "tests/smoke.sh",
    "tests/fixtures/orders.json",
    "tests/load/checkout.js",
];

const NO_REBUILD: &str = "Ни с какой: кэш будет использован для всех инструкций";

fn layer_cache(level: u8, rng: &mut Rng) -> Draft {
    let stack = pick(rng, STACKS);
    let sources = stack.sources();
    let dep_copy = Instr::Copy {
        sources: stack.dep_copy.to_vec(),
        dest: "./",
    };
    let copy_all = || Instr::Copy {
        sources: vec!["."],
        dest: ".",
    };
    let mut instrs = vec![
        Instr::From(pick(rng, stack.bases)),
        Instr::Workdir(pick(rng, WORKDIRS)),
    ];
    let mut ignored: Vec<&str> = Vec::new();
    let changed: &str;
    match level {
        1 => {
            instrs.extend([
                dep_copy,
                Instr::Run(stack.install),
                copy_all(),
                Instr::Run(stack.build),
            ]);
            changed = if chance(rng, 0.5) {
                *pick(rng, stack.deps)
            } else {
                *pick(rng, &sources)
            };
        }
        2 => {
            if chance(rng, 0.5) {
                instrs.extend([
                    copy_all(),
                    Instr::Run(stack.install),
                    Instr::Run(stack.build),
                ]);
                changed = *pick(rng, &sources);
            } else {
                let mut dirs = stack.dirs.to_vec();
                if chance(rng, 0.5) {
                    dirs.reverse();
                }
                instrs.extend([dep_copy, Instr::Run(stack.install)]);
                for (d, _) in &dirs {
                    instrs.push(Instr::Copy {
                        sources: vec![d],
                        dest: d,
                    });
                }
                instrs.push(Instr::Run(stack.build));
                let in_dirs: Vec<&str> = dirs
                    .iter()
                    .flat_map(|(_, files)| files.iter().copied())
                    .collect();
                changed = *pick(rng, &in_dirs);
            }
        }
        _ => {
            ignored = vec!["*.md", "docs/", "tests/", ".git/"];
            let n_extra = range(rng, 0, 2) as usize;
            ignored.extend(pick_n(rng, IGNORE_EXTRA, n_extra));
            ignored = shuffled(rng, ignored);
            instrs.extend([
                dep_copy,
                Instr::Run(stack.install),
                copy_all(),
                Instr::Run(stack.build),
            ]);
            changed = match range(rng, 0, 4) {
                0 => *pick(rng, IGNORED_MD),
                1 => *pick(rng, IGNORED_DOCS),
                2 => *pick(rng, IGNORED_TESTS),
                3 => *pick(rng, &sources),
                _ => *pick(rng, stack.deps),
            };
        }
    }
    instrs.push(Instr::Cmd(pick(rng, stack.cmds)));

    let lines: Vec<String> = instrs.iter().map(Instr::render).collect();
    let option = |i: usize| format!("`{}` (строка {})", lines[i], i + 1);
    let first = instrs
        .iter()
        .position(|ins| ins.depends_on(changed, &ignored));
    let answer = first.map_or_else(|| NO_REBUILD.to_owned(), option);
    let mut distractors: Vec<String> = Vec::new();
    if first.is_some() {
        distractors.push(NO_REBUILD.to_owned());
    }
    let candidates: Vec<usize> = (1..instrs.len()).filter(|i| Some(*i) != first).collect();
    distractors.extend(shuffled(rng, candidates).into_iter().map(option));
    if first.is_some() && level < 3 && chance(rng, 0.5) {
        distractors.remove(0);
        distractors.push(NO_REBUILD.to_owned());
    }

    let ignore_text = if ignored.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nФайл `.dockerignore`:\n\n```text\n{}\n```",
            ignored.join("\n")
        )
    };
    let svc = *pick(rng, SERVICES);
    let scenario = match range(rng, 0, 3) {
        0 => format!(
            "Образ уже собирался, кэш слоёв на месте. Затем в репозитории изменили только файл `{changed}` и запустили `docker build` снова."
        ),
        1 => format!(
            "Образ сервиса `{svc}` уже собирали на этой машине, кэш слоёв сохранился. Разработчик поменял в репозитории только `{changed}` и снова запустил `docker build -t {svc} .`."
        ),
        2 => format!(
            "CI-агент уже собирал этот образ, и кэш слоёв на нём остался. В новом коммите изменён единственный файл — `{changed}`; сборка запускается тем же `docker build`."
        ),
        _ => format!(
            "Предыдущая сборка образа `{svc}` прошла успешно, кэш слоёв не очищали. После неё в рабочей копии репозитория изменился только `{changed}`, и `docker build` запустили повторно."
        ),
    };
    let question = *pick(
        rng,
        &[
            "Начиная с какой инструкции Docker перестанет использовать кэш?",
            "С какой инструкции сборка перестанет брать слои из кэша?",
            "Начиная с какой строки Dockerfile слои будут собираться заново, а не браться из кэша?",
        ],
    );
    Draft::choice(
        format!("{scenario}{ignore_text}\n\n{question}"),
        answer,
        distractors,
    )
    .code("dockerfile", format!("{}\n", lines.join("\n")))
}

fn fmt_cpu(m: i64) -> String {
    if m % 1000 == 0 {
        format!("\"{}\"", m / 1000)
    } else {
        format!("{m}m")
    }
}

fn fmt_mem(mi: i64) -> String {
    if mi % 1024 == 0 {
        format!("{}Gi", mi / 1024)
    } else {
        format!("{mi}Mi")
    }
}

struct NodeCap {
    cpu: i64,
    mem: i64,
    used_cpu: i64,
    used_mem: i64,
}

impl NodeCap {
    fn fits(&self, req_cpu: i64, req_mem: i64) -> i64 {
        ((self.cpu - self.used_cpu) / req_cpu)
            .min((self.mem - self.used_mem) / req_mem)
            .max(0)
    }
}

const REGISTRIES: &[&str] = &[
    "registry.local",
    "harbor.internal/apps",
    "cr.example.com/platform",
    "docker.example.org/team",
];

/// Distinct node names in one of several common naming schemes.
fn node_names(rng: &mut Rng, n: usize) -> Vec<String> {
    let ids: Vec<i64> = (1..=24).collect();
    let mut ids = pick_n(rng, &ids, n);
    ids.sort_unstable();
    let style = range(rng, 0, 3);
    let pool = *pick(rng, &["prod", "main", "general", "apps", "default"]);
    let subnet = range(rng, 0, 31);
    ids.iter()
        .map(|id| match style {
            0 => format!("node-{id}"),
            1 => format!("worker-{id:02}"),
            2 => format!("ip-10-0-{subnet}-{}", 10 + id * 7),
            _ => format!("{pool}-pool-{id:02}"),
        })
        .collect()
}

fn resource_fit(level: u8, rng: &mut Rng) -> Draft {
    let req_cpu = *pick(rng, &[100, 150, 200, 250, 300, 400, 500, 600, 750, 800]);
    let req_mem = *pick(rng, &[128, 192, 256, 320, 384, 512, 640, 768, 1024, 1536]);
    let lim_cpu = req_cpu * *pick(rng, &[2, 3, 4]);
    let lim_mem = req_mem * 2;
    let n_nodes = if level == 3 { range(rng, 2, 3) } else { 1 } as usize;
    let nodes: Vec<NodeCap> = (0..n_nodes)
        .map(|_| {
            let cpu = *pick(
                rng,
                &[1900, 2000, 3500, 3860, 3920, 4000, 7500, 7820, 7910, 8000],
            );
            let mem = *pick(
                rng,
                &[3584, 4096, 6144, 7168, 7680, 8192, 12288, 14336, 15360],
            );
            let (used_cpu, used_mem) = if level >= 2 {
                (
                    range(rng, 2, cpu / 250 - 2) * 250 / 2,
                    range(rng, 2, mem / 512 - 2) * 512 / 2,
                )
            } else {
                (0, 0)
            };
            NodeCap {
                cpu,
                mem,
                used_cpu,
                used_mem,
            }
        })
        .collect();
    let answer: i64 = nodes.iter().map(|n| n.fits(req_cpu, req_mem)).sum();

    let names = node_names(rng, n_nodes);
    let rows: Vec<Vec<String>> = nodes
        .iter()
        .zip(names)
        .map(|(n, name)| {
            let mut row = vec![name, fmt_cpu(n.cpu).replace('"', ""), fmt_mem(n.mem)];
            if level >= 2 {
                row.push(format!("{}m", n.used_cpu));
                row.push(format!("{}Mi", n.used_mem));
            }
            row
        })
        .collect();
    let table = if level >= 2 {
        md_table(
            &[
                "Узел",
                "Allocatable CPU",
                "Allocatable memory",
                "Уже запрошено CPU",
                "Уже запрошено memory",
            ],
            &rows,
        )
    } else {
        md_table(&["Узел", "Allocatable CPU", "Allocatable memory"], &rows)
    };
    let app = *pick(rng, SERVICES);
    let image = format!(
        "{}/{app}:{}.{}.{}",
        pick(rng, REGISTRIES),
        range(rng, 1, 3),
        range(rng, 0, 19),
        range(rng, 0, 9)
    );
    let yaml = format!(
        "containers:\n  - name: {app}\n    image: {image}\n    resources:\n      requests:\n        cpu: {}\n        memory: {}\n      limits:\n        cpu: {}\n        memory: {}\n",
        fmt_cpu(req_cpu),
        fmt_mem(req_mem),
        fmt_cpu(lim_cpu),
        fmt_mem(lim_mem)
    );
    let intro = *pick(
        rng,
        &[
            "Узлы кластера:",
            "Состояние узлов кластера:",
            "Данные по узлам кластера:",
        ],
    );
    let pod = match range(rng, 0, 2) {
        0 => format!(
            "Под Deployment `{app}` состоит из одного контейнера, его ресурсы — в манифесте."
        ),
        1 => format!("У единственного контейнера Deployment `{app}` такие ресурсы."),
        _ => format!(
            "Deployment `{app}` масштабируют; в поде один контейнер с ресурсами из манифеста."
        ),
    };
    let single = n_nodes == 1;
    let question = match range(rng, 0, 1) {
        0 => format!(
            "Сколько реплик максимум поместится {}?",
            if single {
                "на узел"
            } else {
                "на эти узлы суммарно"
            }
        ),
        _ => format!(
            "Какое наибольшее число реплик планировщик сможет разместить {}?",
            if single {
                "на этом узле"
            } else {
                "на этих узлах суммарно"
            }
        ),
    };
    let rest = *pick(
        rng,
        &[
            "Других ограничений (taints, affinity, лимит числа подов) нет.",
            "Taints, affinity и лимит числа подов на узле не мешают.",
        ],
    );
    Draft::integer(
        format!("{intro}\n\n{table}\n{pod} {question} {rest}"),
        answer,
    )
    .code("yaml", yaml)
}

fn ip(v: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        v >> 24,
        (v >> 16) & 255,
        (v >> 8) & 255,
        v & 255
    )
}

fn mask(prefix: u32) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

fn random_private(rng: &mut Rng) -> u32 {
    let base: u32 = match range(rng, 0, 2) {
        0 => 10 << 24,
        1 => (172 << 24) | ((16 + range(rng, 0, 15) as u32) << 16),
        _ => (192 << 24) | (168 << 16),
    };
    let span = if base >> 24 == 10 {
        1u32 << 24
    } else {
        1 << 16
    };
    base | (rng_u32(rng) % span)
}

fn rng_u32(rng: &mut Rng) -> u32 {
    range(rng, 0, i64::from(u32::MAX)) as u32
}

const SEGMENTS: &[&str] = &[
    "vlan-120",
    "dmz",
    "db-subnet",
    "k8s-nodes",
    "office-wifi",
    "mgmt",
    "backup-net",
    "voip",
    "cameras",
    "dev-lab",
    "guest-wifi",
    "storage",
];

fn cidr(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 => {
            let prefix = range(rng, 20, 30) as u32;
            let network = random_private(rng) & mask(prefix);
            let hosts = (1i64 << (32 - prefix)) - 2;
            let net = format!("{}/{prefix}", ip(network));
            let segment = *pick(rng, SEGMENTS);
            let prompt = match range(rng, 0, 4) {
                0 => format!(
                    "Сколько адресов можно назначить хостам в подсети `{net}` (без адреса сети и широковещательного)?"
                ),
                1 => format!(
                    "Сегменту `{segment}` выделили подсеть `{net}`. Сколько адресов в ней можно раздать хостам, если адрес сети и широковещательный не используются?"
                ),
                2 => format!(
                    "Для сегмента `{segment}` планируют подсеть `{net}`. Сколько узлов максимум можно в ней адресовать (адрес сети и широковещательный адрес хостам не назначаются)?"
                ),
                3 => format!(
                    "Подсеть `{net}` отдана под `{segment}`. Сколько в ней адресов, пригодных для хостов? Адрес сети и широковещательный не считаются."
                ),
                _ => format!(
                    "Сколько хостов поместится в подсеть `{net}`, выделенную под `{segment}`? Адрес сети и широковещательный адрес назначать нельзя."
                ),
            };
            Draft::integer(prompt, hosts)
        }
        2 => {
            let prefix = *pick(rng, &[19, 20, 21, 22, 23, 25, 26, 27, 28, 29]);
            let host = loop {
                let h = random_private(rng);
                let net = h & mask(prefix);
                if h != net && h != (net | !mask(prefix)) {
                    break h;
                }
            };
            let network = host & mask(prefix);
            let broadcast = network | !mask(prefix);
            let ask_broadcast = chance(rng, 0.5);
            let (answer, mut distractors) = if ask_broadcast {
                (
                    broadcast,
                    vec![
                        network,
                        (host & mask(prefix + 1)) | !mask(prefix + 1),
                        (host & mask(prefix - 1)) | !mask(prefix - 1),
                        host | 255,
                        broadcast - 1,
                        broadcast.wrapping_add(1 << (32 - prefix)),
                        broadcast.wrapping_sub(1 << (32 - prefix)),
                    ],
                )
            } else {
                (
                    network,
                    vec![
                        host & mask(prefix + 1),
                        host & mask(prefix - 1),
                        broadcast,
                        host & !255,
                        network + 1,
                        network.wrapping_add(1 << (32 - prefix)),
                        network.wrapping_sub(1 << (32 - prefix)),
                    ],
                )
            };
            distractors.retain(|d| *d != answer);
            let what = if ask_broadcast {
                "широковещательный адрес"
            } else {
                "адрес сети"
            };
            Draft::choice(
                format!(
                    "Хосту назначен адрес `{}/{prefix}`. Какой у его подсети {what}?",
                    ip(host)
                ),
                ip(answer),
                distractors.into_iter().map(ip),
            )
        }
        _ => {
            let prefix = range(rng, 25, 28) as u32;
            let network = random_private(rng) & mask(prefix);
            let broadcast = network | !mask(prefix);
            let size = broadcast - network;
            loop {
                let mut options: Vec<(u32, bool)> = vec![
                    (network, false),
                    (broadcast, false),
                    (broadcast.wrapping_add(1), false),
                    (network.wrapping_sub(1), false),
                    (network + 1 + rng_u32(rng) % (size - 1), true),
                    (network + 1 + rng_u32(rng) % (size - 1), true),
                    (broadcast + 1 + rng_u32(rng) % size, false),
                ];
                options = shuffled(rng, options);
                options.truncate(5);
                let mut ips: Vec<u32> = options.iter().map(|o| o.0).collect();
                ips.sort_unstable();
                ips.dedup();
                let hits = options.iter().filter(|o| o.1).count();
                if ips.len() < 5 || hits == 0 || hits == 5 {
                    continue;
                }
                return Draft::multi(
                    format!(
                        "Какие адреса можно назначить хостам в подсети `{}/{prefix}`? Отметьте все подходящие.",
                        ip(network)
                    ),
                    options.into_iter().map(|(a, ok)| (ip(a), ok)).collect(),
                );
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum LocKind {
    Exact,
    Prefix,
    PrefixStop,
    Regex { case_insensitive: bool },
}

#[derive(Clone, Copy, PartialEq)]
enum Pattern {
    Path(&'static str),
    Extensions(&'static [&'static str]),
    VersionedApi,
    Contains(&'static str),
}

#[derive(Clone, Copy)]
struct Location {
    kind: LocKind,
    pattern: Pattern,
}

impl Location {
    const fn new(kind: LocKind, pattern: Pattern) -> Self {
        Self { kind, pattern }
    }

    fn render(&self) -> String {
        let modifier = match self.kind {
            LocKind::Exact => "= ",
            LocKind::Prefix => "",
            LocKind::PrefixStop => "^~ ",
            LocKind::Regex {
                case_insensitive: false,
            } => "~ ",
            LocKind::Regex {
                case_insensitive: true,
            } => "~* ",
        };
        let pattern = match self.pattern {
            Pattern::Path(p) => p.to_owned(),
            Pattern::Extensions(exts) => format!("\\.({})$", exts.join("|")),
            Pattern::VersionedApi => "^/api/v[0-9]+/".to_owned(),
            Pattern::Contains(s) => s.to_owned(),
        };
        format!("{modifier}{pattern}")
    }

    fn regex_matches(&self, uri: &str, case_insensitive: bool) -> bool {
        let fold = |s: &str| {
            if case_insensitive {
                s.to_ascii_lowercase()
            } else {
                s.to_owned()
            }
        };
        let subject = fold(uri);
        match self.pattern {
            Pattern::Extensions(exts) => exts
                .iter()
                .any(|e| subject.ends_with(&format!(".{}", fold(e)))),
            Pattern::VersionedApi => {
                let Some(rest) = subject.strip_prefix(&fold("/api/v")) else {
                    return false;
                };
                let digits = rest.chars().take_while(char::is_ascii_digit).count();
                digits > 0 && rest[digits..].starts_with('/')
            }
            Pattern::Contains(s) => subject.contains(&fold(s)),
            Pattern::Path(_) => false,
        }
    }
}

/// nginx: exact match wins; otherwise the longest prefix is remembered; `^~` on it skips
/// regexes; otherwise the first matching regex in config order wins; otherwise the prefix.
fn nginx_select(locations: &[Location], uri: &str) -> Option<usize> {
    let exact = locations.iter().position(|l| {
        l.kind == LocKind::Exact && matches!(l.pattern, Pattern::Path(p) if p == uri)
    });
    if exact.is_some() {
        return exact;
    }
    let longest = locations
        .iter()
        .enumerate()
        .filter(|(_, l)| matches!(l.kind, LocKind::Prefix | LocKind::PrefixStop))
        .filter_map(|(i, l)| match l.pattern {
            Pattern::Path(p) if uri.starts_with(p) => Some((i, p.len())),
            _ => None,
        })
        .max_by_key(|(_, len)| *len)
        .map(|(i, _)| i);
    if let Some(i) = longest {
        if locations[i].kind == LocKind::PrefixStop {
            return Some(i);
        }
    }
    let regex = locations.iter().position(|l| match l.kind {
        LocKind::Regex { case_insensitive } => l.regex_matches(uri, case_insensitive),
        _ => false,
    });
    regex.or(longest)
}

const LOC_LABELS: &[&str] = &["A", "B", "C", "D", "E", "F", "G"];

fn nginx_location(level: u8, rng: &mut Rng) -> Draft {
    use LocKind::*;
    use Pattern::*;
    let images: &'static [&'static str] = pick(
        rng,
        &[
            &["png", "jpg", "gif"][..],
            &["jpg", "jpeg", "webp"][..],
            &["png", "svg", "ico"][..],
        ],
    );
    let (mut locations, uris): (Vec<Location>, Vec<String>) = match level {
        1 => (
            vec![
                Location::new(Prefix, Path("/")),
                Location::new(Prefix, Path("/api/")),
                Location::new(Prefix, Path("/api/v2/")),
                Location::new(Exact, Path("/api/health")),
                Location::new(Prefix, Path("/static/")),
            ],
            vec![
                "/api/v2/users".into(),
                "/api/v1/users".into(),
                "/api/health".into(),
                "/api/healthz".into(),
                "/static/app.js".into(),
                "/apidocs".into(),
                "/api".into(),
            ],
        ),
        2 => (
            vec![
                Location::new(Prefix, Path("/")),
                Location::new(Prefix, Path("/static/")),
                Location::new(
                    Regex {
                        case_insensitive: true,
                    },
                    Extensions(images),
                ),
                Location::new(Prefix, Path("/api/")),
                Location::new(
                    Regex {
                        case_insensitive: false,
                    },
                    VersionedApi,
                ),
            ],
            vec![
                format!("/static/img/logo.{}", images[0]),
                "/static/app.js".into(),
                format!("/avatars/u1.{}", images[1].to_uppercase()),
                "/api/v3/orders".into(),
                "/api/orders".into(),
                "/api/version".into(),
            ],
        ),
        _ => (
            vec![
                Location::new(Prefix, Path("/")),
                Location::new(PrefixStop, Path("/static/")),
                Location::new(
                    Regex {
                        case_insensitive: false,
                    },
                    Extensions(images),
                ),
                Location::new(
                    Regex {
                        case_insensitive: true,
                    },
                    Contains("/admin"),
                ),
                Location::new(Prefix, Path("/media/")),
                Location::new(Exact, Path("/media/")),
            ],
            vec![
                format!("/static/logo.{}", images[0]),
                format!("/media/photo.{}", images[1]),
                format!("/media/photo.{}", images[1].to_uppercase()),
                format!("/admin/logo.{}", images[2]),
                format!("/ADMIN/users"),
                "/media/".into(),
                "/media/docs/report.pdf".into(),
            ],
        ),
    };
    if level == 3 && chance(rng, 0.5) {
        locations.swap(2, 3);
    }
    let order: Vec<usize> = shuffled(rng, (1..locations.len()).collect());
    let mut arranged = vec![locations[0]];
    if level == 3 {
        arranged = locations.clone();
    } else {
        arranged.extend(order.iter().map(|&i| locations[i]));
    }
    let labels = shuffled(rng, LOC_LABELS[..arranged.len()].to_vec());
    let uri = pick(rng, &uris).clone();
    let Some(chosen) = nginx_select(&arranged, &uri) else {
        return Draft::choice(
            "Какой location обработает запрос `/`?",
            "location /",
            ["location = /", "location ^~ /", "location ~ /"].map(str::to_owned),
        );
    };
    let config: String = arranged
        .iter()
        .zip(&labels)
        .map(|(l, label)| {
            format!(
                "    location {} {{\n        return 200 \"{label}\";\n    }}\n",
                l.render()
            )
        })
        .collect();
    let code = format!("server {{\n    listen 80;\n\n{config}}}\n");
    let answer = labels[chosen].to_owned();
    let distractors = shuffled(
        rng,
        labels
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != chosen)
            .map(|(_, l)| (*l).to_owned())
            .collect(),
    );
    Draft::choice(
        format!("Какой ответ вернёт nginx на запрос `GET {uri}`?"),
        answer,
        distractors,
    )
    .code("nginx", code)
}

/// Days since 1970-01-01 → (year, month, day); Howard Hinnant's civil calendar algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn weekday(days: i64) -> u32 {
    (days + 4).rem_euclid(7) as u32
}

const WEEKDAYS: [&str; 7] = ["вс", "пн", "вт", "ср", "чт", "пт", "сб"];

fn fmt_moment(minute: i64) -> String {
    let days = minute.div_euclid(1440);
    let m = minute.rem_euclid(1440);
    let (y, mo, d) = civil_from_days(days);
    format!(
        "{}, {d:02}.{mo:02}.{y} {:02}:{:02}",
        WEEKDAYS[weekday(days) as usize],
        m / 60,
        m % 60
    )
}

#[derive(Clone)]
struct CronField {
    text: String,
    allowed: u64,
}

fn field_any(lo: u32, hi: u32) -> CronField {
    CronField {
        text: "*".into(),
        allowed: (lo..=hi).fold(0, |m, v| m | (1 << v)),
    }
}

fn field_values(values: &[u32]) -> CronField {
    CronField {
        text: join(values, ","),
        allowed: values.iter().fold(0, |m, v| m | (1 << v)),
    }
}

fn field_range(a: u32, b: u32) -> CronField {
    CronField {
        text: format!("{a}-{b}"),
        allowed: (a..=b).fold(0, |m, v| m | (1 << v)),
    }
}

fn field_step(lo: u32, hi: u32, step: u32) -> CronField {
    CronField {
        text: format!("*/{step}"),
        allowed: (lo..=hi)
            .step_by(step as usize)
            .fold(0, |m, v| m | (1 << v)),
    }
}

struct Cron {
    minute: CronField,
    hour: CronField,
    dom: CronField,
    month: CronField,
    dow: CronField,
}

impl Cron {
    fn render(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.minute.text, self.hour.text, self.dom.text, self.month.text, self.dow.text
        )
    }

    fn day_matches(&self, days: i64, dow_shift: u32) -> bool {
        let (_, m, d) = civil_from_days(days);
        let wd = (weekday(days) + dow_shift) % 7;
        self.month.allowed & (1 << m) != 0
            && self.dom.allowed & (1 << d) != 0
            && self.dow.allowed & (1 << wd) != 0
    }

    /// First matching minute strictly after `now` (minutes since epoch).
    fn next_after(&self, now: i64, dow_shift: u32, ignore_day: bool) -> Option<i64> {
        let start_day = now.div_euclid(1440);
        for day in start_day..start_day + 400 {
            if !ignore_day && !self.day_matches(day, dow_shift) {
                continue;
            }
            for h in 0..24u32 {
                if self.hour.allowed & (1 << h) == 0 {
                    continue;
                }
                for m in 0..60u32 {
                    if self.minute.allowed & (1 << m) == 0 {
                        continue;
                    }
                    let t = day * 1440 + i64::from(h * 60 + m);
                    if t > now {
                        return Some(t);
                    }
                }
            }
        }
        None
    }
}

fn cron_next(level: u8, rng: &mut Rng) -> Draft {
    for _ in 0..64 {
        let day = 20_454 + range(rng, 0, 364);
        let now = day * 1440 + range(rng, 0, 1439);
        let any_day = || (field_any(1, 31), field_any(1, 12), field_any(0, 6));
        let (minute, hour, (dom, month, dow)) = match level {
            1 => match range(rng, 0, 2) {
                0 => (
                    field_step(0, 59, *pick(rng, &[5, 10, 15, 20, 30])),
                    field_any(0, 23),
                    any_day(),
                ),
                1 => (
                    field_values(&[range(rng, 0, 59) as u32]),
                    field_any(0, 23),
                    any_day(),
                ),
                _ => (
                    field_values(&[range(rng, 0, 11) as u32 * 5]),
                    field_values(&[range(rng, 0, 23) as u32]),
                    any_day(),
                ),
            },
            2 => {
                let m = field_values(&[range(rng, 0, 11) as u32 * 5]);
                match range(rng, 0, 2) {
                    0 => {
                        let a = range(rng, 6, 10) as u32;
                        (m, field_range(a, a + range(rng, 4, 9) as u32), any_day())
                    }
                    1 => (m, field_step(0, 23, *pick(rng, &[2, 3, 4, 6])), any_day()),
                    _ => {
                        let mut ms = pick_n(rng, &[0u32, 10, 15, 20, 30, 40, 45, 50], 2);
                        ms.sort_unstable();
                        let h = range(rng, 0, 23) as u32;
                        (
                            field_values(&ms),
                            field_values(&[h, (h + range(rng, 2, 8) as u32) % 24]),
                            any_day(),
                        )
                    }
                }
            }
            _ => {
                let m = field_values(&[range(rng, 0, 11) as u32 * 5]);
                let h = field_values(&[range(rng, 6, 22) as u32]);
                match range(rng, 0, 2) {
                    0 => (
                        m,
                        h,
                        (field_any(1, 31), field_any(1, 12), field_range(1, 5)),
                    ),
                    1 => (
                        m,
                        h,
                        (field_any(1, 31), field_any(1, 12), field_values(&[0, 6])),
                    ),
                    _ => {
                        let dom = if chance(rng, 0.5) {
                            field_values(&[1])
                        } else {
                            field_step(1, 31, 10)
                        };
                        (m, h, (dom, field_any(1, 12), field_any(0, 6)))
                    }
                }
            }
        };
        let cron = Cron {
            minute,
            hour,
            dom,
            month,
            dow,
        };
        let Some(answer) = cron.next_after(now, 0, false) else {
            continue;
        };
        if answer == now + 1 || cron.next_after(now - 1, 0, false) == Some(now) {
            continue;
        }
        if level == 3 && answer - now < 1440 && chance(rng, 0.8) {
            continue;
        }
        let mut candidates: Vec<i64> = Vec::new();
        if let Some(second) = cron.next_after(answer, 0, false) {
            candidates.push(second);
        }
        if let Some(shifted) = cron.next_after(now, 1, false) {
            candidates.push(shifted);
        }
        if let Some(shifted) = cron.next_after(now, 6, false) {
            candidates.push(shifted);
        }
        if let Some(any) = cron.next_after(now, 0, true) {
            candidates.push(any);
        }
        if cron.minute.text.starts_with("*/") {
            let step: i64 = cron.minute.text[2..].parse().unwrap_or(15);
            candidates.push(now + step);
        }
        if cron.hour.text == "*" {
            if let Ok(m) = cron.minute.text.parse::<i64>() {
                candidates.push(now + m.max(1));
            }
        }
        candidates.extend([
            answer + 60,
            answer - 60,
            answer + 1440,
            answer - 1440,
            answer + 30,
            answer - 15,
            answer + 15,
        ]);
        let distractors: Vec<String> = candidates
            .into_iter()
            .filter(|c| *c != answer && *c > now)
            .map(fmt_moment)
            .collect();
        return Draft::choice(
            format!(
                "Сейчас {} (часовой пояс сервера). Когда в следующий раз запустится задача из crontab?",
                fmt_moment(now)
            ),
            fmt_moment(answer),
            distractors,
        )
        .code("bash", format!("{} /usr/local/bin/backup.sh\n", cron.render()));
    }
    Draft::choice(
        "Сейчас пн, 02.03.2026 10:17. Когда в следующий раз запустится задача `*/15 * * * *`?",
        "пн, 02.03.2026 10:30",
        [
            "пн, 02.03.2026 10:32",
            "пн, 02.03.2026 10:15",
            "пн, 02.03.2026 10:45",
        ]
        .map(str::to_owned),
    )
}

fn rwx(bits: u32) -> String {
    let mut s = String::with_capacity(9);
    for shift in [6, 3, 0] {
        let b = (bits >> shift) & 7;
        s.push(if b & 4 != 0 { 'r' } else { '-' });
        s.push(if b & 2 != 0 { 'w' } else { '-' });
        s.push(if b & 1 != 0 { 'x' } else { '-' });
    }
    s
}

fn with_special(mode: u32, dir: bool) -> String {
    let mut chars: Vec<char> = format!("{}{}", if dir { 'd' } else { '-' }, rwx(mode & 0o777))
        .chars()
        .collect();
    let mut set = |pos: usize, flag: u32, lower: char, upper: char| {
        if mode & flag != 0 {
            chars[pos] = if chars[pos] == 'x' { lower } else { upper };
        }
    };
    set(3, 0o4000, 's', 'S');
    set(6, 0o2000, 's', 'S');
    set(9, 0o1000, 't', 'T');
    chars.into_iter().collect()
}

fn octal(mode: u32) -> String {
    format!("{mode:03o}")
}

const UNIX_GROUPS: &[&str] = &[
    "devops",
    "www-data",
    "staff",
    "backup",
    "deploy",
    "adm",
    "developers",
    "ops",
];

const TEAMS: &[&str] = &[
    "devops",
    "analytics",
    "backend",
    "qa",
    "design",
    "billing",
    "support",
    "ml",
];

/// A plausible regular-file path: scripts, configs, data or logs in a fitting directory.
fn perm_path(rng: &mut Rng) -> String {
    let (dirs, names): (&[&str], &[&str]) = match range(rng, 0, 3) {
        0 => (
            &[
                "",
                "scripts/",
                "/opt/{svc}/bin/",
                "/usr/local/bin/",
                "/home/{user}/bin/",
            ],
            &[
                "deploy.sh",
                "backup.sh",
                "rotate-logs.sh",
                "migrate.sh",
                "healthcheck.sh",
                "entrypoint.sh",
                "sync.py",
                "cleanup.py",
            ],
        ),
        1 => (
            &[
                "",
                "config/",
                "/etc/{svc}/",
                "/opt/{svc}/config/",
                "/srv/{svc}/",
            ],
            &[
                "config.yaml",
                "settings.ini",
                "app.toml",
                "secrets.json",
                "database.yml",
                ".env",
            ],
        ),
        2 => (
            &["", "/var/backups/", "/srv/data/", "/home/{user}/"],
            &[
                "backup.tar",
                "dump.sql",
                "report.csv",
                "archive.tar.gz",
                "export.json",
            ],
        ),
        _ => (
            &["", "logs/", "/var/log/{svc}/", "/var/log/app/"],
            &[
                "app.log",
                "access.log",
                "error.log",
                "audit.log",
                "worker.log",
            ],
        ),
    };
    let dir = pick(rng, dirs)
        .replace("{svc}", pick(rng, SERVICES))
        .replace("{user}", pick(rng, PEOPLE));
    format!("{dir}{}", pick(rng, names))
}

fn permissions(level: u8, rng: &mut Rng) -> Draft {
    let pick_mode = |rng: &mut Rng| {
        let digits = [7u32, 6, 5, 4, 0, 1, 2, 3];
        let u = *pick(rng, &digits[..4]);
        let g = *pick(rng, &digits[..6]);
        let o = *pick(rng, &digits[..6]);
        (u << 6) | (g << 3) | o
    };
    match level {
        1 => {
            let mode = pick_mode(rng);
            let path = perm_path(rng);
            let file = if chance(rng, 0.5) {
                format!(
                    "`{path}` (владелец `{}`, группа `{}`)",
                    pick(rng, PEOPLE),
                    pick(rng, UNIX_GROUPS)
                )
            } else {
                format!("`{path}`")
            };
            let perms = rwx(mode);
            let prompt = match range(rng, 0, 2) {
                0 => format!(
                    "`ls -l` показывает для файла {file} права `-{perms}`. Какое восьмеричное значение нужно передать `chmod`, чтобы получить такие права?"
                ),
                1 => format!(
                    "В выводе `ls -l` у файла {file} права `-{perms}`. Какой восьмеричный режим нужно указать в `chmod`, чтобы выставить ровно такие права?"
                ),
                _ => format!(
                    "Файлу {file} нужно выставить права `-{perms}`. Какое восьмеричное значение для этого передать `chmod`?"
                ),
            };
            Draft::text(prompt, vec![octal(mode), format!("0{}", octal(mode))])
                .hint("Три восьмеричные цифры, например 640")
        }
        2 => {
            let start = pick_mode(rng);
            let who_options = ["u", "g", "o", "a", "go", "ug"];
            let perm_options = ["r", "w", "x", "rw", "rx", "wx"];
            let n_ops = range(rng, 2, 3) as usize;
            let mut mode = start;
            let mut ops = Vec::new();
            for _ in 0..n_ops {
                let who = *pick(rng, &who_options);
                let op = *pick(rng, &['+', '-', '+']);
                let perms = *pick(rng, &perm_options);
                let bits = perms.chars().fold(0u32, |b, c| {
                    b | match c {
                        'r' => 4,
                        'w' => 2,
                        _ => 1,
                    }
                });
                let mut m = 0u32;
                for w in who.chars() {
                    let shifts: &[u32] = match w {
                        'u' => &[6],
                        'g' => &[3],
                        'o' => &[0],
                        _ => &[6, 3, 0],
                    };
                    for s in shifts {
                        m |= bits << s;
                    }
                }
                mode = if op == '+' { mode | m } else { mode & !m };
                ops.push(format!("{who}{op}{perms}"));
            }
            Draft::text(
                format!(
                    "У файла права `{}`. Выполнили `chmod {} file`. Какими станут права в восьмеричной записи?",
                    octal(start),
                    ops.join(",")
                ),
                vec![octal(mode), format!("0{}", octal(mode))],
            )
            .hint("Три восьмеричные цифры, например 640")
        }
        _ => {
            if chance(rng, 0.5) {
                let umask = *pick(
                    rng,
                    &[
                        0o022u32, 0o027, 0o077, 0o002, 0o007, 0o026, 0o037, 0o033, 0o023,
                    ],
                );
                let file = 0o666 & !umask;
                let dir = 0o777 & !umask;
                let fmt = |f: u32, d: u32| format!("файл {}, каталог {}", octal(f), octal(d));
                let distractors = vec![
                    fmt(dir, dir),
                    fmt(file, file),
                    fmt(dir, file),
                    fmt(0o666, 0o777),
                    fmt(0o777 & !umask & 0o666, 0o755),
                    fmt(0o644, 0o755),
                ];
                let new_file = *pick(
                    rng,
                    &[
                        "report.txt",
                        "notes.md",
                        "data.csv",
                        "app.log",
                        "draft.txt",
                        "output.json",
                        "todo.txt",
                        "users.csv",
                        "summary.md",
                        "build.log",
                        "metrics.json",
                        "plan.txt",
                    ],
                );
                let new_dir = *pick(
                    rng,
                    &[
                        "backup",
                        "logs",
                        "tmp",
                        "releases",
                        "cache",
                        "uploads",
                        "archive",
                        "reports",
                        "exports",
                        "artifacts",
                        "snapshots",
                        "drafts",
                    ],
                );
                let prompt = match range(rng, 0, 3) {
                    0 => format!(
                        "В shell выполнили `umask {umask:03o}`, затем `touch {new_file}` и `mkdir {new_dir}`. Какие права получат новые файл и каталог?"
                    ),
                    1 => format!(
                        "Пользователь `{}` выполнил в своей сессии `umask {umask:03o}`, а затем создал файл (`touch {new_file}`) и каталог (`mkdir {new_dir}`). Какими будут их права?",
                        pick(rng, PEOPLE)
                    ),
                    2 => format!(
                        "Скрипт начинается с `umask {umask:03o}` и затем выполняет `mkdir {new_dir}` и `touch {new_dir}/{new_file}`. Какие права получат созданные файл и каталог?"
                    ),
                    _ => format!(
                        "В сессии пользователя `{}` действует `umask {umask:03o}`. Какие права получат файл и каталог, созданные командами `touch {new_file}` и `mkdir {new_dir}`?",
                        pick(rng, PEOPLE)
                    ),
                };
                Draft::choice(prompt, fmt(file, dir), distractors)
            } else {
                let dir = chance(rng, 0.3);
                let special = if dir {
                    *pick(rng, &[0o1000u32, 0o2000])
                } else {
                    *pick(rng, &[0o4000u32, 0o2000])
                };
                let base = if dir {
                    *pick(rng, &[0o777u32, 0o775, 0o770, 0o776, 0o755, 0o750, 0o771])
                } else {
                    *pick(
                        rng,
                        &[
                            0o755u32, 0o750, 0o644, 0o711, 0o754, 0o700, 0o740, 0o640, 0o751,
                        ],
                    )
                };
                let mode = special | base;
                let answer = with_special(mode, dir);
                let distractors = vec![
                    with_special(base, dir),
                    with_special(0o4000 | base, dir),
                    with_special(0o2000 | base, dir),
                    with_special(0o1000 | base, dir),
                    with_special(mode ^ 0o111, dir),
                ];
                let team = *pick(rng, TEAMS);
                let target = if dir {
                    let path = match range(rng, 0, 5) {
                        0 => "/srv/shared".to_owned(),
                        1 => format!("/srv/{team}"),
                        2 => format!("/var/shared/{team}"),
                        3 => format!("/home/projects/{team}"),
                        4 => "/data/exchange".to_owned(),
                        _ => format!("/opt/{team}/drop"),
                    };
                    format!("каталога `{path}`")
                } else {
                    let tool = *pick(
                        rng,
                        &[
                            "tool",
                            "backup-agent",
                            "netcheck",
                            "pinger",
                            "mount-helper",
                            "sync-agent",
                            "report-gen",
                            "db-dump",
                        ],
                    );
                    let path = match range(rng, 0, 2) {
                        0 => format!("/usr/local/bin/{tool}"),
                        1 => format!("/usr/local/sbin/{tool}"),
                        _ => format!("/opt/{team}/bin/{tool}"),
                    };
                    format!("файла `{path}`")
                };
                let prompt = match range(rng, 0, 2) {
                    0 => format!(
                        "Выполнили `chmod {mode:04o}` для {target}. Что покажет `ls -l` в колонке прав?"
                    ),
                    1 => format!(
                        "Для {target} выполнили `sudo chmod {mode:04o}`. Как будет выглядеть строка прав в выводе `ls -l`?"
                    ),
                    _ => format!(
                        "Администратор задал режим `{mode:04o}` для {target} командой `chmod`. Что будет в колонке прав у `ls -l`?"
                    ),
                };
                Draft::choice(prompt, answer, distractors)
            }
        }
    }
}

fn counter_increase(level: u8, rng: &mut Rng) -> Draft {
    let interval = *pick(rng, &[15i64, 30, 60]);
    let n = range(rng, 5, 7) as usize;
    let mut values = vec![range(rng, 100, 5000)];
    let reset_at = if level >= 2 {
        range(rng, 2, n as i64 - 2) as usize
    } else {
        usize::MAX
    };
    for i in 1..n {
        let v = if i == reset_at {
            range(rng, 0, 40)
        } else {
            values[i - 1] + range(rng, 5, 120)
        };
        values.push(v);
    }
    let increase: i64 = values
        .windows(2)
        .map(|w| if w[1] >= w[0] { w[1] - w[0] } else { w[1] })
        .sum();
    let rows: Vec<Vec<String>> = values
        .iter()
        .enumerate()
        .map(|(i, v)| vec![format!("t0 + {} с", i as i64 * interval), v.to_string()])
        .collect();
    let table = md_table(&["Время", "http_requests_total"], &rows);
    let reset_note = if level >= 2 {
        " Сервис мог перезапускаться: после рестарта counter начинается с нуля."
    } else {
        ""
    };
    if level < 3 {
        Draft::integer(
            format!(
                "Значения counter `http_requests_total` одного экземпляра сервиса:\n\n{table}\nСколько запросов обработано между первым и последним замером?{reset_note}"
            ),
            increase,
        )
    } else {
        let window = (n as i64 - 1) * interval;
        let rate = increase as f64 / window as f64;
        Draft::number(
            format!(
                "Значения counter `http_requests_total` одного экземпляра сервиса:\n\n{table}\n\
                 Какова средняя скорость в запросах в секунду между первым и последним замером (прирост с учётом сбросов, делённый на длительность, без экстраполяции)?{reset_note}"
            ),
            (rate * 100.0).round() / 100.0,
            0.011,
        )
        .hint("Число, округлите до сотых")
    }
}

const SLO_TARGETS: &[f64] = &[
    98.0, 98.5, 99.0, 99.2, 99.25, 99.5, 99.6, 99.7, 99.75, 99.8, 99.85, 99.9, 99.95, 99.99,
];
const COMPONENT_AVAILABILITY: &[f64] = &[99.0, 99.2, 99.5, 99.7, 99.8, 99.9, 99.95, 99.98, 99.99];
const REPLICA_AVAILABILITY: &[f64] = &[95.0, 96.0, 97.0, 98.0, 98.5, 99.0, 99.5];

/// Request path tiers, so a named chain always reads edge → application → storage.
const EDGE_TIER: &[&str] = &[
    "CDN",
    "балансировщик",
    "API-шлюз",
    "Ingress-контроллер",
    "обратный прокси",
];
const APP_TIER: &[&str] = &[
    "бэкенд",
    "сервис авторизации",
    "сервер приложений",
    "приложение",
];
const STORAGE_TIER: &[&str] = &[
    "база данных",
    "кэш",
    "брокер сообщений",
    "объектное хранилище",
    "поисковый кластер",
];

/// Front components named in the accusative after «через».
const BALANCERS: &[&str] = &[
    "балансировщик",
    "Ingress-контроллер",
    "API-шлюз",
    "обратный прокси",
    "L7-балансировщик",
];

/// Wording of the redundant pair with gender agreement.
struct Replicas {
    into: &'static str,
    each: &'static str,
    enough: &'static str,
}

const REPLICAS: &[Replicas] = &[
    Replicas {
        into: "в один из двух независимых экземпляров приложения",
        each: "каждый",
        enough: "работал хотя бы один экземпляр",
    },
    Replicas {
        into: "в одну из двух независимых реплик бэкенда",
        each: "каждая",
        enough: "работала хотя бы одна реплика",
    },
    Replicas {
        into: "на один из двух независимых серверов приложения",
        each: "каждый",
        enough: "работал хотя бы один сервер",
    },
    Replicas {
        into: "в один из двух независимых подов сервиса",
        each: "каждый",
        enough: "работал хотя бы один под",
    },
    Replicas {
        into: "на один из двух независимых узлов с приложением",
        each: "каждый",
        enough: "работал хотя бы один узел",
    },
];

fn error_budget(level: u8, rng: &mut Rng) -> Draft {
    let svc = *pick(rng, SERVICES);
    match level {
        1 | 2 => {
            let days = range(rng, 7, 90);
            let minutes = (days * 24 * 60) as f64;
            let budget_of = |slo: f64| (1.0 - slo / 100.0) * minutes;
            // Level 2 lists incidents rounded to tenths of a minute: keep them meaningful.
            let targets: Vec<f64> = SLO_TARGETS
                .iter()
                .copied()
                .filter(|s| level == 1 || budget_of(*s) >= 10.0)
                .collect();
            let slo = *pick(rng, &targets);
            let budget = budget_of(slo);
            let period = plural_ru(days, "день", "дня", "дней");
            let slo = fmt_number(slo);
            let intro = match range(rng, 0, 3) {
                0 => {
                    format!("SLO доступности сервиса `{svc}` — {slo}% в скользящем окне {period}.")
                }
                1 => format!(
                    "Для сервиса `{svc}` установлен SLO доступности {slo}% на скользящем окне {period}."
                ),
                2 => format!(
                    "Команда сервиса `{svc}` договорилась о SLO: доступность {slo}% в окне {period}."
                ),
                _ => format!(
                    "Целевая доступность (SLO) сервиса `{svc}` — {slo}% за скользящее окно {period}."
                ),
            };
            if level == 1 {
                let question = *pick(
                    rng,
                    &[
                        "Сколько минут полной недоступности допускает бюджет ошибок за это окно?",
                        "Сколько минут сервис может быть полностью недоступен за это окно, не нарушив SLO?",
                        "Чему равен бюджет ошибок за окно в минутах полной недоступности?",
                        "Каков бюджет ошибок на это окно, если выразить его в минутах полного простоя?",
                    ],
                );
                Draft::number(
                    format!("{intro} {question}"),
                    (budget * 10.0).round() / 10.0,
                    0.11,
                )
                .hint("Число минут, округлите до десятых")
            } else {
                let incidents: Vec<f64> = (0..range(rng, 2, 3))
                    .map(|_| {
                        (budget * f64::from(range(rng, 5, 25) as u32) / 100.0 * 10.0).round() / 10.0
                    })
                    .collect();
                let spent: f64 = incidents.iter().sum();
                let left = budget - spent;
                let list = incidents
                    .iter()
                    .map(|m| format!("{} мин", fmt_number(*m)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let history = match range(rng, 0, 2) {
                    0 => format!("В этом окне уже были инциденты с полной недоступностью: {list}."),
                    1 => format!("За это окно уже случились полные отказы длительностью {list}."),
                    _ => format!(
                        "В текущем окне зафиксированы инциденты полной недоступности: {list}."
                    ),
                };
                let question = *pick(
                    rng,
                    &[
                        "Сколько минут бюджета ошибок осталось?",
                        "Сколько минут бюджета ошибок ещё не израсходовано?",
                        "Каков остаток бюджета ошибок в минутах?",
                    ],
                );
                Draft::number(
                    format!("{intro} {history} {question}"),
                    (left * 10.0).round() / 10.0,
                    0.11,
                )
                .hint("Число минут, округлите до десятых")
            }
        }
        _ => {
            let (availability, text) = if chance(rng, 0.5) {
                let front = *pick(rng, COMPONENT_AVAILABILITY);
                let replica = *pick(rng, REPLICA_AVAILABILITY);
                let pair = 1.0 - (1.0 - replica / 100.0).powi(2);
                let r = pick(rng, REPLICAS);
                (
                    pair * front / 100.0,
                    format!(
                        "Запрос к сервису `{svc}` проходит через {} (доступность {}%) и затем попадает {} ({} — {}%); достаточно, чтобы {}.",
                        pick(rng, BALANCERS),
                        fmt_number(front),
                        r.into,
                        r.each,
                        fmt_number(replica),
                        r.enough
                    ),
                )
            } else {
                let parts: Vec<f64> = (0..3).map(|_| *pick(rng, COMPONENT_AVAILABILITY)).collect();
                let availability = parts.iter().map(|p| p / 100.0).product::<f64>();
                let [a, b, c] = [parts[0], parts[1], parts[2]].map(fmt_number);
                let text = if chance(rng, 0.5) {
                    format!(
                        "Запрос к сервису `{svc}` последовательно проходит через три независимых компонента с доступностью {a}%, {b}% и {c}%; отказ любого — отказ запроса."
                    )
                } else {
                    format!(
                        "Цепочка обработки запроса к сервису `{svc}`: {} ({a}%) → {} ({b}%) → {} ({c}%). Компоненты отказывают независимо, отказ любого — отказ запроса.",
                        pick(rng, EDGE_TIER),
                        pick(rng, APP_TIER),
                        pick(rng, STORAGE_TIER)
                    )
                };
                (availability, text)
            };
            let question = *pick(
                rng,
                &[
                    "Какова итоговая доступность сервиса в процентах?",
                    "Какую доступность в процентах обеспечивает такая схема в целом?",
                    "Оцените итоговую доступность сервиса в процентах.",
                ],
            );
            Draft::number(
                format!("{text} {question}"),
                (availability * 1e4).round() / 100.0,
                0.011,
            )
            .hint("Проценты, округлите до сотых")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(weekday(0), 4);
        assert_eq!(civil_from_days(20_514), (2026, 3, 2));
        assert_eq!(weekday(20_514), 1);
        assert_eq!(fmt_moment(20_514 * 1440 + 617), "пн, 02.03.2026 10:17");
    }

    #[test]
    fn cron_next_run() {
        let now = 20_514 * 1440 + 617;
        let every15 = Cron {
            minute: field_step(0, 59, 15),
            hour: field_any(0, 23),
            dom: field_any(1, 31),
            month: field_any(1, 12),
            dow: field_any(0, 6),
        };
        assert_eq!(
            fmt_moment(every15.next_after(now, 0, false).unwrap()),
            "пн, 02.03.2026 10:30"
        );
        let weekdays = Cron {
            minute: field_values(&[30]),
            hour: field_values(&[8]),
            dom: field_any(1, 31),
            month: field_any(1, 12),
            dow: field_range(1, 5),
        };
        let friday_evening = (20_514 + 4) * 1440 + 20 * 60;
        assert_eq!(
            fmt_moment(weekdays.next_after(friday_evening, 0, false).unwrap()),
            "пн, 09.03.2026 08:30"
        );
    }

    #[test]
    fn nginx_rules() {
        use LocKind::*;
        use Pattern::*;
        let locs = [
            Location::new(Prefix, Path("/")),
            Location::new(PrefixStop, Path("/static/")),
            Location::new(
                Regex {
                    case_insensitive: true,
                },
                Extensions(&["png"]),
            ),
            Location::new(Prefix, Path("/media/")),
            Location::new(Exact, Path("/media/")),
            Location::new(
                Regex {
                    case_insensitive: false,
                },
                VersionedApi,
            ),
        ];
        assert_eq!(nginx_select(&locs, "/static/a.png"), Some(1));
        assert_eq!(nginx_select(&locs, "/media/a.PNG"), Some(2));
        assert_eq!(nginx_select(&locs, "/media/"), Some(4));
        assert_eq!(nginx_select(&locs, "/media/a.pdf"), Some(3));
        assert_eq!(nginx_select(&locs, "/api/v12/x"), Some(5));
        assert_eq!(nginx_select(&locs, "/api/vx/x"), Some(0));
    }

    #[test]
    fn docker_cache_rules() {
        let copy_all = Instr::Copy {
            sources: vec!["."],
            dest: ".",
        };
        let pkg = Instr::Copy {
            sources: vec!["package*.json"],
            dest: "./",
        };
        assert!(copy_all.depends_on("src/a.ts", &[]));
        assert!(!copy_all.depends_on("README.md", &["*.md"]));
        assert!(copy_all.depends_on("docs/README.md", &["*.md"]));
        assert!(pkg.depends_on("package-lock.json", &[]));
        assert!(!pkg.depends_on("src/package.json", &[]));
    }

    #[test]
    fn permission_rendering() {
        assert_eq!(rwx(0o750), "rwxr-x---");
        assert_eq!(with_special(0o4755, false), "-rwsr-xr-x");
        assert_eq!(with_special(0o4644, false), "-rwSr--r--");
        assert_eq!(with_special(0o1777, true), "drwxrwxrwt");
        assert_eq!(with_special(0o2750, false), "-rwxr-s---");
    }
}
