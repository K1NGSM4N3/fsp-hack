//! Backend templates: Python and Go semantics, indexes, rate limiting, caching.

use catalog::Specialization::{Backend, Devops, Ml, Qa};

use super::util::{chance, join, perturb_sequence, pick, pick_n, plural_ru, range, shuffled};
use super::{C_CHOICE, C_OPEN, Draft, Level, Rng, Source, Template, Topic};

pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        id: "py.mutable_default",
        title: "Изменяемые значения по умолчанию в Python",
        topic: Topic::Languages,
        specializations: &[Backend, Ml],
        skills: &["python"],
        levels: &[
            Level::new(1, 1.1, -0.6, C_CHOICE, 90),
            Level::new(2, 1.3, 0.2, C_CHOICE, 120),
            Level::new(3, 1.4, 1.0, C_CHOICE, 120),
        ],
        source: Source::Computed(mutable_default),
    },
    Template {
        id: "py.aliasing",
        title: "Ссылки и копии списков в Python",
        topic: Topic::Languages,
        specializations: &[Backend, Ml, Qa],
        skills: &["python"],
        levels: &[
            Level::new(1, 1.1, -0.9, C_CHOICE, 90),
            Level::new(2, 1.3, 0.1, C_CHOICE, 120),
            Level::new(3, 1.5, 0.9, C_CHOICE, 120),
        ],
        source: Source::Computed(aliasing),
    },
    Template {
        id: "go.defer_order",
        title: "Порядок defer в Go",
        topic: Topic::Languages,
        specializations: &[Backend, Devops],
        skills: &["go"],
        levels: &[
            Level::new(1, 1.0, -0.5, C_CHOICE, 90),
            Level::new(2, 1.3, 0.4, C_CHOICE, 120),
            Level::new(3, 1.2, 0.9, C_OPEN, 150),
        ],
        source: Source::Computed(go_defer),
    },
    Template {
        id: "db.index_choice",
        title: "Выбор составного индекса",
        topic: Topic::Databases,
        specializations: &[Backend],
        skills: &["postgresql", "sql"],
        levels: &[
            Level::new(1, 1.0, -0.4, C_CHOICE, 90),
            Level::new(2, 1.3, 0.4, C_CHOICE, 120),
            Level::new(3, 1.5, 1.1, C_CHOICE, 120),
        ],
        source: Source::Computed(index_choice),
    },
    Template {
        id: "arch.token_bucket",
        title: "Rate limiting: token bucket",
        topic: Topic::Architecture,
        specializations: &[Backend, Devops],
        skills: &["system_design"],
        levels: &[
            Level::new(1, 1.1, -0.3, C_OPEN, 120),
            Level::new(2, 1.3, 0.4, C_OPEN, 150),
            Level::new(3, 1.5, 1.1, C_OPEN, 180),
        ],
        source: Source::Computed(token_bucket),
    },
    Template {
        id: "arch.lru_cache",
        title: "Вытеснение из LRU-кэша",
        topic: Topic::Architecture,
        specializations: &[Backend, Devops],
        skills: &["system_design", "data_structures", "redis"],
        levels: &[
            Level::new(1, 1.0, -0.7, C_OPEN, 90),
            Level::new(2, 1.2, 0.1, C_OPEN, 120),
            Level::new(3, 1.4, 0.8, C_CHOICE, 120),
        ],
        source: Source::Computed(lru_cache),
    },
];

fn py_list(v: &[i64]) -> String {
    format!("[{}]", join(v, ", "))
}

fn distinct_values(rng: &mut Rng, n: usize, lo: i64, hi: i64) -> Vec<i64> {
    let pool: Vec<i64> = (lo..=hi).collect();
    pick_n(rng, &pool, n)
}

#[derive(Clone, Copy)]
struct DefaultModel {
    fresh_default: bool,
    snapshot: bool,
}

