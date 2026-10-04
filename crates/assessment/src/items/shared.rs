//! Templates shared by several specializations: algorithms, SQL, HTTP, git, shell.

use catalog::Specialization::{self, Backend, Devops, Frontend, Ml, Qa, Security};

use super::util::{chance, join, md_table, pick, pick_n, range, shuffled};
use super::{C_CHOICE, C_MULTI5, C_OPEN, Draft, Level, Rng, Source, Template, Topic};

const ALL_SPECS: &[Specialization] = &[Backend, Frontend, Devops, Qa, Ml, Security];

pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        id: "algo.loop_complexity",
        title: "Асимптотика вложенных циклов",
        topic: Topic::Algorithms,
        specializations: &[Backend, Frontend, Qa, Ml],
        skills: &["algorithms", "python"],
        levels: &[
            Level::new(1, 1.1, -1.2, C_CHOICE, 60),
            Level::new(2, 1.3, -0.3, C_CHOICE, 90),
            Level::new(3, 1.4, 0.7, C_CHOICE, 120),
        ],
        source: Source::Computed(loop_complexity),
    },
    Template {
        id: "algo.binary_search_trace",
        title: "Трассировка бинарного поиска",
        topic: Topic::Algorithms,
        specializations: &[Backend, Frontend, Qa, Ml],
        skills: &["algorithms", "python"],
        levels: &[
            Level::new(1, 1.0, -0.7, C_OPEN, 120),
            Level::new(2, 1.3, 0.3, C_CHOICE, 150),
            Level::new(3, 1.4, 0.9, C_OPEN, 150),
        ],
        source: Source::Computed(binary_search),
    },
    Template {
        id: "ds.stack_queue_heap",
        title: "Стек, очередь и куча",
        topic: Topic::Algorithms,
        specializations: &[Backend, Frontend, Ml],
        skills: &["data_structures", "python"],
        levels: &[
            Level::new(1, 1.0, -1.3, C_CHOICE, 60),
            Level::new(2, 1.2, -0.4, C_CHOICE, 90),
            Level::new(3, 1.3, 0.4, C_CHOICE, 90),
        ],
        source: Source::Computed(stack_queue_heap),
    },
    Template {
        id: "sql.group_by_having",
        title: "GROUP BY и HAVING",
        topic: Topic::Databases,
        specializations: &[Backend, Qa, Ml],
        skills: &["sql", "postgresql"],
        levels: &[
            Level::new(1, 1.1, -0.9, C_OPEN, 120),
            Level::new(2, 1.3, -0.1, C_OPEN, 150),
            Level::new(3, 1.5, 0.8, C_OPEN, 180),
        ],
        source: Source::Computed(group_by_having),
    },
    Template {
        id: "sql.left_join_rows",
        title: "Число строк LEFT JOIN",
        topic: Topic::Databases,
        specializations: &[Backend, Qa, Ml],
        skills: &["sql", "postgresql"],
        levels: &[
            Level::new(1, 1.2, -0.6, C_OPEN, 120),
            Level::new(2, 1.4, 0.2, C_OPEN, 150),
            Level::new(3, 1.5, 1.0, C_OPEN, 180),
        ],
        source: Source::Computed(left_join_rows),
    },
    Template {
        id: "http.status_codes",
        title: "Коды ответа HTTP",
        topic: Topic::Web,
        specializations: &[Backend, Frontend, Devops, Qa, Security],
        skills: &["http", "rest"],
        levels: &[
            Level::new(1, 1.0, -1.4, C_CHOICE, 60),
            Level::new(2, 1.2, -0.4, C_CHOICE, 60),
            Level::new(3, 1.3, 0.6, C_CHOICE, 60),
        ],
        source: Source::Computed(http_status),
    },
    Template {
        id: "http.method_semantics",
        title: "Безопасные и идемпотентные методы HTTP",
        topic: Topic::Web,
        specializations: &[Backend, Frontend, Qa],
        skills: &["http", "rest", "api_testing"],
        levels: &[
            Level::new(1, 1.0, -1.0, C_CHOICE, 60),
            Level::new(2, 1.3, 0.0, C_MULTI5, 90),
            Level::new(3, 1.4, 0.7, C_MULTI5, 90),
        ],
        source: Source::Computed(http_methods),
    },
    Template {
        id: "git.history_count",
        title: "История git после серии команд",
        topic: Topic::Tools,
        specializations: ALL_SPECS,
        skills: &["git"],
        levels: &[
            Level::new(1, 1.1, -0.8, C_OPEN, 120),
            Level::new(2, 1.3, 0.1, C_OPEN, 150),
            Level::new(3, 1.5, 0.9, C_OPEN, 180),
        ],
        source: Source::Computed(git_history),
    },
    Template {
        id: "bash.log_pipeline",
        title: "Конвейер команд над логом",
        topic: Topic::Tools,
        specializations: &[Backend, Devops, Qa, Security],
        skills: &["bash", "linux"],
        levels: &[
            Level::new(1, 1.0, -0.9, C_OPEN, 90),
            Level::new(2, 1.2, -0.1, C_CHOICE, 120),
            Level::new(3, 1.4, 0.7, C_OPEN, 150),
        ],
        source: Source::Computed(log_pipeline),
    },
];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Complexity {
    poly: u32,
    log: u32,
}

impl Complexity {
    const fn new(poly: u32, log: u32) -> Self {
        Self { poly, log }
    }

    fn times(self, other: Self) -> Self {
        Self::new(self.poly + other.poly, self.log + other.log)
    }