fn mutable_default(level: u8, rng: &mut Rng) -> Draft {
    if level == 3 {
        return class_attribute(rng);
    }
    let fname = *pick(rng, &["add_item", "collect", "register", "remember"]);
    let param = *pick(rng, &["bucket", "acc", "items", "seen"]);
    let vars = ["a", "b", "c", "d"];
    let calls = if level == 1 {
        range(rng, 2, 3)
    } else {
        range(rng, 3, 4)
    } as usize;
    let values = distinct_values(rng, calls, 1, 9);
    let explicit: Vec<bool> = (0..calls)
        .map(|i| level == 2 && i > 0 && chance(rng, 0.4))
        .collect();
    let explicit = if level == 2 && !explicit.contains(&true) {
        let mut e = explicit;
        e[calls - 1] = true;
        e
    } else {
        explicit
    };
    let printed: Vec<usize> = if level == 1 {
        vec![0]
    } else {
        let defaults: Vec<usize> = (0..calls).filter(|i| !explicit[*i]).collect();
        let mut p = vec![defaults[0]];
        if let Some(&e) = (0..calls).find(|i| explicit[*i]).as_ref() {
            p.push(e);
        }
        p.sort_unstable();
        p
    };

    let run = |m: DefaultModel| -> String {
        let mut shared: Vec<i64> = Vec::new();
        let mut lists: Vec<Vec<i64>> = Vec::new();
        let mut owner: Vec<Option<usize>> = Vec::new();
        for i in 0..calls {
            if explicit[i] || m.fresh_default {
                lists.push(vec![values[i]]);
                owner.push(None);
            } else {
                shared.push(values[i]);
                lists.push(shared.clone());
                owner.push(Some(i));
            }
        }
        printed
            .iter()
            .map(|&i| match owner[i] {
                Some(_) if !m.snapshot => py_list(&shared),
                _ => py_list(&lists[i]),
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let answer = run(DefaultModel {
        fresh_default: false,
        snapshot: false,
    });
    let mut distractors = vec![
        run(DefaultModel {
            fresh_default: true,
            snapshot: false,
        }),
        run(DefaultModel {
            fresh_default: false,
            snapshot: true,
        }),
    ];
    let mut all = values.clone();
    distractors.push(
        printed
            .iter()
            .map(|_| py_list(&all))
            .collect::<Vec<_>>()
            .join(" "),
    );
    all.reverse();
    distractors.push(
        printed
            .iter()
            .map(|_| py_list(&all))
            .collect::<Vec<_>>()
            .join(" "),
    );
    distractors.push(
        printed
            .iter()
            .map(|&i| py_list(&values[..=i]))
            .collect::<Vec<_>>()
            .join(" "),
    );
    distractors.push(
        printed
            .iter()
            .map(|_| "[]".to_owned())
            .collect::<Vec<_>>()
            .join(" "),
    );

    let mut code =
        format!("def {fname}(item, {param}=[]):\n    {param}.append(item)\n    return {param}\n\n");
    for i in 0..calls {
        let arg = if explicit[i] { ", []" } else { "" };
        code.push_str(&format!("{} = {fname}({}{arg})\n", vars[i], values[i]));
    }
    let shown: Vec<&str> = printed.iter().map(|&i| vars[i]).collect();
    code.push_str(&format!("print({})\n", shown.join(", ")));
    Draft::choice("Что выведет программа?", answer, distractors).code("python", code)
}

#[derive(Clone, Copy, PartialEq)]
enum CartOp {
    Add(usize, i64),
    Assign(usize, i64),
}

#[derive(Clone, Copy, PartialEq)]
enum CartModel {
    Python,
    PerInstance,
    AssignWritesClass,
    AppendIgnoresShadow,
}

fn run_cart(ops: &[CartOp], model: CartModel) -> (Vec<i64>, Vec<i64>) {
    let mut class: Vec<i64> = Vec::new();
    let mut own: [Option<Vec<i64>>; 2] = [None, None];
    if model == CartModel::PerInstance {
        own = [Some(Vec::new()), Some(Vec::new())];
    }
    for op in ops {
        match *op {
            CartOp::Add(who, v) => match &mut own[who] {
                Some(list) if model != CartModel::AppendIgnoresShadow => list.push(v),
                _ => class.push(v),
            },
            CartOp::Assign(who, v) => {
                if model == CartModel::AssignWritesClass {
                    class = vec![v];
                } else {
                    own[who] = Some(vec![v]);
                }
            }
        }
    }
    let view = |who: usize| own[who].clone().unwrap_or_else(|| class.clone());
    (view(0), view(1))
}

fn class_attribute(rng: &mut Rng) -> Draft {
    let class_name = *pick(rng, &["Cart", "Basket", "Order", "Session"]);
    let attr = *pick(rng, &["items", "events", "tags", "log"]);
    let owners = pick_n(rng, super::util::PEOPLE, 2);
    let values = distinct_values(rng, 6, 1, 9);
    let mut ops = vec![CartOp::Add(0, values[0]), CartOp::Add(1, values[1])];
    let assigned = range(rng, 0, 1) as usize;
    let extra = range(rng, 1, 3) as usize;
    let assign_at = range(rng, 1, ops.len() as i64) as usize;
    ops.insert(assign_at, CartOp::Assign(assigned, values[2]));
    for k in 0..extra {
        ops.push(CartOp::Add(range(rng, 0, 1) as usize, values[3 + k]));
    }

    let show = |(a, b): (Vec<i64>, Vec<i64>)| format!("{} {}", py_list(&a), py_list(&b));
    let answer = show(run_cart(&ops, CartModel::Python));
    let mut distractors: Vec<String> = [
        CartModel::PerInstance,
        CartModel::AssignWritesClass,
        CartModel::AppendIgnoresShadow,
    ]
    .iter()
    .map(|m| show(run_cart(&ops, *m)))
    .collect();
    let (a, b) = run_cart(&ops, CartModel::Python);
    distractors.push(show((b.clone(), a.clone())));
    let mut merged = a.clone();
    merged.extend(b.iter().filter(|v| !a.contains(v)));
    distractors.push(show((merged.clone(), merged)));

    let names = ["a", "b"];
    let mut code = format!(
        "class {class_name}:\n    {attr} = []\n\n    def __init__(self, owner):\n        self.owner = owner\n\n    def add(self, x):\n        self.{attr}.append(x)\n\n\n"
    );
    code.push_str(&format!(
        "a = {class_name}(\"{}\")\nb = {class_name}(\"{}\")\n",
        owners[0], owners[1]
    ));
    for op in &ops {
        match op {
            CartOp::Add(who, v) => code.push_str(&format!("{}.add({v})\n", names[*who])),
            CartOp::Assign(who, v) => code.push_str(&format!("{}.{attr} = [{v}]\n", names[*who])),
        }
    }
    code.push_str(&format!("print(a.{attr}, b.{attr})\n"));
    Draft::choice("Что выведет программа?", answer, distractors).code("python", code)
}

#[derive(Clone, Copy, PartialEq)]
enum AliasOp {
    Append(usize, i64),
    Pop(usize),
    PlusRebind(usize, i64),
    PlusAssign(usize, i64),
}

#[derive(Clone, Copy, Default)]
struct AliasModel {
    copy_is_alias: bool,
    assign_is_copy: bool,
    plus_assign_rebinds: bool,
    plus_mutates: bool,
}

/// Variables `a`, `b`, `c`: `b = a`, `c` = copy of `a`; returns the final lengths.
fn run_alias(initial: usize, ops: &[AliasOp], m: AliasModel) -> [usize; 3] {
    let mut lists: Vec<usize> = vec![initial];
    let mut var = [0usize, 0, 0];
    if m.assign_is_copy {
        lists.push(initial);
        var[1] = lists.len() - 1;
    }
    if !m.copy_is_alias {
        lists.push(initial);
        var[2] = lists.len() - 1;
    }
    for op in ops {
        match *op {
            AliasOp::Append(v, _) => lists[var[v]] += 1,
            AliasOp::Pop(v) => lists[var[v]] = lists[var[v]].saturating_sub(1),
            AliasOp::PlusRebind(v, _) if m.plus_mutates => lists[var[v]] += 1,
            AliasOp::PlusAssign(v, _) if !m.plus_assign_rebinds => lists[var[v]] += 1,
            AliasOp::PlusRebind(v, _) | AliasOp::PlusAssign(v, _) => {
                lists.push(lists[var[v]] + 1);
                var[v] = lists.len() - 1;
            }
        }
    }
    [lists[var[0]], lists[var[1]], lists[var[2]]]
}

fn aliasing(level: u8, rng: &mut Rng) -> Draft {
    if level == 3 {
        return if chance(rng, 0.5) {
            nested_grid(rng)
        } else {
            shallow_copy(rng)
        };
    }
    let names = ["a", "b", "c"];
    let initial = range(rng, 2, 4) as usize;
    let init_values = distinct_values(rng, initial, 1, 9);
    let copy_expr = *pick(rng, &["a[:]", "a.copy()", "list(a)"]);
    let n_ops = range(rng, 3, 4) as usize;
    let mut ops = Vec::new();
    for i in 0..n_ops {
        let v = range(rng, 0, 2) as usize;
        let x = range(rng, 10, 99);
        let can_pop = run_alias(initial, &ops, AliasModel::default())[v] > 0;
        let op = if level == 2 && (i == 0 || chance(rng, 0.5)) {
            if chance(rng, 0.5) {
                AliasOp::PlusRebind(v.max(1), x)
            } else {
                AliasOp::PlusAssign(v, x)
            }
        } else if can_pop && chance(rng, 0.2) {
            AliasOp::Pop(v)
        } else {
            AliasOp::Append(v, x)
        };
        ops.push(op);
    }
    if level == 2 && !ops.iter().any(|o| matches!(o, AliasOp::PlusRebind(..))) {
        ops[n_ops - 1] = AliasOp::PlusRebind(1, range(rng, 10, 99));
    }
    let render = |l: [usize; 3]| format!("{} {} {}", l[0], l[1], l[2]);
    let answer = run_alias(initial, &ops, AliasModel::default());
    let models = [
        AliasModel {
            copy_is_alias: true,
            ..AliasModel::default()
        },
        AliasModel {
            assign_is_copy: true,
            ..AliasModel::default()
        },
        AliasModel {
            plus_assign_rebinds: true,
            ..AliasModel::default()
        },
        AliasModel {
            plus_mutates: true,
            ..AliasModel::default()
        },
        AliasModel {
            copy_is_alias: true,
            assign_is_copy: true,
            ..AliasModel::default()
        },
    ];
    let mut distractors: Vec<String> = models
        .iter()
        .map(|m| render(run_alias(initial, &ops, *m)))
        .collect();
    let [x, y, z] = answer;
    for variant in [
        [x, x, x],
        [z, y, x],
        [x, y, x],
        [x + 1, y + 1, z],
        [x, y, z + 1],
        [x + 1, y, z],
        [x, y + 1, z + 1],
    ] {
        distractors.push(render(variant));
    }

    let mut code = format!("a = {}\nb = a\nc = {copy_expr}\n", py_list(&init_values));
    for op in &ops {
        code.push_str(&match *op {
            AliasOp::Append(v, x) => format!("{}.append({x})\n", names[v]),
            AliasOp::Pop(v) => format!("{}.pop()\n", names[v]),
            AliasOp::PlusRebind(v, x) => format!("{0} = {0} + [{x}]\n", names[v]),
            AliasOp::PlusAssign(v, x) => format!("{} += [{x}]\n", names[v]),
        });
    }
    code.push_str("print(len(a), len(b), len(c))\n");
    Draft::choice("Что выведет программа?", render(answer), distractors).code("python", code)
}

fn nested_grid(rng: &mut Rng) -> Draft {
    let rows = range(rng, 2, 4) as usize;
    let cols = range(rng, 2, 4) as usize;
    let shared = chance(rng, 0.6);
    let writes: Vec<(usize, usize, i64)> = (0..range(rng, 2, 3))
        .map(|_| {
            (
                range(rng, 0, rows as i64 - 1) as usize,
                range(rng, 0, cols as i64 - 1) as usize,
                range(rng, 1, 9),
            )
        })
        .collect();
    let total = |shared: bool| -> i64 {
        let mut grid = vec![vec![0i64; cols]; rows];
        for &(r, c, v) in &writes {
            if shared {
                for row in &mut grid {
                    row[c] = v;
                }
            } else {
                grid[r][c] = v;
            }
        }
        grid.iter().flatten().sum()
    };
    let answer = total(shared);
    let naive: i64 = writes.iter().map(|w| w.2).sum();
    let mut distractors: Vec<i64> = vec![total(!shared), naive, naive * rows as i64];
    distractors.extend(
        [answer + 1, answer * 2, answer - 1]
            .into_iter()
            .filter(|v| *v >= 0),
    );
    let ctor = if shared {
        format!("[[0] * {cols}] * {rows}")
    } else {
        format!("[[0] * {cols} for _ in range({rows})]")
    };
    let mut code = format!("grid = {ctor}\n");
    for (r, c, v) in &writes {
        code.push_str(&format!("grid[{r}][{c}] = {v}\n"));
    }
    code.push_str("print(sum(sum(row) for row in grid))\n");
    Draft::choice(
        "Что выведет программа?",
        answer.to_string(),
        distractors
            .into_iter()
            .filter(|d| *d != answer)
            .map(|d| d.to_string()),
    )
    .code("python", code)
}

#[derive(Clone, Copy)]
enum CopyOp {
    AppendInner(usize, i64),
    AppendOuter,
    ReplaceInner(usize),
}

fn shallow_copy(rng: &mut Rng) -> Draft {
    let (n1, n2) = (range(rng, 1, 3) as usize, range(rng, 1, 2) as usize);
    let first = distinct_values(rng, n1, 1, 9);
    let second = distinct_values(rng, n2, 1, 9);
    let mut ops: Vec<CopyOp> = (0..range(rng, 2, 3))
        .map(|_| match range(rng, 0, 3) {
            0 | 1 => CopyOp::AppendInner(range(rng, 0, 1) as usize, range(rng, 10, 99)),
            2 => CopyOp::AppendOuter,
            _ => CopyOp::ReplaceInner(range(rng, 0, 1) as usize),
        })
        .collect();
    if !ops.iter().any(|o| matches!(o, CopyOp::AppendInner(..))) {
        ops.insert(0, CopyOp::AppendInner(0, range(rng, 10, 99)));
    }

    let mut m_slots = [0usize, 1];
    let mut objects = vec![first.len(), second.len()];
    let mut m_len = 2;
    for op in &ops {
        match *op {
            CopyOp::AppendInner(i, _) => objects[m_slots[i]] += 1,
            CopyOp::AppendOuter => m_len += 1,
            CopyOp::ReplaceInner(i) => {
                objects.push(1);
                m_slots[i] = objects.len() - 1;
            }
        }
    }
    let lens = [objects[0], objects[1]];
    let s_len = 2;
    let d = [first.len(), second.len()];
    let k = range(rng, 0, 1) as usize;
    let answer = [lens[k], s_len, d[k]];
    let render = |v: [usize; 3]| format!("{} {} {}", v[0], v[1], v[2]);
    let m_view = objects[m_slots[k]];
    let distractors = vec![
        render([d[k], s_len, d[k]]),
        render([lens[k], m_len, d[k]]),
        render([lens[k], s_len, lens[k]]),
        render([m_view, m_len, lens[k]]),
        render([lens[k] + 1, s_len, d[k]]),
        render([lens[k], s_len, d[k] + 1]),
        render([lens[k], s_len + 1, d[k]]),
        render([lens[k] + 1, s_len + 1, d[k]]),
        render([d[k], s_len, lens[k]]),
    ];

    let mut code = format!(
        "import copy\n\nm = [{}, {}]\ns = copy.copy(m)\nd = copy.deepcopy(m)\n",
        py_list(&first),
        py_list(&second)
    );
    for op in &ops {
        code.push_str(&match *op {
            CopyOp::AppendInner(i, v) => format!("m[{i}].append({v})\n"),
            CopyOp::AppendOuter => "m.append([0])\n".to_owned(),
            CopyOp::ReplaceInner(i) => format!("m[{i}] = [{}]\n", 70 + i),
        });
    }
    code.push_str(&format!("print(len(s[{k}]), len(s), len(d[{k}]))\n"));
    Draft::choice("Что выведет программа?", render(answer), distractors).code("python", code)
}

#[derive(Clone, Copy, PartialEq)]
enum GoStmt {
    Print(char),
    Defer(char),
}

fn go_defer(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 => go_defer_labels(rng),
        2 => go_defer_values(rng),
        _ => go_named_results(rng),
    }
}

fn go_defer_labels(rng: &mut Rng) -> Draft {
    let n = range(rng, 5, 6) as usize;
    let labels = shuffled(rng, ('A'..='F').take(n).collect::<Vec<_>>());
    let mut stmts: Vec<GoStmt> = labels
        .iter()
        .map(|&c| {
            if chance(rng, 0.5) {
                GoStmt::Defer(c)
            } else {
                GoStmt::Print(c)
            }
        })
        .collect();
    let defers = stmts
        .iter()
        .filter(|s| matches!(s, GoStmt::Defer(_)))
        .count();
    if defers < 2 {
        stmts[1] = GoStmt::Defer(labels[1]);
        stmts[3] = GoStmt::Defer(labels[3]);
    }
    if !stmts.iter().any(|s| matches!(s, GoStmt::Print(_))) {
        stmts[0] = GoStmt::Print(labels[0]);
    }
    let prints: String = stmts
        .iter()
        .filter_map(|s| match s {
            GoStmt::Print(c) => Some(*c),
            _ => None,
        })
        .collect();
    let deferred: String = stmts
        .iter()
        .filter_map(|s| match s {
            GoStmt::Defer(c) => Some(*c),
            _ => None,
        })
        .collect();
    let lifo: String = deferred.chars().rev().collect();
    let answer = format!("{prints}{lifo}");
    let source: String = labels.iter().collect();
    let mut distractors = vec![
        format!("{prints}{deferred}"),
        source.clone(),
        format!("{lifo}{prints}"),
        format!("{deferred}{prints}"),
        source.chars().rev().collect(),
    ];
    distractors.extend(perturb_sequence(&answer));
    let body: String = stmts
        .iter()
        .map(|s| match s {
            GoStmt::Print(c) => format!("\tfmt.Print(\"{c}\")\n"),
            GoStmt::Defer(c) => format!("\tdefer fmt.Print(\"{c}\")\n"),
        })
        .collect();
    let code = format!("package main\n\nimport \"fmt\"\n\nfunc main() {{\n{body}}}\n");
    Draft::choice("Что выведет программа?", answer, distractors).code("go", code)
}

#[derive(Clone, Copy)]
enum GoValueStmt {
    Assign(i64),
    DeferArg,
    DeferClosure,
    DeferLoop(i64),
    Print,
}

#[derive(Clone, Copy, Default)]
struct GoModel {
    closure_snapshots: bool,
    args_late: bool,
    fifo: bool,
}

fn run_go_values(stmts: &[GoValueStmt], init: i64, m: GoModel) -> String {
    enum Deferred {
        Value(i64),
        Variable,
    }
    let mut x = init;
    let mut out = String::new();
    let mut deferred = Vec::new();
    for s in stmts {
        match *s {
            GoValueStmt::Assign(v) => x = v,
            GoValueStmt::DeferArg => deferred.push(if m.args_late {
                Deferred::Variable
            } else {
                Deferred::Value(x)
            }),
            GoValueStmt::DeferClosure => deferred.push(if m.closure_snapshots {
                Deferred::Value(x)
            } else {
                Deferred::Variable
            }),
            GoValueStmt::DeferLoop(n) => deferred.extend((0..n).map(Deferred::Value)),
            GoValueStmt::Print => out.push_str(&x.to_string()),
        }
    }
    if !m.fifo {
        deferred.reverse();
    }
    for d in deferred {
        out.push_str(&match d {
            Deferred::Value(v) => v.to_string(),
            Deferred::Variable => x.to_string(),
        });
    }
    out
}

fn go_defer_values(rng: &mut Rng) -> Draft {
    let var = *pick(rng, &["x", "n", "count", "v"]);
    let values = distinct_values(rng, 3, 1, 9);
    let init = values[0];
    let mut stmts = vec![
        GoValueStmt::DeferArg,
        GoValueStmt::Assign(values[1]),
        GoValueStmt::DeferClosure,
        GoValueStmt::DeferLoop(range(rng, 2, 3)),
        GoValueStmt::Print,
        GoValueStmt::Assign(values[2]),
    ];
    if chance(rng, 0.5) {
        stmts.swap(2, 3);
    }
    if chance(rng, 0.5) {
        stmts.swap(3, 4);
    }
    let answer = run_go_values(&stmts, init, GoModel::default());
    let models = [
        GoModel {
            closure_snapshots: true,
            ..GoModel::default()
        },
        GoModel {
            args_late: true,
            ..GoModel::default()
        },
        GoModel {
            fifo: true,
            ..GoModel::default()
        },
        GoModel {
            closure_snapshots: true,
            args_late: true,
            ..GoModel::default()
        },
        GoModel {
            closure_snapshots: true,
            fifo: true,
            ..GoModel::default()
        },
    ];
    let mut distractors: Vec<String> = models
        .iter()
        .map(|m| run_go_values(&stmts, init, *m))
        .collect();
    distractors.extend(perturb_sequence(&answer));

    let mut body = format!("\t{var} := {init}\n");
    for s in &stmts {
        body.push_str(&match *s {
            GoValueStmt::Assign(v) => format!("\t{var} = {v}\n"),
            GoValueStmt::DeferArg => format!("\tdefer fmt.Print({var})\n"),
            GoValueStmt::DeferClosure => {
                format!("\tdefer func() {{\n\t\tfmt.Print({var})\n\t}}()\n")
            }
            GoValueStmt::DeferLoop(n) => {
                format!("\tfor i := 0; i < {n}; i++ {{\n\t\tdefer fmt.Print(i)\n\t}}\n")
            }
            GoValueStmt::Print => format!("\tfmt.Print({var})\n"),
        });
    }
    let code = format!("package main\n\nimport \"fmt\"\n\nfunc main() {{\n{body}}}\n");
    Draft::choice("Что выведет программа?", answer, distractors).code("go", code)
}

#[derive(Clone, Copy)]
enum DeferUpdate {
    Add(i64),
    Mul(i64),
}

impl DeferUpdate {
    fn apply(self, v: i64) -> i64 {
        match self {
            DeferUpdate::Add(k) => v + k,
            DeferUpdate::Mul(k) => v * k,
        }
    }

    fn render(self, var: &str) -> String {
        match self {
            DeferUpdate::Add(k) => format!("{var} += {k}"),
            DeferUpdate::Mul(k) => format!("{var} *= {k}"),
        }
    }
}

fn go_named_results(rng: &mut Rng) -> Draft {
    let named = chance(rng, 0.6);
    let var = if named {
        *pick(rng, &["result", "res", "total", "out", "n", "sum"])
    } else {
        *pick(rng, &["x", "v", "acc", "val", "cnt", "k"])
    };
    let func = *pick(
        rng,
        &[
            "calc", "compute", "build", "eval", "score", "measure", "next", "load",
        ],
    );
    let init = range(rng, 2, 9);
    let ret_add = range(rng, 0, 5);
    let updates: Vec<DeferUpdate> = (0..range(rng, 1, 2))
        .map(|_| {
            if chance(rng, 0.5) {
                DeferUpdate::Add(range(rng, 1, 9))
            } else {
                DeferUpdate::Mul(range(rng, 2, 3))
            }
        })
        .collect();
    let returned = init + ret_add;
    let answer = if named {
        updates.iter().rev().fold(returned, |v, u| u.apply(v))
    } else {
        returned
    };

    let ret_expr = if ret_add == 0 {
        var.to_owned()
    } else {
        format!("{var} + {ret_add}")
    };
    let mut body = String::new();
    if named {
        body.push_str(&format!("func {func}() ({var} int) {{\n"));
    } else {
        body.push_str(&format!("func {func}() int {{\n\t{var} := 0\n"));
    }
    for u in &updates {
        body.push_str(&format!(
            "\tdefer func() {{\n\t\t{}\n\t}}()\n",
            u.render(var)
        ));
    }
    body.push_str(&format!("\t{var} = {init}\n\treturn {ret_expr}\n}}\n"));
    let code = format!(
        "package main\n\nimport \"fmt\"\n\n{body}\nfunc main() {{\n\tfmt.Println({func}())\n}}\n"
    );
    Draft::integer("Что выведет программа?", answer).code("go", code)
}

struct IndexTable {
    name: &'static str,
    equality: [&'static str; 6],
    time: &'static str,
    amount: &'static str,
}

const INDEX_TABLES: &[IndexTable] = &[
    IndexTable {
        name: "orders",
        equality: [
            "customer_id",
            "status",
            "region",
            "manager_id",
            "channel",
            "warehouse_id",
        ],
        time: "created_at",
        amount: "amount",
    },
    IndexTable {
        name: "payments",
        equality: [
            "merchant_id",
            "status",
            "currency",
            "method",
            "account_id",
            "country",
        ],
        time: "created_at",
        amount: "amount",
    },
    IndexTable {
        name: "tickets",
        equality: [
            "assignee_id",
            "status",
            "priority",
            "queue_id",
            "project_id",
            "reporter_id",
        ],
        time: "opened_at",
        amount: "sla_minutes",
    },
    IndexTable {
        name: "shipments",
        equality: [
            "carrier_id",
            "status",
            "warehouse_id",
            "region",
            "client_id",
            "route_id",
        ],
        time: "shipped_at",
        amount: "weight_kg",
    },
    IndexTable {
        name: "invoices",
        equality: [
            "client_id",
            "status",
            "currency",
            "contract_id",
            "manager_id",
            "branch_id",
        ],
        time: "issued_at",
        amount: "total",
    },
    IndexTable {
        name: "bookings",
        equality: [
            "hotel_id",
            "status",
            "guest_id",
            "channel",
            "room_type",
            "agent_id",
        ],
        time: "check_in",
        amount: "price",
    },
];

fn idx(cols: &[&str]) -> String {
    format!("({})", cols.join(", "))
}

fn index_choice(level: u8, rng: &mut Rng) -> Draft {
    let table = pick(rng, INDEX_TABLES);
    let name = table.name;
    let cols = pick_n(rng, &table.equality, 4);
    let (a, b, x, y) = (cols[0], cols[1], cols[2], cols[3]);
    let time = table.time;
    let range_col = if chance(rng, 0.5) { time } else { table.amount };
    match level {
        1 => {
            let correct = if chance(rng, 0.5) {
                idx(&[a])
            } else {
                idx(&[a, x])
            };
            let distractors = shuffled(
                rng,
                vec![
                    idx(&[x, a]),
                    idx(&[y]),
                    idx(&[x, y]),
                    idx(&[y, a, x]),
                    idx(&[x]),
                ],
            );
            Draft::choice(
                format!("Таблица `{name}` большая. Какой B-tree индекс позволит найти строки для запроса ниже поиском по дереву, а не полным просмотром таблицы или индекса?"),
                correct,
                distractors,
            )
            .code("sql", format!("SELECT * FROM {name}\nWHERE {a} = $1;"))
        }
        2 => {
            let (e1, e2) = if chance(rng, 0.5) { (a, b) } else { (b, a) };
            let correct = idx(&[e1, e2, range_col]);
            let distractors = shuffled(
                rng,
                vec![
                    idx(&[a, range_col, b]),
                    idx(&[range_col, a, b]),
                    idx(&[b, range_col, a]),
                    idx(&[range_col, b]),
                    idx(&[a, x, b, range_col]),
                ],
            );
            Draft::choice(
                "Какой индекс позволит ограничить диапазон сканирования B-tree всеми тремя условиями запроса?",
                correct,
                distractors,
            )
            .code(
                "sql",
                format!(
                    "SELECT * FROM {name}\nWHERE {a} = $1\n  AND {b} = $2\n  AND {range_col} > $3;"
                ),
            )
        }
        _ => {
            let two = chance(rng, 0.5);
            let (correct, distractors, filter) = if two {
                let (e1, e2) = if chance(rng, 0.5) { (a, b) } else { (b, a) };
                (
                    idx(&[e1, e2, time]),
                    vec![
                        idx(&[a, time, b]),
                        idx(&[time, a, b]),
                        idx(&[a, b]),
                        idx(&[b, time]),
                        idx(&[a, b, x, time]),
                    ],
                    format!("WHERE {a} = $1\n  AND {b} = $2"),
                )
            } else {
                (
                    idx(&[a, time]),
                    vec![idx(&[time, a]), idx(&[a]), idx(&[time]), idx(&[a, x, time])],
                    format!("WHERE {a} = $1"),
                )
            };
            let distractors = shuffled(rng, distractors);
            let dir = if chance(rng, 0.5) { "DESC" } else { "ASC" };
            let limit = *pick(rng, &[10, 20, 25, 50, 100]);
            Draft::choice(
                format!("Какой индекс позволит ограничить сканирование B-tree всеми условиями WHERE и выдать первые {limit} строк в нужном порядке без отдельной сортировки?"),
                correct,
                distractors,
            )
            .code(
                "sql",
                format!("SELECT * FROM {name}\n{filter}\nORDER BY {time} {dir}\nLIMIT {limit};"),
            )
        }
    }
}

/// Continuous-refill token bucket in fixed-point units of `1/denominator` token.
fn run_bucket(capacity: i64, rate_num: i64, denom: i64, bursts: &[(i64, i64)]) -> i64 {
    let cap = capacity * denom;
    let mut tokens = cap;
    let mut last = 0;
    let mut rejected = 0;
    for &(t, n) in bursts {
        tokens = (tokens + (t - last) * rate_num).min(cap);
        last = t;
        let served = (tokens / denom).min(n);
        tokens -= served * denom;
        rejected += n - served;
    }
    rejected
}

const RATE_LIMITED: &[&str] = &[
    "Эндпоинт `POST /api/login`",
    "Эндпоинт `GET /api/search`",
    "Эндпоинт `POST /api/orders`",
    "Эндпоинт `GET /api/v2/products`",
    "Эндпоинт `POST /api/sms/send`",
    "Публичный API погоды",
    "API платёжного шлюза",
    "API отправки уведомлений",
    "Webhook-приёмник",
    "API геокодера",
    "API загрузки файлов",
    "Внутренний API рекомендаций",
];

fn token_bucket(level: u8, rng: &mut Rng) -> Draft {
    let subject = *pick(rng, RATE_LIMITED);
    let (scope, source) = *pick(
        rng,
        &[
            ("на каждый API-ключ", "с одного API-ключа"),
            ("на каждого клиента", "от одного клиента"),
            ("на каждый IP-адрес", "с одного IP-адреса"),
            ("на каждого пользователя", "от одного пользователя"),
        ],
    );
    let capacity = range(rng, 4, 15);
    let (rate_num, denom, rate_text) = match level {
        3 => {
            if chance(rng, 0.5) {
                (1, 2, "0.5 токена в секунду".to_owned())
            } else {
                (3, 2, "1.5 токена в секунду".to_owned())
            }
        }
        _ => {
            let r = range(rng, 1, 4);
            (
                r,
                1,
                format!("{} в секунду", plural_ru(r, "токен", "токена", "токенов")),
            )
        }
    };
    let n_bursts = match level {
        1 => 2,
        2 => range(rng, 3, 4),
        _ => range(rng, 3, 4),
    } as usize;
    let mut bursts = Vec::new();
    let mut t = 0;
    for i in 0..n_bursts {
        if i > 0 {
            t += range(rng, 1, 6);
        }
        let n = if i == 0 {
            capacity + range(rng, 1, 8)
        } else {
            range(rng, 2, capacity + 4)
        };
        bursts.push((t, n));
    }
    let rejected = run_bucket(capacity, rate_num, denom, &bursts);
    let schedule = bursts
        .iter()
        .map(|(t, n)| {
            format!(
                "- t = {t} с: {}",
                plural_ru(*n, "запрос", "запроса", "запросов")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let fractional = if denom > 1 {
        " Пополнение непрерывное: дробная часть токена накапливается, но запрос может забрать только целый токен."
    } else {
        ""
    };
    let capacity_text = plural_ru(capacity, "токен", "токена", "токенов");
    Draft::integer(
        format!(
            "{subject} защищён rate limiter'ом по алгоритму token bucket ({scope}): ёмкость {capacity_text}, пополнение {rate_text}, в момент t = 0 корзина полна. \
             Каждый запрос забирает 1 токен; если токена нет, запрос отклоняется (429).{fractional}\n\n\
             Запросы {source} приходят пачками (внутри пачки — мгновенно):\n\n{schedule}\n\nСколько запросов будет отклонено всего?"
        ),
        rejected,
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Eviction {
    Lru,
    Fifo,
    Lfu,
    Mru,
}

fn run_cache(cap: usize, accesses: &[char], policy: Eviction) -> (usize, Vec<char>) {
    let mut entries: Vec<(char, usize, usize, usize)> = Vec::new();
    let mut misses = 0;
    for (t, &k) in accesses.iter().enumerate() {
        if let Some(e) = entries.iter_mut().find(|e| e.0 == k) {
            e.2 = t;
            e.3 += 1;
            continue;
        }
        misses += 1;
        if entries.len() == cap {
            let victim = match policy {
                Eviction::Lru => entries.iter().enumerate().min_by_key(|(_, e)| e.2),
                Eviction::Fifo => entries.iter().enumerate().min_by_key(|(_, e)| e.1),
                Eviction::Lfu => entries.iter().enumerate().min_by_key(|(_, e)| (e.3, e.2)),
                Eviction::Mru => entries.iter().enumerate().max_by_key(|(_, e)| e.2),
            }
            .map(|(i, _)| i);
            if let Some(i) = victim {
                entries.remove(i);
            }
        }
        entries.push((k, t, t, 1));
    }
    let mut keys: Vec<char> = entries.iter().map(|e| e.0).collect();
    keys.sort_unstable();
    (misses, keys)
}

/// Letters that cannot be confused with digits or with each other in a monospace sequence.
const CACHE_KEYS: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'K', 'M', 'N', 'P', 'R', 'S', 'T', 'X', 'Y', 'Z',
];

const CACHED: &[&str] = &[
    "профилей пользователей",
    "сессий",
    "карточек товаров",
    "DNS-ответов",
    "результатов SQL-запросов",
    "превью изображений",
    "конфигураций тенантов",
    "курсов валют",
];

fn lru_cache(level: u8, rng: &mut Rng) -> Draft {
    let (cap, len, alphabet) = match level {
        1 => (2usize, range(rng, 6, 7) as usize, 3usize),
        2 => (3, range(rng, 9, 11) as usize, 5),
        _ => (3, range(rng, 9, 10) as usize, 5),
    };
    let mut keys = pick_n(rng, CACHE_KEYS, alphabet);
    keys.sort_unstable();
    let what = *pick(rng, CACHED);
    for _ in 0..64 {
        let mut accesses: Vec<char> = Vec::with_capacity(len);
        while accesses.len() < len {
            let k = if accesses.len() >= 2 && chance(rng, 0.35) {
                accesses[accesses.len() - range(rng, 2, accesses.len().min(4) as i64) as usize]
            } else {
                *pick(rng, &keys)
            };
            if accesses.last() != Some(&k) {
                accesses.push(k);
            }
        }
        let sequence = join(&accesses, " ");
        let (misses, contents) = run_cache(cap, &accesses, Eviction::Lru);
        if level < 3 {
            let (fifo_misses, _) = run_cache(cap, &accesses, Eviction::Fifo);
            if fifo_misses == misses && chance(rng, 0.7) {
                continue;
            }
            return Draft::integer(
                format!(
                    "LRU-кэш {what} вмещает {cap} ключа и изначально пуст. Последовательность обращений:\n\n`{sequence}`\n\n\
                     Сколько будет промахов (cache miss)? Первое обращение к ключу — тоже промах."
                ),
                misses as i64,
            );
        }
        let render = |k: &[char]| join(k, ", ");
        let others: Vec<Vec<char>> = [Eviction::Fifo, Eviction::Lfu, Eviction::Mru]
            .iter()
            .map(|p| run_cache(cap, &accesses, *p).1)
            .filter(|k| *k != contents)
            .collect();
        if others.len() < 2 {
            continue;
        }
        let mut distractors: Vec<String> = others.iter().map(|k| render(k)).collect();
        let mut first_three: Vec<char> = Vec::new();
        for k in &accesses {
            if !first_three.contains(k) && first_three.len() < cap {
                first_three.push(*k);
            }
        }
        first_three.sort_unstable();
        distractors.push(render(&first_three));
        let mut combos: Vec<Vec<char>> = Vec::new();
        for i in 0..keys.len() {
            for j in i + 1..keys.len() {
                for k in j + 1..keys.len() {
                    combos.push(vec![keys[i], keys[j], keys[k]]);
                }
            }
        }
        distractors.extend(shuffled(rng, combos).iter().map(|c| render(c)));
        return Draft::choice(
            format!(
                "LRU-кэш {what} вмещает {cap} ключа и изначально пуст. Последовательность обращений (каждое обращение — чтение с загрузкой при промахе):\n\n`{sequence}`\n\n\
                 Какие ключи окажутся в кэше после последнего обращения?"
            ),
            render(&contents),
            distractors,
        );
    }
    if level < 3 {
        Draft::integer(
            format!(
                "LRU-кэш вмещает {cap} ключа и изначально пуст. Обращения: `A B A B`. Сколько будет промахов? Первое обращение к ключу — тоже промах."
            ),
            2,
        )
    } else {
        Draft::choice(
            "LRU-кэш вмещает 3 ключа. Обращения: `A B C A D`. Какие ключи останутся в кэше?",
            "A, C, D",
            ["A, B, C", "B, C, D", "A, B, D"].map(str::to_owned),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_bucket_math() {
        assert_eq!(run_bucket(5, 1, 1, &[(0, 8), (2, 3)]), 4);
        assert_eq!(run_bucket(5, 1, 2, &[(0, 5), (3, 2)]), 1);
        assert_eq!(run_bucket(4, 3, 2, &[(0, 4), (1, 2), (2, 2)]), 1);
    }

    #[test]
    fn cache_policies() {
        let acc: Vec<char> = "ABCADBEA".chars().collect();
        let (misses, keys) = run_cache(3, &acc, Eviction::Lru);
        assert_eq!(misses, 7);
        assert_eq!(keys, vec!['A', 'B', 'E']);
        let (_, fifo) = run_cache(3, &acc, Eviction::Fifo);
        assert_eq!(fifo, vec!['A', 'D', 'E']);
    }

    #[test]
    fn python_aliasing_model() {
        use AliasOp::*;
        let ops = [
            Append(1, 3),
            Append(2, 4),
            PlusRebind(1, 5),
            PlusAssign(0, 6),
        ];
        assert_eq!(run_alias(2, &ops, AliasModel::default()), [4, 4, 3]);
    }

    #[test]
    fn go_defer_semantics() {
        use GoValueStmt::*;
        let stmts = [DeferArg, Assign(5), DeferClosure, DeferLoop(2), Print];
        assert_eq!(run_go_values(&stmts, 1, GoModel::default()), "51051");
    }

    #[test]
    fn class_attribute_shadowing() {
        use CartOp::*;
        let ops = [Add(0, 1), Add(1, 2), Assign(1, 7), Add(1, 3), Add(0, 4)];
        assert_eq!(
            run_cart(&ops, CartModel::Python),
            (vec![1, 2, 4], vec![7, 3])
        );
    }
}