    fn render(self) -> String {
        fn sup(k: u32) -> &'static str {
            match k {
                2 => "²",
                3 => "³",
                4 => "⁴",
                5 => "⁵",
                _ => "",
            }
        }
        let n = match self.poly {
            0 => String::new(),
            1 => "n".to_owned(),
            p => format!("n{}", sup(p)),
        };
        let l = match self.log {
            0 => String::new(),
            1 => "log n".to_owned(),
            q => format!("log{} n", sup(q)),
        };
        match (n.is_empty(), l.is_empty()) {
            (true, true) => "O(1)".to_owned(),
            (false, true) => format!("O({n})"),
            (true, false) => format!("O({l})"),
            (false, false) => format!("O({n} {l})"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Loop {
    Linear,
    /// `range(i)` / `range(i, n)` directly inside a linear loop over `i`.
    LinearDependent,
    LinearStep(u8),
    Doubling,
    Halving,
    /// `j = i; while j > 0: j //= 2` directly inside a linear loop: Σ log i = Θ(n log n).
    HalvingDependent,
    Constant(u32),
    Square,
}

impl Loop {
    fn factor(self) -> Complexity {
        match self {
            Loop::Linear | Loop::LinearDependent | Loop::LinearStep(_) => Complexity::new(1, 0),
            Loop::Doubling | Loop::Halving | Loop::HalvingDependent => Complexity::new(0, 1),
            Loop::Constant(_) => Complexity::new(0, 0),
            Loop::Square => Complexity::new(2, 0),
        }
    }
}

const LOOP_VARS: &[&str] = &["i", "j", "k", "m"];

fn render_loops(loops: &[Loop], depth: usize, indent: usize, rng: &mut Rng, out: &mut String) {
    let pad = "    ".repeat(indent);
    let Some((&first, rest)) = loops.split_first() else {
        out.push_str(&format!("{pad}total += 1\n"));
        return;
    };
    let v = LOOP_VARS[depth];
    let outer = if depth > 0 { LOOP_VARS[depth - 1] } else { "n" };
    match first {
        Loop::Linear => out.push_str(&format!("{pad}for {v} in range(n):\n")),
        Loop::LinearDependent => {
            if chance(rng, 0.5) {
                out.push_str(&format!("{pad}for {v} in range({outer}):\n"));
            } else {
                out.push_str(&format!("{pad}for {v} in range({outer}, n):\n"));
            }
        }
        Loop::LinearStep(s) => out.push_str(&format!("{pad}for {v} in range(0, n, {s}):\n")),
        Loop::Constant(c) => out.push_str(&format!("{pad}for {v} in range({c}):\n")),
        Loop::Square => out.push_str(&format!("{pad}for {v} in range(n * n):\n")),
        Loop::Doubling => {
            out.push_str(&format!("{pad}{v} = 1\n{pad}while {v} < n:\n"));
            render_loops(rest, depth + 1, indent + 1, rng, out);
            out.push_str(&format!("{pad}    {v} *= 2\n"));
            return;
        }
        Loop::Halving => {
            out.push_str(&format!("{pad}{v} = n\n{pad}while {v} > 1:\n"));
            render_loops(rest, depth + 1, indent + 1, rng, out);
            out.push_str(&format!("{pad}    {v} //= 2\n"));
            return;
        }
        Loop::HalvingDependent => {
            out.push_str(&format!("{pad}{v} = {outer}\n{pad}while {v} > 0:\n"));
            render_loops(rest, depth + 1, indent + 1, rng, out);
            out.push_str(&format!("{pad}    {v} //= 2\n"));
            return;
        }
    }
    render_loops(rest, depth + 1, indent + 1, rng, out);
}

fn block_complexity(block: &[Loop]) -> Complexity {
    block
        .iter()
        .fold(Complexity::new(0, 0), |acc, l| acc.times(l.factor()))
}

fn loop_complexity(level: u8, rng: &mut Rng) -> Draft {
    use Loop::*;
    let blocks: Vec<Vec<Loop>> = match level {
        1 => {
            let patterns: [&[Loop]; 7] = [
                &[Linear],
                &[Linear, Linear],
                &[Linear, LinearDependent],
                &[Linear, Constant(10)],
                &[Constant(100), Linear],
                &[Linear, LinearStep(2)],
                &[Linear, Linear, Linear],
            ];
            vec![pick(rng, &patterns).to_vec()]
        }
        2 => {
            let patterns: [&[Loop]; 8] = [
                &[Doubling],
                &[Linear, Doubling],
                &[Halving, Linear],
                &[Linear, HalvingDependent],
                &[Doubling, Halving],
                &[Linear, Linear, Doubling],
                &[Constant(1000), Halving],
                &[Linear, LinearDependent, Halving],
            ];
            vec![pick(rng, &patterns).to_vec()]
        }
        _ => {
            let heavy: [&[Loop]; 6] = [
                &[Square],
                &[Square, Doubling],
                &[Linear, Linear, Halving],
                &[Linear, LinearDependent, LinearDependent],
                &[Linear, Doubling, Halving],
                &[Square, Constant(5)],
            ];
            let light: [&[Loop]; 6] = [
                &[Linear, HalvingDependent],
                &[Doubling, Doubling],
                &[Linear, Constant(50), Linear],
                &[Linear, Linear],
                &[Halving, Linear],
                &[Linear],
            ];
            let mut blocks = vec![pick(rng, &heavy).to_vec(), pick(rng, &light).to_vec()];
            if chance(rng, 0.5) {
                blocks.swap(0, 1);
            }
            blocks
        }
    };
    let complexities: Vec<Complexity> = blocks.iter().map(|b| block_complexity(b)).collect();
    let answer = complexities
        .iter()
        .copied()
        .max()
        .unwrap_or(Complexity::new(0, 0));

    let fname = pick(rng, &["count_ops", "work", "process", "solve"]).to_string();
    let mut code = format!("def {fname}(n):\n    total = 0\n");
    for block in &blocks {
        render_loops(block, 0, 1, rng, &mut code);
    }
    code.push_str("    return total\n");

    let mut candidates = vec![
        complexities
            .iter()
            .fold(Complexity::new(0, 0), |acc, c| acc.times(*c)),
        Complexity::new(answer.poly + 1, answer.log),
        Complexity::new(answer.poly, answer.log + 1),
        Complexity::new(answer.poly + 1, 0),
    ];
    if answer.log > 0 {
        candidates.push(Complexity::new(answer.poly, answer.log - 1));
        candidates.push(Complexity::new(answer.poly + 1, answer.log - 1));
    }
    if answer.poly > 0 {
        candidates.push(Complexity::new(answer.poly - 1, answer.log));
        candidates.push(Complexity::new(answer.poly - 1, answer.log + 1));
    }
    let candidates = shuffled(rng, candidates);
    let distractors = candidates
        .into_iter()
        .chain([
            Complexity::new(1, 0),
            Complexity::new(2, 0),
            Complexity::new(1, 1),
            Complexity::new(0, 1),
        ])
        .filter(|c| *c != answer)
        .map(Complexity::render);

    Draft::choice(
        format!(
            "Какова асимптотическая сложность функции `{fname}` по времени (сколько раз выполнится `total += 1` в зависимости от `n`)?"
        ),
        answer.render(),
        distractors,
    )
    .code("python", code)
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct SearchVariant {
    hi_is_len: bool,
    strict_loop: bool,
    lo_to_mid: bool,
    hi_to_mid: bool,
}

const CORRECT_SEARCH: SearchVariant = SearchVariant {
    hi_is_len: false,
    strict_loop: false,
    lo_to_mid: false,
    hi_to_mid: false,
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum SearchOutcome {
    Found(i64),
    Infinite,
    IndexError,
}

impl SearchOutcome {
    fn render(self) -> String {
        match self {
            SearchOutcome::Found(i) => format!("Вернёт {i}"),
            SearchOutcome::Infinite => "Зациклится (бесконечный цикл)".to_owned(),
            SearchOutcome::IndexError => "Упадёт с IndexError".to_owned(),
        }
    }
}

fn run_search(v: SearchVariant, arr: &[i64], target: i64) -> (SearchOutcome, u32) {
    let len = arr.len() as i64;
    let (mut lo, mut hi) = (0i64, if v.hi_is_len { len } else { len - 1 });
    let mut seen = Vec::new();
    let mut steps = 0;
    loop {
        let go = if v.strict_loop { lo < hi } else { lo <= hi };
        if !go {
            return (SearchOutcome::Found(-1), steps);
        }
        if seen.contains(&(lo, hi)) {
            return (SearchOutcome::Infinite, steps);
        }
        seen.push((lo, hi));
        steps += 1;
        let mid = (lo + hi).div_euclid(2);
        if mid < 0 || mid >= len {
            return (SearchOutcome::IndexError, steps);
        }
        let value = arr[mid as usize];
        if value == target {
            return (SearchOutcome::Found(mid), steps);
        }
        if value < target {
            lo = if v.lo_to_mid { mid } else { mid + 1 };
        } else {
            hi = if v.hi_to_mid { mid } else { mid - 1 };
        }
    }
}

fn render_search(v: SearchVariant, count_steps: bool) -> String {
    let hi = if v.hi_is_len {
        "len(arr)"
    } else {
        "len(arr) - 1"
    };
    let cond = if v.strict_loop { "lo < hi" } else { "lo <= hi" };
    let lo_upd = if v.lo_to_mid { "mid" } else { "mid + 1" };
    let hi_upd = if v.hi_to_mid { "mid" } else { "mid - 1" };
    if count_steps {
        format!(
            "def search(arr, target):\n    lo, hi = 0, {hi}\n    steps = 0\n    while {cond}:\n        steps += 1\n        mid = (lo + hi) // 2\n        if arr[mid] == target:\n            return steps\n        if arr[mid] < target:\n            lo = {lo_upd}\n        else:\n            hi = {hi_upd}\n    return steps\n"
        )
    } else {
        format!(
            "def find(arr, target):\n    lo, hi = 0, {hi}\n    while {cond}:\n        mid = (lo + hi) // 2\n        if arr[mid] == target:\n            return mid\n        if arr[mid] < target:\n            lo = {lo_upd}\n        else:\n            hi = {hi_upd}\n    return -1\n"
        )
    }
}

fn sorted_distinct(rng: &mut Rng, len: usize) -> Vec<i64> {
    let mut v = Vec::with_capacity(len);
    let mut x = range(rng, 1, 9);
    for _ in 0..len {
        v.push(x);
        x += range(rng, 2, 9);
    }
    v
}

fn absent_value(rng: &mut Rng, arr: &[i64]) -> i64 {
    loop {
        let x = range(rng, arr[0] - 3, arr[arr.len() - 1] + 6);
        if !arr.contains(&x) {
            return x;
        }
    }
}

fn binary_search(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 => {
            let len = range(rng, 8, 15) as usize;
            let arr = sorted_distinct(rng, len);
            let target = if chance(rng, 0.6) {
                *pick(rng, &arr)
            } else {
                absent_value(rng, &arr)
            };
            let (_, steps) = run_search(CORRECT_SEARCH, &arr, target);
            Draft::integer(
                format!(
                    "Что вернёт вызов `search({}, {target})`?",
                    format_args!("[{}]", join(&arr, ", "))
                ),
                i64::from(steps),
            )
            .code("python", render_search(CORRECT_SEARCH, true))
        }
        2 => {
            let bugs = [
                SearchVariant {
                    strict_loop: true,
                    ..CORRECT_SEARCH
                },
                SearchVariant {
                    lo_to_mid: true,
                    ..CORRECT_SEARCH
                },
                SearchVariant {
                    hi_is_len: true,
                    ..CORRECT_SEARCH
                },
                SearchVariant {
                    hi_to_mid: true,
                    ..CORRECT_SEARCH
                },
            ];
            let variant = *pick(rng, &bugs);
            let len = range(rng, 5, 9) as usize;
            let mut arr = sorted_distinct(rng, len);
            let mut target = arr[0];
            for _ in 0..64 {
                arr = sorted_distinct(rng, arr.len());
                target = if chance(rng, 0.5) {
                    *pick(rng, &arr)
                } else if chance(rng, 0.5) {
                    arr[arr.len() - 1] + range(rng, 1, 5)
                } else {
                    absent_value(rng, &arr)
                };
                if run_search(variant, &arr, target).0 != run_search(CORRECT_SEARCH, &arr, target).0
                {
                    break;
                }
            }
            let (outcome, _) = run_search(variant, &arr, target);
            let (expected, _) = run_search(CORRECT_SEARCH, &arr, target);
            let mut distractors = vec![
                expected,
                SearchOutcome::Found(-1),
                SearchOutcome::Infinite,
                SearchOutcome::IndexError,
            ];
            if let Some(i) = arr.iter().position(|&x| x == target) {
                distractors.push(SearchOutcome::Found(i as i64 + 1));
            }
            distractors.push(SearchOutcome::Found(0));
            let distractors = shuffled(rng, distractors);
            Draft::choice(
                format!(
                    "В реализации бинарного поиска может быть ошибка. Что произойдёт при вызове `find([{}], {target})`?",
                    join(&arr, ", ")
                ),
                outcome.render(),
                distractors.into_iter().filter(|d| *d != outcome).map(SearchOutcome::render),
            )
            .code("python", render_search(variant, false))
        }
        _ => {
            let len = range(rng, 8, 12) as usize;
            let mut arr: Vec<i64> = (0..len).map(|_| range(rng, 1, 9)).collect();
            arr.sort_unstable();
            let strict = chance(rng, 0.5);
            let x = if chance(rng, 0.7) {
                *pick(rng, &arr)
            } else {
                range(rng, 0, 10)
            };
            let answer = arr
                .iter()
                .filter(|&&v| if strict { v < x } else { v <= x })
                .count();
            let cmp = if strict { "<" } else { "<=" };
            let code = format!(
                "def bound(arr, x):\n    lo, hi = 0, len(arr)\n    while lo < hi:\n        mid = (lo + hi) // 2\n        if arr[mid] {cmp} x:\n            lo = mid + 1\n        else:\n            hi = mid\n    return lo\n"
            );
            Draft::integer(
                format!("Что вернёт вызов `bound([{}], {x})`?", join(&arr, ", ")),
                answer as i64,
            )
            .code("python", code)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ContainerOp {
    Push(i64),
    PushLeft(i64),
    Pop,
    PopLeft,
}

#[derive(Clone, Copy, PartialEq)]
enum Semantics {
    Stack,
    Fifo,
    Deque,
    DequeSwapped,
    DequeAppendLeftAsAppend,
    MinHeap,
    MaxHeap,
}

fn run_container(ops: &[ContainerOp], sem: Semantics) -> (Vec<i64>, Vec<i64>) {
    let mut items: Vec<i64> = Vec::new();
    let mut out = Vec::new();
    for &op in ops {
        let from_back = match op {
            ContainerOp::Push(v) => {
                items.push(v);
                continue;
            }
            ContainerOp::PushLeft(v) => {
                if sem == Semantics::DequeAppendLeftAsAppend {
                    items.push(v);
                } else {
                    items.insert(0, v);
                }
                continue;
            }
            ContainerOp::Pop => !matches!(sem, Semantics::Fifo | Semantics::DequeSwapped),
            ContainerOp::PopLeft => sem == Semantics::DequeSwapped,
        };
        let index = match sem {
            Semantics::MinHeap => items
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| **v)
                .map(|(i, _)| i),
            Semantics::MaxHeap => items
                .iter()
                .enumerate()
                .max_by_key(|(_, v)| **v)
                .map(|(i, _)| i),
            _ if from_back => items.len().checked_sub(1),
            _ => (!items.is_empty()).then_some(0),
        };
        if let Some(i) = index {
            out.push(items.remove(i));
        }
    }
    (out, items)
}

fn py_list(v: &[i64]) -> String {
    format!("[{}]", join(v, ", "))
}

fn stack_queue_heap(level: u8, rng: &mut Rng) -> Draft {
    let n_ops = match level {
        1 => range(rng, 6, 8),
        _ => range(rng, 7, 9),
    } as usize;
    let mut ops = Vec::new();
    let mut size = 0usize;
    let mut used = Vec::new();
    let mut fresh = |rng: &mut Rng| loop {
        let v = range(rng, 1, 20);
        if !used.contains(&v) {
            used.push(v);
            return v;
        }
    };
    let mut pops = 0;
    let min_pops = if level == 3 { 3 } else { 2 };
    while ops.len() < n_ops || pops < min_pops || size == 0 {
        let pop = size > 0 && chance(rng, if size > 2 { 0.5 } else { 0.3 });
        let op = match (level, pop) {
            (2, true) => {
                if chance(rng, 0.5) {
                    ContainerOp::Pop
                } else {
                    ContainerOp::PopLeft
                }
            }
            (_, true) => ContainerOp::Pop,
            (2, false) => {
                let v = fresh(rng);
                if chance(rng, 0.4) {
                    ContainerOp::PushLeft(v)
                } else {
                    ContainerOp::Push(v)
                }
            }
            (_, false) => ContainerOp::Push(fresh(rng)),
        };
        if pop {
            size -= 1;
            pops += 1;
        } else {
            size += 1;
        }
        ops.push(op);
        if ops.len() > 16 {
            break;
        }
    }

    let (sem, wrong, code_lines, header, show_rest): (
        Semantics,
        Vec<Semantics>,
        Vec<String>,
        &str,
        bool,
    ) = match level {
        1 => (
            Semantics::Stack,
            vec![Semantics::Fifo],
            ops.iter()
                .map(|op| match op {
                    ContainerOp::Push(v) => format!("stack.append({v})"),
                    _ => "out.append(stack.pop())".to_owned(),
                })
                .collect(),
            "stack = []\nout = []\n",
            true,
        ),
        2 => (
            Semantics::Deque,
            vec![Semantics::DequeSwapped, Semantics::DequeAppendLeftAsAppend],
            ops.iter()
                .map(|op| match op {
                    ContainerOp::Push(v) => format!("d.append({v})"),
                    ContainerOp::PushLeft(v) => format!("d.appendleft({v})"),
                    ContainerOp::Pop => "out.append(d.pop())".to_owned(),
                    ContainerOp::PopLeft => "out.append(d.popleft())".to_owned(),
                })
                .collect(),
            "from collections import deque\n\nd = deque()\nout = []\n",
            true,
        ),
        _ => (
            Semantics::MinHeap,
            vec![Semantics::MaxHeap, Semantics::Fifo, Semantics::Stack],
            ops.iter()
                .map(|op| match op {
                    ContainerOp::Push(v) => format!("heapq.heappush(h, {v})"),
                    _ => "out.append(heapq.heappop(h))".to_owned(),
                })
                .collect(),
            "import heapq\n\nh = []\nout = []\n",
            false,
        ),
    };
    let render = |(out, rest): (Vec<i64>, Vec<i64>)| {
        if show_rest {
            format!("{} {}", py_list(&out), py_list(&rest))
        } else {
            py_list(&out)
        }
    };
    let (out, rest) = run_container(&ops, sem);
    let answer = render((out.clone(), rest.clone()));
    let reversed = |v: &[i64]| v.iter().rev().copied().collect::<Vec<_>>();
    let pushed: Vec<i64> = ops
        .iter()
        .filter_map(|op| match op {
            ContainerOp::Push(v) | ContainerOp::PushLeft(v) => Some(*v),
            _ => None,
        })
        .collect();
    let mut candidates: Vec<(Vec<i64>, Vec<i64>)> =
        wrong.iter().map(|w| run_container(&ops, *w)).collect();
    candidates.push((reversed(&out), rest.clone()));
    candidates.push((out.clone(), reversed(&rest)));
    candidates.push((out.clone(), pushed));
    let mut rotated = out.clone();
    rotated.rotate_left(1);
    candidates.push((rotated, rest.clone()));
    let mut sorted = out.clone();
    sorted.sort_unstable();
    candidates.push((sorted.clone(), rest.clone()));
    candidates.push((reversed(&sorted), rest.clone()));
    if let (Some(&o), Some(&r)) = (out.last(), rest.first()) {
        let (mut o2, mut r2) = (out.clone(), rest.clone());
        let last = o2.len() - 1;
        o2[last] = r;
        r2[0] = o;
        candidates.push((o2, r2));
    }
    for (i, value) in rest.iter().enumerate().take(3) {
        let mut o2 = out.clone();
        let k = i % o2.len().max(1);
        if let Some(slot) = o2.get_mut(k) {
            *slot = *value;
        }
        candidates.push((o2, rest.clone()));
    }
    let distractors: Vec<String> = candidates.into_iter().map(render).collect();

    let tail = match level {
        1 => "print(out, stack)",
        2 => "print(out, list(d))",
        _ => "print(out)",
    };
    let code = format!("{header}{}\n{tail}\n", code_lines.join("\n"));
    Draft::choice("Что выведет программа?", answer, distractors).code("python", code)
}

#[derive(Clone)]
struct OrderRow {
    customer: &'static str,
    status: &'static str,
    amount: Option<i64>,
}

fn order_rows(rng: &mut Rng, with_nulls: bool) -> (Vec<&'static str>, Vec<OrderRow>) {
    let count = range(rng, 4, 5) as usize;
    let customers = pick_n(rng, super::util::PEOPLE, count);
    let n = range(rng, 8, 10) as usize;
    let mut rows = Vec::with_capacity(n);
    for i in 0..n {
        let customer = if i < customers.len() {
            customers[i]
        } else {
            *pick(rng, &customers)
        };
        let status = *pick(rng, &["paid", "paid", "new", "cancelled"]);
        let amount = if with_nulls && chance(rng, 0.25) {
            None
        } else {
            Some(range(rng, 2, 40) * 50)
        };
        rows.push(OrderRow {
            customer,
            status,
            amount,
        });
    }
    let rows = shuffled(rng, rows);
    (customers, rows)
}

fn orders_table(rows: &[OrderRow]) -> String {
    let body: Vec<Vec<String>> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            vec![
                (i + 1).to_string(),
                r.customer.to_owned(),
                r.status.to_owned(),
                r.amount.map_or("NULL".to_owned(), |a| a.to_string()),
            ]
        })
        .collect();
    md_table(&["id", "customer", "status", "amount"], &body)
}

/// A threshold strictly between two distinct values, so that `> t` splits them.
fn threshold_between(rng: &mut Rng, values: &[i64]) -> Option<i64> {
    let mut v = values.to_vec();
    v.sort_unstable();
    v.dedup();
    if v.len() < 2 {
        return None;
    }
    let i = range(rng, 0, v.len() as i64 - 2) as usize;
    let (lo, hi) = (v[i], v[i + 1]);
    let mid = (lo + hi) / 2;
    let t = (mid / 50) * 50;
    Some(if t >= lo && t < hi { t } else { lo })
}

fn group_by_having(level: u8, rng: &mut Rng) -> Draft {
    for _ in 0..32 {
        let (customers, rows) = order_rows(rng, level == 3);
        let table = orders_table(&rows);
        let (sql, answer) = match level {
            1 => {
                let counts: Vec<i64> = customers
                    .iter()
                    .map(|c| rows.iter().filter(|r| r.customer == *c).count() as i64)
                    .collect();
                let k = range(rng, 2, 3);
                let answer = counts.iter().filter(|&&c| c >= k).count() as i64;
                if answer == 0 || answer == counts.len() as i64 {
                    continue;
                }
                (
                    format!(
                        "SELECT customer, COUNT(*) AS orders_count\nFROM orders\nGROUP BY customer\nHAVING COUNT(*) >= {k};"
                    ),
                    answer,
                )
            }
            2 => {
                let sums: Vec<i64> = customers
                    .iter()
                    .filter_map(|c| {
                        let paid: Vec<i64> = rows
                            .iter()
                            .filter(|r| r.customer == *c && r.status == "paid")
                            .filter_map(|r| r.amount)
                            .collect();
                        (!paid.is_empty()).then(|| paid.iter().sum())
                    })
                    .collect();
                let Some(t) = threshold_between(rng, &sums) else {
                    continue;
                };
                let answer = sums.iter().filter(|&&s| s > t).count() as i64;
                if answer == 0 {
                    continue;
                }
                (
                    format!(
                        "SELECT customer, SUM(amount) AS total\nFROM orders\nWHERE status = 'paid'\nGROUP BY customer\nHAVING SUM(amount) > {t};"
                    ),
                    answer,
                )
            }
            _ => {
                let stats: Vec<(i64, f64, f64)> = customers
                    .iter()
                    .map(|c| {
                        let all: Vec<&OrderRow> =
                            rows.iter().filter(|r| r.customer == *c).collect();
                        let vals: Vec<i64> = all.iter().filter_map(|r| r.amount).collect();
                        let avg = if vals.is_empty() {
                            f64::NAN
                        } else {
                            vals.iter().sum::<i64>() as f64 / vals.len() as f64
                        };
                        let naive = vals.iter().sum::<i64>() as f64 / all.len() as f64;
                        (vals.len() as i64, avg, naive)
                    })
                    .collect();
                if !rows.iter().any(|r| r.amount.is_none()) {
                    continue;
                }
                let eligible: Vec<i64> = stats
                    .iter()
                    .filter(|s| s.0 >= 2)
                    .map(|s| s.1.round() as i64)
                    .collect();
                let Some(t) = threshold_between(rng, &eligible) else {
                    continue;
                };
                let t = t as f64;
                let answer = stats.iter().filter(|s| s.0 >= 2 && s.1 > t).count() as i64;
                let naive = stats.iter().filter(|s| s.0 >= 2 && s.2 > t).count() as i64;
                if answer == 0 || (naive == answer && chance(rng, 0.7)) {
                    continue;
                }
                if stats.iter().any(|s| s.0 >= 2 && (s.1 - t).abs() < 1.0) {
                    continue;
                }
                (
                    format!(
                        "SELECT customer\nFROM orders\nGROUP BY customer\nHAVING COUNT(amount) >= 2 AND AVG(amount) > {t};"
                    ),
                    answer,
                )
            }
        };
        return Draft::integer(
            format!("Таблица `orders` (PostgreSQL):\n\n{table}\nСколько строк вернёт запрос?"),
            answer,
        )
        .code("sql", sql);
    }
    Draft::integer("Таблица `orders` пуста. Сколько строк вернёт запрос?", 0).code(
        "sql",
        "SELECT customer, COUNT(*)\nFROM orders\nGROUP BY customer;",
    )
}

fn left_join_rows(level: u8, rng: &mut Rng) -> Draft {
    for _ in 0..32 {
        let count = range(rng, 4, 6) as usize;
        let names = pick_n(rng, super::util::PEOPLE, count);
        let users = names.len() as i64;
        let n_orders = range(rng, 5, 8);
        let mut orders: Vec<(i64, i64)> = (0..n_orders)
            .map(|_| (range(rng, 1, users), range(rng, 1, 20) * 100))
            .collect();
        if level == 1 && chance(rng, 0.6) {
            orders.push((users + range(rng, 1, 3), range(rng, 1, 20) * 100));
        }
        let orders = shuffled(rng, orders);
        let per_user = |pred: &dyn Fn(i64) -> bool| -> Vec<i64> {
            (1..=users)
                .map(|u| {
                    orders
                        .iter()
                        .filter(|(uid, amt)| *uid == u && pred(*amt))
                        .count() as i64
                })
                .collect()
        };
        let all = per_user(&|_| true);
        if !all.contains(&0) || !all.iter().any(|&c| c >= 2) {
            continue;
        }
        let amounts: Vec<i64> = orders.iter().map(|o| o.1).collect();
        let (sql, answer) = match level {
            1 => (
                "SELECT u.name, o.id\nFROM users u\nLEFT JOIN orders o ON o.user_id = u.id;"
                    .to_owned(),
                all.iter().map(|&c| c.max(1)).sum::<i64>(),
            ),
            2 => {
                let Some(t) = threshold_between(rng, &amounts) else {
                    continue;
                };
                let matched = per_user(&|a| a > t);
                (
                    format!(
                        "SELECT u.name, o.id\nFROM users u\nLEFT JOIN orders o ON o.user_id = u.id\nWHERE o.amount > {t};"
                    ),
                    matched.iter().sum::<i64>(),
                )
            }
            _ => {
                let Some(t) = threshold_between(rng, &amounts) else {
                    continue;
                };
                let matched = per_user(&|a| a > t);
                let answer = matched.iter().map(|&c| c.max(1)).sum::<i64>();
                if answer == all.iter().map(|&c| c.max(1)).sum::<i64>() {
                    continue;
                }
                (
                    format!(
                        "SELECT u.name, o.id\nFROM users u\nLEFT JOIN orders o\n  ON o.user_id = u.id AND o.amount > {t};"
                    ),
                    answer,
                )
            }
        };
        let users_table = md_table(
            &["id", "name"],
            &names
                .iter()
                .enumerate()
                .map(|(i, n)| vec![(i + 1).to_string(), (*n).to_owned()])
                .collect::<Vec<_>>(),
        );
        let orders_table = md_table(
            &["id", "user_id", "amount"],
            &orders
                .iter()
                .enumerate()
                .map(|(i, (u, a))| vec![(100 + i + 1).to_string(), u.to_string(), a.to_string()])
                .collect::<Vec<_>>(),
        );
        return Draft::integer(
            format!(
                "Таблица `users`:\n\n{users_table}\nТаблица `orders`:\n\n{orders_table}\nСколько строк вернёт запрос?"
            ),
            answer,
        )
        .code("sql", sql);
    }
    Draft::integer(
        "Таблицы `users` и `orders` пусты. Сколько строк вернёт запрос?",
        0,
    )
    .code(
        "sql",
        "SELECT u.name, o.id\nFROM users u\nLEFT JOIN orders o ON o.user_id = u.id;",
    )
}

struct StatusCase {
    code: &'static str,
    level: u8,
    stems: &'static [&'static str],
    confusable: &'static [&'static str],
    /// Codes that some style guides accept for the scenario: never shown as distractors.
    exclude: &'static [&'static str],
}

const STATUS_CASES: &[StatusCase] = &[
    StatusCase {
        code: "200 OK",
        level: 1,
        stems: &[
            "Клиент запросил `GET /api/products/42`, товар найден и возвращён в теле ответа. Какой код ответа уместен?",
            "Запрос `GET /api/profile` выполнен успешно, сервер возвращает JSON с данными профиля. Какой код ответа?",
        ],
        confusable: &[
            "201 Created",
            "204 No Content",
            "304 Not Modified",
            "202 Accepted",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "201 Created",
        level: 1,
        stems: &[
            "`POST /api/orders` создал новый заказ; в ответе — его представление и заголовок `Location`. Какой код ответа?",
            "Регистрация пользователя через `POST /api/users` прошла успешно, новый ресурс создан. Какой код ответа?",
        ],
        confusable: &[
            "202 Accepted",
            "204 No Content",
            "302 Found",
            "304 Not Modified",
        ],
        exclude: &["200 OK"],
    },
    StatusCase {
        code: "400 Bad Request",
        level: 1,
        stems: &[
            "Клиент прислал тело с синтаксически некорректным JSON (не закрыта фигурная скобка). Какой код ответа?",
            "Сервер не смог разобрать запрос: тело `{\"name\": \"x\",,}` не является валидным JSON. Какой код ответа?",
        ],
        confusable: &[
            "401 Unauthorized",
            "404 Not Found",
            "500 Internal Server Error",
            "415 Unsupported Media Type",
        ],
        exclude: &["422 Unprocessable Content"],
    },
    StatusCase {
        code: "401 Unauthorized",
        level: 1,
        stems: &[
            "Запрос к защищённому API пришёл без заголовка `Authorization`. Какой код ответа?",
            "Срок действия access-токена истёк, клиент должен заново аутентифицироваться. Какой код ответа?",
        ],
        confusable: &[
            "403 Forbidden",
            "400 Bad Request",
            "407 Proxy Authentication Required",
            "419 Page Expired",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "403 Forbidden",
        level: 1,
        stems: &[
            "Пользователь аутентифицирован, но его роли не разрешено удалять чужие проекты. Какой код ответа?",
            "Токен валиден, однако у сотрудника нет прав на просмотр раздела бухгалтерии. Какой код ответа?",
        ],
        confusable: &[
            "401 Unauthorized",
            "405 Method Not Allowed",
            "400 Bad Request",
            "407 Proxy Authentication Required",
        ],
        exclude: &["404 Not Found"],
    },
    StatusCase {
        code: "404 Not Found",
        level: 1,
        stems: &[
            "`GET /api/users/9999` — пользователя с таким id не существует. Какой код ответа?",
            "Клиент обратился по адресу `/api/v1/reprots` (опечатка в пути), такого ресурса нет. Какой код ответа?",
        ],
        confusable: &[
            "400 Bad Request",
            "403 Forbidden",
            "204 No Content",
            "405 Method Not Allowed",
        ],
        exclude: &["410 Gone"],
    },
    StatusCase {
        code: "500 Internal Server Error",
        level: 1,
        stems: &[
            "Обработчик корректного запроса упал с необработанным исключением `NullPointerException`. Какой код ответа?",
            "В коде сервера произошла непредвиденная ошибка (деление на ноль), хотя запрос клиента корректен. Какой код ответа?",
        ],
        confusable: &[
            "400 Bad Request",
            "502 Bad Gateway",
            "503 Service Unavailable",
            "504 Gateway Timeout",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "204 No Content",
        level: 2,
        stems: &[
            "`DELETE /api/files/17` выполнен успешно, тело ответа не нужно. Какой код ответа точнее всего?",
            "`PUT /api/settings` сохранил настройки, сервер ничего не возвращает в теле. Какой код ответа точнее всего?",
        ],
        confusable: &[
            "201 Created",
            "205 Reset Content",
            "304 Not Modified",
            "404 Not Found",
        ],
        exclude: &["200 OK", "202 Accepted"],
    },
    StatusCase {
        code: "304 Not Modified",
        level: 2,
        stems: &[
            "Браузер прислал `If-None-Match` с ETag, который совпадает с текущей версией ресурса. Какой код ответа?",
            "Клиент сделал условный GET с `If-Modified-Since`, ресурс с тех пор не менялся. Какой код ответа?",
        ],
        confusable: &[
            "204 No Content",
            "200 OK",
            "412 Precondition Failed",
            "302 Found",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "405 Method Not Allowed",
        level: 2,
        stems: &[
            "Эндпоинт `/api/reports` поддерживает только GET, а клиент отправил DELETE. Какой код ответа?",
            "На ресурс, доступный только для чтения, пришёл `POST`; метод для него не поддерживается. Какой код ответа (вместе с заголовком `Allow`)?",
        ],
        confusable: &[
            "403 Forbidden",
            "400 Bad Request",
            "404 Not Found",
            "409 Conflict",
        ],
        exclude: &["501 Not Implemented"],
    },
    StatusCase {
        code: "409 Conflict",
        level: 2,
        stems: &[
            "Попытка перевести заказ в статус «отправлен», хотя он уже отменён: запрос конфликтует с текущим состоянием ресурса. Какой код ответа?",
            "Два пользователя редактировали один документ, и сервер обнаружил, что второе сохранение конфликтует с текущим состоянием ресурса. Какой код ответа?",
        ],
        confusable: &[
            "403 Forbidden",
            "404 Not Found",
            "429 Too Many Requests",
            "500 Internal Server Error",
        ],
        exclude: &[
            "400 Bad Request",
            "422 Unprocessable Content",
            "412 Precondition Failed",
        ],
    },
    StatusCase {
        code: "429 Too Many Requests",
        level: 2,
        stems: &[
            "Клиент превысил лимит 100 запросов в минуту. Какой код ответа (обычно с заголовком `Retry-After`)?",
            "Сработал rate limiter: с одного IP пришло слишком много попыток входа. Какой код ответа?",
        ],
        confusable: &[
            "403 Forbidden",
            "409 Conflict",
            "400 Bad Request",
            "423 Locked",
        ],
        exclude: &["503 Service Unavailable"],
    },
    StatusCase {
        code: "503 Service Unavailable",
        level: 2,
        stems: &[
            "Сервис временно выведен на техническое обслуживание и вернётся через 10 минут. Какой код ответа?",
            "Сервер перегружен и временно не принимает запросы; клиенту стоит повторить позже согласно `Retry-After`. Какой код ответа?",
        ],
        confusable: &[
            "500 Internal Server Error",
            "502 Bad Gateway",
            "504 Gateway Timeout",
            "501 Not Implemented",
        ],
        exclude: &["429 Too Many Requests"],
    },
    StatusCase {
        code: "301 Moved Permanently",
        level: 2,
        stems: &[
            "Страница навсегда переехала с `/old-blog` на `/blog`, поисковики должны обновить ссылку. Какой код ответа для GET-запросов?",
            "Домен сменился навсегда: все GET-запросы к `old.example.ru` нужно постоянно перенаправлять на `example.ru`. Какой код ответа?",
        ],
        confusable: &[
            "302 Found",
            "307 Temporary Redirect",
            "303 See Other",
            "304 Not Modified",
        ],
        exclude: &["308 Permanent Redirect"],
    },
    StatusCase {
        code: "502 Bad Gateway",
        level: 3,
        stems: &[
            "Nginx проксирует запрос на приложение, но процесс приложения упал, и соединение с upstream отклонено. Какой код вернёт nginx?",
            "Обратный прокси получил от вышестоящего сервера некорректный ответ. Какой код он вернёт клиенту?",
        ],
        confusable: &[
            "504 Gateway Timeout",
            "503 Service Unavailable",
            "500 Internal Server Error",
            "501 Not Implemented",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "504 Gateway Timeout",
        level: 3,
        stems: &[
            "Nginx ждал ответа приложения дольше `proxy_read_timeout` и не дождался. Какой код получит клиент?",
            "API-шлюз не получил ответ от вышестоящего сервиса за отведённое время. Какой код ответа вернёт шлюз?",
        ],
        confusable: &[
            "502 Bad Gateway",
            "408 Request Timeout",
            "503 Service Unavailable",
            "500 Internal Server Error",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "307 Temporary Redirect",
        level: 3,
        stems: &[
            "Нужно временно перенаправить `POST /api/upload` на другой сервер так, чтобы клиент повторил именно POST с тем же телом. Какой код ответа?",
            "Нужен временный редирект, при котором метод и тело запроса гарантированно сохраняются. Какой код ответа?",
        ],
        confusable: &[
            "302 Found",
            "303 See Other",
            "308 Permanent Redirect",
            "301 Moved Permanently",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "308 Permanent Redirect",
        level: 3,
        stems: &[
            "API навсегда переехал на новый адрес, и клиенты должны повторять даже `POST` и `PUT` без смены метода. Какой код редиректа?",
            "Нужен постоянный редирект с гарантией сохранения метода и тела запроса. Какой код ответа?",
        ],
        confusable: &[
            "301 Moved Permanently",
            "307 Temporary Redirect",
            "302 Found",
            "303 See Other",
        ],
        exclude: &[],
    },
    StatusCase {
        code: "412 Precondition Failed",
        level: 3,
        stems: &[
            "Клиент отправил `PUT` с заголовком `If-Match: \"v7\"`, а текущая версия ресурса — `\"v8\"`. Какой код ответа?",
            "Условный запрос на обновление не выполнен: ETag в `If-Match` не совпал с актуальным. Какой код ответа?",
        ],
        confusable: &[
            "304 Not Modified",
            "400 Bad Request",
            "428 Precondition Required",
            "417 Expectation Failed",
        ],
        exclude: &["409 Conflict"],
    },
    StatusCase {
        code: "415 Unsupported Media Type",
        level: 3,
        stems: &[
            "Эндпоинт принимает только `application/json`, а клиент отправил тело с `Content-Type: text/xml`. Какой код ответа?",
            "Сервер не умеет обрабатывать формат тела, указанный в `Content-Type` запроса. Какой код ответа?",
        ],
        confusable: &[
            "406 Not Acceptable",
            "422 Unprocessable Content",
            "501 Not Implemented",
            "405 Method Not Allowed",
        ],
        exclude: &["400 Bad Request"],
    },
    StatusCase {
        code: "422 Unprocessable Content",
        level: 3,
        stems: &[
            "JSON синтаксически корректен, но поле `age` равно `-5`, что нарушает правила валидации. Какой код точнее всего по RFC 9110?",
            "Тело запроса разобрано успешно, но семантически неверно: дата окончания раньше даты начала. Какой код точнее всего по RFC 9110?",
        ],
        confusable: &[
            "415 Unsupported Media Type",
            "500 Internal Server Error",
            "406 Not Acceptable",
            "412 Precondition Failed",
        ],
        exclude: &["400 Bad Request", "409 Conflict"],
    },
    StatusCase {
        code: "406 Not Acceptable",
        level: 3,
        stems: &[
            "Клиент прислал `Accept: application/xml`, а сервер умеет отдавать только JSON и отказывает. Какой код ответа?",
            "Сервер не может сформировать ответ ни в одном из форматов, перечисленных клиентом в заголовке `Accept`. Какой код ответа?",
        ],
        confusable: &[
            "415 Unsupported Media Type",
            "400 Bad Request",
            "422 Unprocessable Content",
            "501 Not Implemented",
        ],
        exclude: &[],
    },
];

fn http_status(level: u8, rng: &mut Rng) -> Draft {
    let cases: Vec<&StatusCase> = STATUS_CASES.iter().filter(|c| c.level == level).collect();
    let case = *pick(rng, &cases);
    let stem = *pick(rng, case.stems);
    let allowed = |code: &&str| *code != case.code && !case.exclude.contains(code);
    let confusable: Vec<&str> = case.confusable.iter().copied().filter(allowed).collect();
    let mut distractors: Vec<String> = pick_n(rng, &confusable, 2)
        .into_iter()
        .map(str::to_owned)
        .collect();
    let others: Vec<&str> = STATUS_CASES
        .iter()
        .map(|c| c.code)
        .filter(allowed)
        .collect();
    distractors.extend(shuffled(rng, others).into_iter().map(str::to_owned));
    distractors.extend(confusable.into_iter().map(str::to_owned));
    Draft::choice(stem, case.code, distractors)
}

#[derive(Clone, Copy)]
struct Method {
    name: &'static str,
    safe: bool,
    idempotent: bool,
}

const METHODS: &[Method] = &[
    Method {
        name: "GET",
        safe: true,
        idempotent: true,
    },
    Method {
        name: "HEAD",
        safe: true,
        idempotent: true,
    },
    Method {
        name: "OPTIONS",
        safe: true,
        idempotent: true,
    },
    Method {
        name: "PUT",
        safe: false,
        idempotent: true,
    },
    Method {
        name: "DELETE",
        safe: false,
        idempotent: true,
    },
    Method {
        name: "POST",
        safe: false,
        idempotent: false,
    },
    Method {
        name: "PATCH",
        safe: false,
        idempotent: false,
    },
];

fn http_methods(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 => {
            let ask_non_idempotent = chance(rng, 0.5);
            let (good, bad): (Vec<&Method>, Vec<&Method>) = if ask_non_idempotent {
                METHODS.iter().partition(|m| !m.idempotent)
            } else {
                METHODS.iter().partition(|m| m.safe)
            };
            let correct = pick(rng, &good).name;
            let distractors = pick_n(rng, &bad, 3).into_iter().map(|m| m.name.to_owned());
            let stem = if ask_non_idempotent {
                *pick(
                    rng,
                    &[
                        "Какой из методов HTTP **не** является идемпотентным по RFC 9110?",
                        "Повтор какого запроса может создать дубликат, потому что метод не идемпотентен по RFC 9110?",
                    ],
                )
            } else {
                *pick(
                    rng,
                    &[
                        "Какой из методов HTTP является безопасным (safe), то есть не должен менять состояние сервера?",
                        "Какой метод по семантике HTTP предназначен только для чтения и считается безопасным (safe)?",
                    ],
                )
            };
            Draft::choice(stem, correct, distractors)
        }
        _ => {
            let is_target = |m: &Method| {
                if level == 2 {
                    m.idempotent
                } else {
                    m.idempotent && !m.safe
                }
            };
            loop {
                let chosen = pick_n(rng, METHODS, 5);
                let hits = chosen.iter().filter(|m| is_target(m)).count();
                if hits == 0 || hits == chosen.len() {
                    continue;
                }
                let stem = if level == 2 {
                    *pick(
                        rng,
                        &[
                            "Отметьте все идемпотентные методы HTTP (по RFC 9110).",
                            "Клиент повторил запрос из-за таймаута. Повтор каких методов по семантике HTTP не меняет итоговое состояние сервера (идемпотентны)? Отметьте все.",
                        ],
                    )
                } else {
                    *pick(
                        rng,
                        &[
                            "Отметьте методы, которые идемпотентны, но **не** являются безопасными (safe) по RFC 9110.",
                            "Какие методы меняют состояние сервера, но при повторе дают тот же итоговый результат? Отметьте все.",
                        ],
                    )
                };
                let options = chosen
                    .iter()
                    .map(|m| (m.name.to_owned(), is_target(m)))
                    .collect();
                return Draft::multi(stem, options);
            }
        }
    }
}

#[derive(Clone, Debug)]
enum GitOp {
    Commit(&'static str),
    SwitchCreate(&'static str),
    Switch(&'static str),
    Merge(&'static str),
    MergeNoFf(&'static str),
    MergeSquash(&'static str),
    ResetHard(usize),
    ResetSoft(usize),
    Revert,
    CherryPick(&'static str),
    Rebase(&'static str),
}

impl GitOp {
    fn render(&self) -> String {
        match self {
            GitOp::Commit(m) => format!("git commit -am \"{m}\""),
            GitOp::SwitchCreate(b) => format!("git switch -c {b}"),
            GitOp::Switch(b) => format!("git switch {b}"),
            GitOp::Merge(b) => format!("git merge {b}"),
            GitOp::MergeNoFf(b) => format!("git merge --no-ff {b}"),
            GitOp::MergeSquash(b) => format!("git merge --squash {b}"),
            GitOp::ResetHard(k) => format!("git reset --hard HEAD~{k}"),
            GitOp::ResetSoft(k) => format!("git reset --soft HEAD~{k}"),
            GitOp::Revert => "git revert --no-edit HEAD".to_owned(),
            GitOp::CherryPick(b) => format!("git cherry-pick {b}"),
            GitOp::Rebase(b) => format!("git rebase {b}"),
        }
    }
}

struct Repo {
    parents: Vec<Vec<usize>>,
    branches: Vec<(&'static str, usize)>,
    head: &'static str,
}

impl Repo {
    fn new(initial: usize) -> Self {
        let mut parents = vec![Vec::new()];
        for i in 1..initial {
            parents.push(vec![i - 1]);
        }
        Self {
            parents,
            branches: vec![("main", initial - 1)],
            head: "main",
        }
    }

    fn tip(&self, branch: &str) -> usize {
        self.branches
            .iter()
            .find(|(b, _)| *b == branch)
            .map_or(0, |(_, c)| *c)
    }

    fn set_tip(&mut self, branch: &'static str, commit: usize) {
        match self.branches.iter_mut().find(|(b, _)| *b == branch) {
            Some(entry) => entry.1 = commit,
            None => self.branches.push((branch, commit)),
        }
    }

    fn reachable(&self, from: usize) -> Vec<bool> {
        let mut seen = vec![false; self.parents.len()];
        let mut stack = vec![from];
        while let Some(c) = stack.pop() {
            if !seen[c] {
                seen[c] = true;
                stack.extend(self.parents[c].iter().copied());
            }
        }
        seen
    }

    fn count(&self, branch: &str) -> usize {
        self.reachable(self.tip(branch))
            .iter()
            .filter(|s| **s)
            .count()
    }

    fn is_ancestor(&self, a: usize, b: usize) -> bool {
        self.reachable(b)[a]
    }

    fn new_commit(&mut self, parents: Vec<usize>) -> usize {
        self.parents.push(parents);
        self.parents.len() - 1
    }

    fn commit_on_head(&mut self) {
        let tip = self.tip(self.head);
        let c = self.new_commit(vec![tip]);
        self.set_tip(self.head, c);
    }

    fn first_parent_depth(&self, mut c: usize) -> usize {
        let mut depth = 0;
        while let Some(&p) = self.parents[c].first() {
            c = p;
            depth += 1;
        }
        depth
    }

    fn apply(&mut self, op: &GitOp) {
        match op {
            GitOp::Commit(_) | GitOp::Revert => self.commit_on_head(),
            GitOp::SwitchCreate(b) => {
                let tip = self.tip(self.head);
                self.set_tip(b, tip);
                self.head = b;
            }
            GitOp::Switch(b) => self.head = b,
            GitOp::Merge(b) | GitOp::MergeNoFf(b) => {
                let (ours, theirs) = (self.tip(self.head), self.tip(b));
                if self.is_ancestor(theirs, ours) {
                    return;
                }
                if matches!(op, GitOp::Merge(_)) && self.is_ancestor(ours, theirs) {
                    self.set_tip(self.head, theirs);
                } else {
                    let c = self.new_commit(vec![ours, theirs]);
                    self.set_tip(self.head, c);
                }
            }
            GitOp::MergeSquash(_) => {}
            GitOp::ResetHard(k) | GitOp::ResetSoft(k) => {
                let mut c = self.tip(self.head);
                for _ in 0..*k {
                    c = self.parents[c].first().copied().unwrap_or(c);
                }
                self.set_tip(self.head, c);
            }
            GitOp::CherryPick(_) => self.commit_on_head(),
            GitOp::Rebase(onto) => {
                let base = self.reachable(self.tip(onto));
                let mut replay = Vec::new();
                let mut c = self.tip(self.head);
                while !base[c] {
                    replay.push(c);
                    match self.parents[c].first() {
                        Some(&p) => c = p,
                        None => break,
                    }
                }
                let mut tip = self.tip(onto);
                for _ in replay.iter().rev() {
                    tip = self.new_commit(vec![tip]);
                }
                self.set_tip(self.head, tip);
            }
        }
    }
}

const COMMIT_MESSAGES: &[&str] = &[
    "feat: add login form",
    "fix: handle empty cart",
    "refactor: extract service",
    "docs: update readme",
    "test: cover parser",
    "chore: bump deps",
    "feat: export to csv",
    "fix: off-by-one in pager",
];

fn git_history(level: u8, rng: &mut Rng) -> Draft {
    let initial = range(rng, 2, 4) as usize;
    let feature = *pick(rng, &["feature", "feat/auth", "bugfix", "topic"]);
    let mut msg = {
        let pool = shuffled(rng, COMMIT_MESSAGES.to_vec());
        let mut i = 0;
        move || {
            i += 1;
            pool[(i - 1) % pool.len()]
        }
    };
    let mut ops: Vec<GitOp> = Vec::new();
    let mut ask = "main";
    match level {
        1 => {
            ops.push(GitOp::SwitchCreate(feature));
            for _ in 0..range(rng, 1, 3) {
                ops.push(GitOp::Commit(msg()));
            }
            ops.push(GitOp::Switch("main"));
            for _ in 0..range(rng, 0, 2) {
                ops.push(GitOp::Commit(msg()));
            }
            ops.push(GitOp::Merge(feature));
            if chance(rng, 0.4) {
                ops.push(GitOp::Commit(msg()));
            }
        }
        2 => {
            let made = range(rng, 2, 3) as usize;
            for _ in 0..made {
                ops.push(GitOp::Commit(msg()));
            }
            match range(rng, 0, 2) {
                0 => {
                    ops.push(GitOp::ResetHard(range(rng, 1, made as i64) as usize));
                    ops.push(GitOp::Commit(msg()));
                }
                1 => {
                    ops.push(GitOp::ResetSoft(range(rng, 2, made as i64) as usize));
                    ops.push(GitOp::Commit(msg()));
                }
                _ => {
                    ops.push(GitOp::Revert);
                    if chance(rng, 0.5) {
                        ops.push(GitOp::ResetHard(1));
                    }
                }
            }
            ops.push(GitOp::SwitchCreate(feature));
            for _ in 0..range(rng, 1, 2) {
                ops.push(GitOp::Commit(msg()));
            }
            if chance(rng, 0.5) {
                ops.push(GitOp::Revert);
            }
            ops.push(GitOp::Switch("main"));
            ops.push(GitOp::Merge(feature));
        }
        _ => {
            ops.push(GitOp::SwitchCreate(feature));
            for _ in 0..range(rng, 2, 3) {
                ops.push(GitOp::Commit(msg()));
            }
            ops.push(GitOp::Switch("main"));
            match range(rng, 0, 3) {
                0 => {
                    ops.push(GitOp::MergeNoFf(feature));
                }
                1 => {
                    for _ in 0..range(rng, 0, 1) {
                        ops.push(GitOp::Commit(msg()));
                    }
                    ops.push(GitOp::MergeSquash(feature));
                    ops.push(GitOp::Commit(msg()));
                }
                2 => {
                    ops.push(GitOp::Commit(msg()));
                    ops.push(GitOp::CherryPick(feature));
                    if chance(rng, 0.5) {
                        ops.push(GitOp::ResetHard(1));
                    }
                }
                _ => {
                    for _ in 0..range(rng, 1, 2) {
                        ops.push(GitOp::Commit(msg()));
                    }
                    ops.push(GitOp::Switch(feature));
                    ops.push(GitOp::Rebase("main"));
                    if chance(rng, 0.5) {
                        ask = feature;
                    } else {
                        ops.push(GitOp::Switch("main"));
                        ops.push(GitOp::Merge(feature));
                    }
                }
            }
        }
    }

    let mut repo = Repo::new(initial);
    for op in &ops {
        if let GitOp::ResetHard(k) | GitOp::ResetSoft(k) = op {
            debug_assert!(*k <= repo.first_parent_depth(repo.tip(repo.head)));
        }
        repo.apply(op);
    }
    let answer = repo.count(ask) as i64;
    let script = ops.iter().map(GitOp::render).collect::<Vec<_>>().join("\n");
    Draft::integer(
        format!(
            "В репозитории есть ветка `main` с линейной историей из {initial} коммитов, рабочая копия чистая, вы на `main`. \
             Выполнили команды ниже; каждый `git commit` фиксирует новые изменения, конфликтов нет.\n\n\
             Что выведет `git rev-list --count {ask}`?"
        ),
        answer,
    )
    .code("bash", script)
}

struct LogLine {
    ip: &'static str,
    method: &'static str,
    path: &'static str,
    status: u16,
}

const IPS: &[&str] = &[
    "10.0.0.5",
    "10.0.0.17",
    "10.0.1.8",
    "192.168.1.20",
    "192.168.1.33",
    "172.16.4.2",
];
const LOG_PATHS: &[&str] = &[
    "/",
    "/login",
    "/api/users",
    "/api/orders",
    "/static/app.js",
    "/health",
    "/admin",
];

fn log_lines(rng: &mut Rng, n: usize, ips: &[&'static str]) -> Vec<LogLine> {
    (0..n)
        .map(|_| {
            let method = if chance(rng, 0.3) { "POST" } else { "GET" };
            let status = *pick(rng, &[200, 200, 200, 201, 301, 404, 404, 500, 503]);
            LogLine {
                ip: pick(rng, ips),
                method,
                path: pick(rng, LOG_PATHS),
                status,
            }
        })
        .collect()
}

fn render_log(lines: &[LogLine]) -> String {
    lines
        .iter()
        .map(|l| format!("{} {} {} {}", l.ip, l.method, l.path, l.status))
        .collect::<Vec<_>>()
        .join("\n")
}

fn unique_max<'a>(
    values: impl Iterator<Item = &'a str>,
) -> Option<(&'a str, Vec<(&'a str, usize)>)> {
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for v in values {
        match counts.iter_mut().find(|(k, _)| *k == v) {
            Some(e) => e.1 += 1,
            None => counts.push((v, 1)),
        }
    }
    let max = counts.iter().map(|c| c.1).max()?;
    let winners: Vec<&str> = counts.iter().filter(|c| c.1 == max).map(|c| c.0).collect();
    (winners.len() == 1).then(|| (winners[0], counts))
}

fn log_pipeline(level: u8, rng: &mut Rng) -> Draft {
    let ips = pick_n(rng, IPS, if level == 2 { 5 } else { 4 });
    for _ in 0..64 {
        let count = range(rng, 9, 12) as usize;
        let lines = log_lines(rng, count, &ips);
        let (command, draft) = match level {
            1 => match range(rng, 0, 2) {
                0 => {
                    let status = *pick(rng, &[404u16, 500, 200]);
                    let n = lines.iter().filter(|l| l.status == status).count() as i64;
                    (
                        format!("grep -c ' {status}$' access.log"),
                        Draft::integer("Что выведет команда?", n),
                    )
                }
                1 => {
                    let n = lines.iter().filter(|l| l.method == "POST").count() as i64;
                    (
                        "grep POST access.log | wc -l".to_owned(),
                        Draft::integer("Что выведет команда?", n),
                    )
                }
                _ => {
                    let n = lines.iter().filter(|l| l.status != 200).count() as i64;
                    (
                        "grep -v ' 200$' access.log | wc -l".to_owned(),
                        Draft::integer("Что выведет команда?", n),
                    )
                }
            },
            2 => {
                let by_ip = chance(rng, 0.5);
                let column = if by_ip { 1 } else { 3 };
                let values: Vec<&str> = lines
                    .iter()
                    .map(|l| if by_ip { l.ip } else { l.path })
                    .collect();
                let Some((winner, counts)) = unique_max(values.iter().copied()) else {
                    continue;
                };
                if counts.len() < 4 {
                    continue;
                }
                let first = values[0];
                let last = values[values.len() - 1];
                let mut distractors: Vec<String> = vec![first.to_owned(), last.to_owned()];
                let mut rest: Vec<(&str, usize)> = counts.clone();
                rest.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(a.0)));
                distractors.extend(rest.iter().map(|c| c.0.to_owned()));
                let what = if by_ip { "IP-адрес" } else { "путь" };
                (
                    format!(
                        "awk '{{print ${column}}}' access.log | sort | uniq -c | sort -rn | head -1"
                    ),
                    Draft::choice(
                        format!("Какой {what} окажется в единственной строке вывода?"),
                        winner,
                        distractors.into_iter().filter(|d| d != winner),
                    ),
                )
            }
            _ => {
                if chance(rng, 0.5) {
                    let threshold = *pick(rng, &[400u16, 500]);
                    let mut hit: Vec<&str> = lines
                        .iter()
                        .filter(|l| l.status >= threshold)
                        .map(|l| l.ip)
                        .collect();
                    hit.sort_unstable();
                    hit.dedup();
                    (
                        format!(
                            "awk '$4 >= {threshold} {{print $1}}' access.log | sort -u | wc -l"
                        ),
                        Draft::integer("Что выведет команда?", hit.len() as i64),
                    )
                } else {
                    let mut hit: Vec<&str> = lines
                        .iter()
                        .filter(|l| l.status >= 400)
                        .map(|l| l.path)
                        .collect();
                    hit.sort_unstable();
                    hit.dedup();
                    (
                        "awk '$4 >= 400' access.log | cut -d' ' -f3 | sort | uniq | wc -l"
                            .to_owned(),
                        Draft::integer("Что выведет команда?", hit.len() as i64),
                    )
                }
            }
        };
        let log = render_log(&lines);
        return draft
            .context(format!(
                "Файл `access.log` (поля: IP, метод, путь, статус):\n\n```text\n{log}\n```"
            ))
            .code("bash", command);
    }
    Draft::choice(
        "Какой IP-адрес окажется в выводе `awk '{print $1}' access.log | sort | uniq -c | sort -rn | head -1`, если в логе 3 строки от 10.0.0.5 и по одной от 10.0.0.17, 10.0.1.8 и 172.16.4.2?",
        "10.0.0.5",
        ["10.0.0.17", "10.0.1.8", "172.16.4.2"].map(str::to_owned),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_rendering() {
        assert_eq!(Complexity::new(0, 0).render(), "O(1)");
        assert_eq!(Complexity::new(1, 1).render(), "O(n log n)");
        assert_eq!(Complexity::new(2, 0).render(), "O(n²)");
        assert_eq!(Complexity::new(0, 2).render(), "O(log² n)");
        assert_eq!(
            block_complexity(&[Loop::Linear, Loop::HalvingDependent]),
            Complexity::new(1, 1)
        );
    }

    #[test]
    fn binary_search_variants() {
        let arr = [1, 3, 5, 7, 9, 11];
        assert_eq!(
            run_search(CORRECT_SEARCH, &arr, 7).0,
            SearchOutcome::Found(3)
        );
        assert_eq!(
            run_search(CORRECT_SEARCH, &arr, 4).0,
            SearchOutcome::Found(-1)
        );
        let strict = SearchVariant {
            strict_loop: true,
            ..CORRECT_SEARCH
        };
        assert_eq!(run_search(strict, &arr, 1).0, SearchOutcome::Found(0));
        assert_eq!(run_search(strict, &arr, 3).0, SearchOutcome::Found(-1));
        let lo_mid = SearchVariant {
            lo_to_mid: true,
            ..CORRECT_SEARCH
        };
        assert_eq!(run_search(lo_mid, &arr, 12).0, SearchOutcome::Infinite);
        let hi_len = SearchVariant {
            hi_is_len: true,
            ..CORRECT_SEARCH
        };
        assert_eq!(run_search(hi_len, &arr, 12).0, SearchOutcome::IndexError);
        assert_eq!(
            run_search(CORRECT_SEARCH, &[1, 3, 5, 7, 9, 11, 13], 13).1,
            3
        );
    }

    #[test]
    fn containers() {
        use ContainerOp::*;
        let ops = [Push(4), Push(9), Pop, Push(2), Pop];
        assert_eq!(run_container(&ops, Semantics::Stack), (vec![9, 2], vec![4]));
        assert_eq!(run_container(&ops, Semantics::Fifo), (vec![4, 9], vec![2]));
        let heap = [Push(5), Push(1), Push(8), Pop, Push(0), Pop, Pop];
        assert_eq!(run_container(&heap, Semantics::MinHeap).0, vec![1, 0, 5]);
        let dq = [Push(1), PushLeft(2), Push(3), PopLeft, Pop];
        assert_eq!(run_container(&dq, Semantics::Deque), (vec![2, 3], vec![1]));
    }

    #[test]
    fn git_simulation() {
        let mut repo = Repo::new(3);
        for op in [
            GitOp::SwitchCreate("feature"),
            GitOp::Commit("a"),
            GitOp::Commit("b"),
            GitOp::Switch("main"),
            GitOp::Merge("feature"),
        ] {
            repo.apply(&op);
        }
        assert_eq!(repo.count("main"), 5);

        let mut repo = Repo::new(3);
        for op in [
            GitOp::SwitchCreate("feature"),
            GitOp::Commit("a"),
            GitOp::Switch("main"),
            GitOp::Commit("b"),
            GitOp::Merge("feature"),
        ] {
            repo.apply(&op);
        }
        assert_eq!(repo.count("main"), 6);

        let mut repo = Repo::new(2);
        for op in [
            GitOp::SwitchCreate("feature"),
            GitOp::Commit("a"),
            GitOp::Commit("b"),
            GitOp::Switch("main"),
            GitOp::Commit("c"),
            GitOp::Switch("feature"),
            GitOp::Rebase("main"),
        ] {
            repo.apply(&op);
        }
        assert_eq!(repo.count("feature"), 5);
        assert_eq!(repo.count("main"), 3);

        let mut repo = Repo::new(2);
        for op in [
            GitOp::SwitchCreate("feature"),
            GitOp::Commit("a"),
            GitOp::Switch("main"),
            GitOp::MergeNoFf("feature"),
        ] {
            repo.apply(&op);
        }
        assert_eq!(repo.count("main"), 4);

        let mut repo = Repo::new(3);
        for op in [
            GitOp::Commit("a"),
            GitOp::Commit("b"),
            GitOp::ResetSoft(2),
            GitOp::Commit("s"),
        ] {
            repo.apply(&op);
        }
        assert_eq!(repo.count("main"), 4);
    }
}
