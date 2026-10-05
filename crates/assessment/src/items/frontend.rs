//! Frontend templates: JavaScript runtime semantics, CSS cascade and layout, React state.

use std::collections::VecDeque;

use catalog::Specialization::Frontend;

use super::util::{chance, join, perturb_sequence, pick, pick_n, range, shuffled};
use super::{C_CHOICE, C_OPEN, Draft, Level, Rng, Source, Template, Topic};

pub(crate) const TEMPLATES: &[Template] = &[
    Template {
        id: "js.event_loop_order",
        title: "Порядок вывода в event loop",
        topic: Topic::Javascript,
        specializations: &[Frontend],
        skills: &["javascript", "nodejs"],
        levels: &[
            Level::new(1, 1.1, -0.6, C_CHOICE, 90),
            Level::new(2, 1.4, 0.3, C_CHOICE, 150),
            Level::new(3, 1.6, 1.2, C_CHOICE, 180),
        ],
        source: Source::Computed(event_loop),
    },
    Template {
        id: "js.closure_loop",
        title: "Замыкания в циклах: var и let",
        topic: Topic::Javascript,
        specializations: &[Frontend],
        skills: &["javascript"],
        levels: &[
            Level::new(1, 1.0, -0.8, C_CHOICE, 90),
            Level::new(2, 1.3, 0.0, C_CHOICE, 120),
            Level::new(3, 1.5, 0.8, C_CHOICE, 150),
        ],
        source: Source::Computed(closure_loop),
    },
    Template {
        id: "js.type_coercion",
        title: "Приведение типов в JavaScript",
        topic: Topic::Javascript,
        specializations: &[Frontend],
        skills: &["javascript"],
        levels: &[
            Level::new(1, 0.9, -1.0, C_CHOICE, 60),
            Level::new(2, 1.2, -0.2, C_CHOICE, 90),
            Level::new(3, 1.4, 0.7, C_CHOICE, 90),
        ],
        source: Source::Computed(type_coercion),
    },
    Template {
        id: "js.array_methods",
        title: "Методы массивов",
        topic: Topic::Javascript,
        specializations: &[Frontend],
        skills: &["javascript", "typescript"],
        levels: &[
            Level::new(1, 1.0, -1.2, C_CHOICE, 90),
            Level::new(2, 1.2, -0.4, C_OPEN, 120),
            Level::new(3, 1.4, 0.5, C_CHOICE, 90),
        ],
        source: Source::Computed(array_methods),
    },
    Template {
        id: "css.specificity",
        title: "Специфичность селекторов CSS",
        topic: Topic::Layout,
        specializations: &[Frontend],
        skills: &["css", "html"],
        levels: &[
            Level::new(1, 1.1, -0.9, C_CHOICE, 90),
            Level::new(2, 1.3, 0.1, C_CHOICE, 120),
            Level::new(3, 1.5, 1.0, C_CHOICE, 150),
        ],
        source: Source::Computed(specificity),
    },
    Template {
        id: "css.box_model",
        title: "Блочная модель CSS",
        topic: Topic::Layout,
        specializations: &[Frontend],
        skills: &["css"],
        levels: &[
            Level::new(1, 1.0, -1.1, C_OPEN, 90),
            Level::new(2, 1.2, -0.3, C_OPEN, 90),
            Level::new(3, 1.4, 0.6, C_OPEN, 120),
        ],
        source: Source::Computed(box_model),
    },
    Template {
        id: "react.state_batching",
        title: "Пакетные обновления состояния React",
        topic: Topic::Frameworks,
        specializations: &[Frontend],
        skills: &["react", "javascript"],
        levels: &[
            Level::new(1, 1.1, -0.3, C_OPEN, 90),
            Level::new(2, 1.4, 0.5, C_OPEN, 120),
            Level::new(3, 1.5, 1.2, C_OPEN, 150),
        ],
        source: Source::Computed(state_batching),
    },
];

#[derive(Clone, Debug)]
enum Js {
    Log(usize),
    Timeout { delay: u32, body: Vec<Js> },
    Micro(Vec<Js>),
    Then(Vec<Vec<Js>>),
    Async { before: Vec<Js>, after: Vec<Js> },
    NewPromise { executor: Vec<Js>, then: Vec<Js> },
}

#[derive(Clone, Copy, Default)]
struct LoopModel {
    macro_first: bool,
    inline_callbacks: bool,
    await_blocks: bool,
    chain_eager: bool,
    executor_async: bool,
}

enum Task {
    Body(Vec<Js>),
    Chain(Vec<Vec<Js>>, usize),
}

struct EventLoop<'a> {
    labels: &'a [char],
    model: LoopModel,
    out: String,
    micro: VecDeque<Task>,
    timers: Vec<(u32, u32, Vec<Js>)>,
    seq: u32,
    now: u32,
}

impl EventLoop<'_> {
    fn exec(&mut self, stmts: &[Js]) {
        let inline = self.model.inline_callbacks;
        for s in stmts {
            match s {
                Js::Log(i) => self.out.push(self.labels[*i]),
                Js::Timeout { delay, body } => {
                    if inline {
                        self.exec(body);
                    } else {
                        self.timers.push((self.now + delay, self.seq, body.clone()));
                        self.seq += 1;
                    }
                }
                Js::Micro(body) => {
                    if inline {
                        self.exec(body);
                    } else {
                        self.micro.push_back(Task::Body(body.clone()));
                    }
                }
                Js::Then(steps) => {
                    if inline {
                        for step in steps {
                            self.exec(step);
                        }
                    } else if self.model.chain_eager {
                        self.micro.extend(steps.iter().cloned().map(Task::Body));
                    } else {
                        self.micro.push_back(Task::Chain(steps.clone(), 0));
                    }
                }
                Js::Async { before, after } => {
                    self.exec(before);
                    if inline || self.model.await_blocks {
                        self.exec(after);
                    } else {
                        self.micro.push_back(Task::Body(after.clone()));
                    }
                }
                Js::NewPromise { executor, then } => {
                    if self.model.executor_async && !inline {
                        let mut body = executor.clone();
                        body.extend(then.iter().cloned());
                        self.micro.push_back(Task::Body(body));
                    } else {
                        self.exec(executor);
                        if inline {
                            self.exec(then);
                        } else {
                            self.micro.push_back(Task::Body(then.clone()));
                        }
                    }
                }
            }
        }
    }

    fn drain_micro(&mut self) {
        while let Some(task) = self.micro.pop_front() {
            match task {
                Task::Body(body) => self.exec(&body),
                Task::Chain(steps, i) => {
                    self.exec(&steps[i]);
                    if i + 1 < steps.len() {
                        self.micro.push_back(Task::Chain(steps, i + 1));
                    }
                }
            }
        }
    }

    fn next_timer(&mut self) -> Option<(u32, Vec<Js>)> {
        let (i, _) = self
            .timers
            .iter()
            .enumerate()
            .min_by_key(|(_, t)| (t.0, t.1))?;
        let (at, _, body) = self.timers.remove(i);
        Some((at, body))
    }

    fn run(labels: &[char], program: &[Js], model: LoopModel) -> String {
        let mut el = EventLoop {
            labels,
            model,
            out: String::new(),
            micro: VecDeque::new(),
            timers: Vec::new(),
            seq: 0,
            now: 0,
        };
        el.exec(program);
        if model.macro_first {
            while !el.timers.is_empty() || !el.micro.is_empty() {
                while let Some((at, body)) = el.next_timer() {
                    el.now = at;
                    el.exec(&body);
                }
                el.drain_micro();
            }
        } else {
            el.drain_micro();
            while let Some((at, body)) = el.next_timer() {
                el.now = at;
                el.exec(&body);
                el.drain_micro();
            }
        }
        el.out
    }
}

fn render_js(stmts: &[Js], indent: usize, labels: &[char], out: &mut String) {
    let pad = "  ".repeat(indent);
    for s in stmts {
        match s {
            Js::Log(i) => out.push_str(&format!("{pad}console.log('{}');\n", labels[*i])),
            Js::Timeout { delay, body } => {
                out.push_str(&format!("{pad}setTimeout(() => {{\n"));
                render_js(body, indent + 1, labels, out);
                out.push_str(&format!("{pad}}}, {delay});\n"));
            }
            Js::Micro(body) => {
                out.push_str(&format!("{pad}queueMicrotask(() => {{\n"));
                render_js(body, indent + 1, labels, out);
                out.push_str(&format!("{pad}}});\n"));
            }
            Js::Then(steps) => {
                out.push_str(&format!("{pad}Promise.resolve()"));
                for step in steps {
                    out.push_str(".then(() => {\n");
                    render_js(step, indent + 1, labels, out);
                    out.push_str(&format!("{pad}}})"));
                }
                out.push_str(";\n");
            }
            Js::Async { before, after } => {
                out.push_str(&format!("{pad}(async () => {{\n"));
                render_js(before, indent + 1, labels, out);
                out.push_str(&format!("{pad}  await null;\n"));
                render_js(after, indent + 1, labels, out);
                out.push_str(&format!("{pad}}})();\n"));
            }
            Js::NewPromise { executor, then } => {
                out.push_str(&format!("{pad}new Promise((resolve) => {{\n"));
                render_js(executor, indent + 1, labels, out);
                out.push_str(&format!("{pad}  resolve();\n{pad}}}).then(() => {{\n"));
                render_js(then, indent + 1, labels, out);
                out.push_str(&format!("{pad}}});\n"));
            }
        }
    }
}

struct Labeler(usize);

impl Labeler {
    fn log(&mut self) -> Js {
        self.0 += 1;
        Js::Log(self.0 - 1)
    }
}

fn event_loop_blocks(level: u8, rng: &mut Rng, l: &mut Labeler) -> Vec<Js> {
    let timeout = |l: &mut Labeler| Js::Timeout {
        delay: 0,
        body: vec![l.log()],
    };
    let mut blocks = vec![l.log()];
    match level {
        1 => {
            blocks.push(l.log());
            blocks.push(timeout(l));
            blocks.push(if chance(rng, 0.5) {
                Js::Micro(vec![l.log()])
            } else {
                Js::Then(vec![vec![l.log()]])
            });
            if chance(rng, 0.5) {
                blocks.push(l.log());
            }
        }
        2 => {
            blocks.push(Js::NewPromise {
                executor: vec![l.log()],
                then: vec![l.log()],
            });
            blocks.push(if chance(rng, 0.5) {
                Js::Then(vec![vec![l.log()], vec![l.log()]])
            } else {
                let outer = l.log();
                Js::Micro(vec![outer, Js::Micro(vec![l.log()])])
            });
            blocks.push(if chance(rng, 0.5) {
                let first = l.log();
                Js::Timeout {
                    delay: 0,
                    body: vec![first, Js::Micro(vec![l.log()])],
                }
            } else {
                timeout(l)
            });
        }
        _ => {
            let before = l.log();
            let after = if chance(rng, 0.5) {
                vec![l.log()]
            } else {
                let a = l.log();
                vec![
                    a,
                    Js::Timeout {
                        delay: 0,
                        body: vec![l.log()],
                    },
                ]
            };
            blocks.push(Js::Async {
                before: vec![before],
                after,
            });
            blocks.push(if chance(rng, 0.5) {
                let first = l.log();
                Js::Micro(vec![
                    first,
                    Js::Timeout {
                        delay: 0,
                        body: vec![l.log()],
                    },
                ])
            } else {
                let first = l.log();
                Js::Then(vec![
                    vec![
                        first,
                        Js::Timeout {
                            delay: 0,
                            body: vec![l.log()],
                        },
                    ],
                    vec![l.log()],
                ])
            });
            if chance(rng, 0.5) {
                blocks.push(Js::Timeout {
                    delay: 100,
                    body: vec![l.log()],
                });
            } else {
                blocks.push(Js::Then(vec![vec![l.log()], vec![l.log()]]));
            }
        }
    }
    blocks
}

fn spaced(s: &str) -> String {
    join(&s.chars().collect::<Vec<_>>(), " ")
}

fn event_loop(level: u8, rng: &mut Rng) -> Draft {
    let mut labeler = Labeler(0);
    let blocks = event_loop_blocks(level, rng, &mut labeler);
    let program = shuffled(rng, blocks);
    let first = range(rng, 0, 26 - labeler.0 as i64) as u8;
    let labels: Vec<char> = shuffled(
        rng,
        (0..labeler.0 as u8)
            .map(|i| char::from(b'A' + first + i))
            .collect(),
    );

    let answer = EventLoop::run(&labels, &program, LoopModel::default());
    let models = [
        LoopModel {
            macro_first: true,
            ..LoopModel::default()
        },
        LoopModel {
            inline_callbacks: true,
            ..LoopModel::default()
        },
        LoopModel {
            await_blocks: true,
            ..LoopModel::default()
        },
        LoopModel {
            chain_eager: true,
            ..LoopModel::default()
        },
        LoopModel {
            executor_async: true,
            ..LoopModel::default()
        },
        LoopModel {
            macro_first: true,
            await_blocks: true,
            ..LoopModel::default()
        },
    ];
    let mut distractors: Vec<String> = models
        .iter()
        .map(|m| EventLoop::run(&labels, &program, *m))
        .collect();
    distractors.extend(perturb_sequence(&answer));

    let mut code = String::new();
    render_js(&program, 0, &labels, &mut code);
    Draft::choice(
        "В каком порядке будут выведены буквы? Код выполняется в браузере или Node.js как обычный скрипт.",
        spaced(&answer),
        distractors.iter().map(|d| spaced(d)),
    )
    .code("javascript", code)
}

#[derive(Clone, Copy, PartialEq)]
enum Decl {
    Var,
    Let,
    LetOutside,
}

#[derive(Clone, Copy, PartialEq)]
enum Captured {
    I,
    J,
    IPlusJ,
}

struct ClosureCase {
    start: i64,
    step: i64,
    iterations: i64,
    factor: i64,
    captured: Captured,
}

impl ClosureCase {
    fn outputs(&self, decl: Decl, end_value: i64) -> Vec<i64> {
        (0..self.iterations)
            .map(|t| {
                let v = self.start + t * self.step;
                let i = if decl == Decl::Let { v } else { end_value };
                let j = v * self.factor;
                match self.captured {
                    Captured::I => i,
                    Captured::J => j,
                    Captured::IPlusJ => i + j,
                }
            })
            .collect()
    }

    fn end(&self) -> i64 {
        self.start + self.iterations * self.step
    }
}

const LOOP_VARS: &[&str] = &["i", "k", "n", "idx", "x", "pos", "cur", "num"];
const INNER_VARS: &[&str] = &["j", "v", "y", "val", "res", "m", "z", "w"];
const FN_LISTS: &[&str] = &[
    "fns",
    "callbacks",
    "getters",
    "handlers",
    "tasks",
    "thunks",
    "readers",
];

/// Equal delays keep timers in registration order, so the delay value is an incidental.
fn timeout_log(rng: &mut Rng, expr: &str) -> String {
    let delay = *pick(rng, &[0, 0, 1, 10, 50, 100]);
    let tail = if delay == 0 && chance(rng, 0.3) {
        String::new()
    } else {
        format!(", {delay}")
    };
    match range(rng, 0, 2) {
        0 => format!("  setTimeout(() => console.log({expr}){tail});\n"),
        1 => format!("  setTimeout(() => {{\n    console.log({expr});\n  }}{tail});\n"),
        _ => format!("  setTimeout(function () {{\n    console.log({expr});\n  }}{tail});\n"),
    }
}

fn closure_loop(level: u8, rng: &mut Rng) -> Draft {
    let iterations = range(rng, 2, 4);
    let (start, step) = match level {
        1 => (range(rng, 0, 5), 1),
        _ => (range(rng, 0, 2), range(rng, 1, 3)),
    };
    let last = start + (iterations - 1) * step;
    let limit = last + if step > 1 { range(rng, 1, step) } else { 1 };
    let decl = match level {
        1 => *pick(rng, &[Decl::Var, Decl::Let]),
        2 => *pick(rng, &[Decl::Var, Decl::Let, Decl::LetOutside]),
        _ => *pick(rng, &[Decl::Var, Decl::LetOutside]),
    };
    let captured = match level {
        1 => Captured::I,
        2 => *pick(rng, &[Captured::I, Captured::J]),
        _ => Captured::IPlusJ,
    };
    let case = ClosureCase {
        start,
        step,
        iterations,
        factor: range(rng, 2, 10),
        captured,
    };
    let fmt = |v: Vec<i64>| join(&v, " ");
    let answer = fmt(case.outputs(decl, case.end()));
    let other_decl = if decl == Decl::Let {
        Decl::Var
    } else {
        Decl::Let
    };
    let mut distractors = vec![
        fmt(case.outputs(other_decl, case.end())),
        fmt(case.outputs(decl, last)),
        fmt(case.outputs(decl, limit)),
        fmt(case.outputs(other_decl, last)),
        fmt(vec![case.end(); iterations as usize]),
        fmt(case
            .outputs(Decl::Let, case.end())
            .into_iter()
            .rev()
            .collect()),
    ];
    distractors.push(fmt(vec![last; iterations as usize]));
    distractors.push(fmt(vec![last * case.factor; iterations as usize]));
    distractors.push(fmt(vec![case.end() * case.factor; iterations as usize]));
    distractors.push(fmt((0..iterations).collect()));
    distractors.push(fmt((1..=iterations).collect()));

    let i = *pick(rng, LOOP_VARS);
    let j = *pick(rng, INNER_VARS);
    let increment = if step == 1 {
        format!("{i}++")
    } else {
        format!("{i} += {step}")
    };
    let header = match decl {
        Decl::Var => format!("for (var {i} = {start}; {i} < {limit}; {increment}) {{\n"),
        Decl::Let => format!("for (let {i} = {start}; {i} < {limit}; {increment}) {{\n"),
        Decl::LetOutside => {
            format!("let {i};\nfor ({i} = {start}; {i} < {limit}; {increment}) {{\n")
        }
    };
    let factor = case.factor;
    let inner_decl = format!(
        "  {} {j} = {};\n",
        pick(rng, &["let", "const"]),
        if chance(rng, 0.5) {
            format!("{i} * {factor}")
        } else {
            format!("{factor} * {i}")
        }
    );
    let code = if level == 3 {
        let list = *pick(rng, FN_LISTS);
        let sum = if chance(rng, 0.5) {
            format!("{i} + {j}")
        } else {
            format!("{j} + {i}")
        };
        let print = match range(rng, 0, 2) {
            0 => format!("console.log({list}.map((f) => f()).join(' '));\n"),
            1 => format!("{list}.forEach((f) => console.log(f()));\n"),
            _ => format!("for (const f of {list}) {{\n  console.log(f());\n}}\n"),
        };
        format!("const {list} = [];\n{header}{inner_decl}  {list}.push(() => {sum});\n}}\n{print}")
    } else {
        let (inner, expr) = match captured {
            Captured::J => (inner_decl, j),
            _ => (String::new(), i),
        };
        let call = timeout_log(rng, expr);
        format!("{header}{inner}{call}}}\n")
    };
    let prompt = pick(
        rng,
        &[
            "Что будет выведено?",
            "Что выведет этот код в консоль?",
            "Какие числа появятся в консоли?",
        ],
    );
    Draft::choice(
        format!("{prompt} Значения перечислены через пробел в порядке вывода."),
        answer,
        distractors,
    )
    .code("javascript", code)
}

#[derive(Clone, Debug, PartialEq)]
enum JsVal {
    Num(Option<i64>),
    Str(String),
    Bool(bool),
    Null,
    Undef,
}

impl JsVal {
    fn to_number(&self) -> Option<i64> {
        match self {
            JsVal::Num(n) => *n,
            JsVal::Str(s) if s.is_empty() => Some(0),
            JsVal::Str(s) => s.parse().ok(),
            JsVal::Bool(b) => Some(i64::from(*b)),
            JsVal::Null => Some(0),
            JsVal::Undef => None,
        }
    }

    fn to_js_string(&self) -> String {
        match self {
            JsVal::Num(Some(n)) => n.to_string(),
            JsVal::Num(None) => "NaN".to_owned(),
            JsVal::Str(s) => s.clone(),
            JsVal::Bool(b) => b.to_string(),
            JsVal::Null => "null".to_owned(),
            JsVal::Undef => "undefined".to_owned(),
        }
    }

    fn literal(&self) -> String {
        match self {
            JsVal::Str(s) => format!("\"{s}\""),
            other => other.to_js_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
}

impl Op {
    fn symbol(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
        }
    }
}

#[derive(Clone, Copy, Default)]
struct CoercionModel {
    plus_is_numeric: bool,
    right_to_left: bool,
    null_is_nan: bool,
    no_precedence: bool,
}

fn js_binary(a: &JsVal, op: Op, b: &JsVal, m: CoercionModel) -> JsVal {
    let num = |v: &JsVal| match v {
        JsVal::Null if m.null_is_nan => None,
        other => other.to_number(),
    };
    let arith =
        |f: fn(i64, i64) -> Option<i64>| JsVal::Num(num(a).zip(num(b)).and_then(|(x, y)| f(x, y)));
    match op {
        Op::Add => {
            let stringy = matches!(a, JsVal::Str(_)) || matches!(b, JsVal::Str(_));
            // JS `+` is string concatenation whenever an operand is a string, unless a (buggy)
            // model forces numeric coercion and both operands actually coerce to a number.
            let concatenates =
                stringy && (!m.plus_is_numeric || num(a).is_none() || num(b).is_none());
            if concatenates {
                JsVal::Str(format!("{}{}", a.to_js_string(), b.to_js_string()))
            } else {
                arith(|x, y| x.checked_add(y))
            }
        }
        Op::Sub => arith(|x, y| x.checked_sub(y)),
        Op::Mul => arith(|x, y| x.checked_mul(y)),
    }
}

fn js_eval(values: &[JsVal], ops: &[Op], m: CoercionModel) -> JsVal {
    let mut vals: Vec<JsVal> = values.to_vec();
    let mut ops: Vec<Op> = ops.to_vec();
    if !m.no_precedence {
        while let Some(i) = ops.iter().position(|o| *o == Op::Mul) {
            let v = js_binary(&vals[i], Op::Mul, &vals[i + 1], m);
            vals.splice(i..=i + 1, [v]);
            ops.remove(i);
        }
    }
    if m.right_to_left {
        let mut acc = vals[vals.len() - 1].clone();
        for i in (0..ops.len()).rev() {
            acc = js_binary(&vals[i], ops[i], &acc, m);
        }
        acc
    } else {
        let mut acc = vals[0].clone();
        for (i, op) in ops.iter().enumerate() {
            acc = js_binary(&acc, *op, &vals[i + 1], m);
        }
        acc
    }
}

fn random_operand(level: u8, rng: &mut Rng) -> JsVal {
    let digit_string = |rng: &mut Rng| JsVal::Str(range(rng, 1, 9).to_string());
    match (level, range(rng, 0, 9)) {
        (3, 0) => JsVal::Bool(true),
        (3, 1) => JsVal::Bool(false),
        (3, 2) => JsVal::Null,
        (3, 3) => JsVal::Undef,
        (_, k) if k < 5 => digit_string(rng),
        _ => JsVal::Num(Some(range(rng, 1, 9))),
    }
}

const COERCION_VARS: &[&str] = &[
    "result", "value", "res", "out", "output", "x", "y", "z", "v", "r", "val", "tmp",
];

fn type_coercion(level: u8, rng: &mut Rng) -> Draft {
    for _ in 0..64 {
        let n = if level == 1 { 2 } else { 3 };
        let values: Vec<JsVal> = (0..n).map(|_| random_operand(level, rng)).collect();
        let op_pool: &[Op] = if level == 3 {
            &[Op::Add, Op::Sub, Op::Mul, Op::Add]
        } else {
            &[Op::Add, Op::Sub, Op::Add]
        };
        let ops: Vec<Op> = (0..n - 1).map(|_| *pick(rng, op_pool)).collect();
        let has_string = values.iter().any(|v| matches!(v, JsVal::Str(_)));
        let has_number = values.iter().any(|v| matches!(v, JsVal::Num(_)));
        if !has_string || (level < 3 && !has_number) || (level > 1 && !ops.contains(&Op::Add)) {
            continue;
        }
        if level == 3
            && !values
                .iter()
                .any(|v| matches!(v, JsVal::Bool(_) | JsVal::Null | JsVal::Undef))
        {
            continue;
        }
        let answer = js_eval(&values, &ops, CoercionModel::default()).literal();
        let models = [
            CoercionModel {
                plus_is_numeric: true,
                ..CoercionModel::default()
            },
            CoercionModel {
                right_to_left: true,
                ..CoercionModel::default()
            },
            CoercionModel {
                null_is_nan: true,
                ..CoercionModel::default()
            },
            CoercionModel {
                no_precedence: true,
                ..CoercionModel::default()
            },
        ];
        let mut distractors: Vec<String> = models
            .iter()
            .map(|m| js_eval(&values, &ops, *m).literal())
            .collect();
        let result = js_eval(&values, &ops, CoercionModel::default());
        match &result {
            JsVal::Str(s) => {
                distractors.push(s.clone());
                distractors.push("NaN".to_owned());
                distractors.push(format!("\"{}\"", s.chars().rev().collect::<String>()));
            }
            JsVal::Num(Some(v)) => {
                distractors.push(format!("\"{v}\""));
                distractors.push((v + 1).to_string());
                distractors.push("NaN".to_owned());
            }
            _ => {
                distractors.push("0".to_owned());
                distractors.push("\"NaN\"".to_owned());
                distractors.push("undefined".to_owned());
            }
        }
        let mut expr = values[0].literal();
        for (i, op) in ops.iter().enumerate() {
            expr.push_str(&format!(" {} {}", op.symbol(), values[i + 1].literal()));
        }
        let (question, code) = if chance(rng, 0.2) {
            let question = pick(
                rng,
                &[
                    "Чему равно значение выражения?",
                    "Каков результат вычисления выражения?",
                    "Что получится в результате вычисления выражения?",
                ],
            );
            ((*question).to_owned(), format!("{expr}\n"))
        } else {
            let name = *pick(rng, COERCION_VARS);
            let keyword = *pick(rng, &["const", "let"]);
            let question = match range(rng, 0, 2) {
                0 => format!("Чему будет равно значение `{name}`?"),
                1 => format!("Какое значение получит переменная `{name}`?"),
                _ => format!("Что окажется в переменной `{name}`?"),
            };
            (question, format!("{keyword} {name} = {expr};\n"))
        };
        return Draft::choice(
            format!("{question} Строковые значения в вариантах указаны в кавычках."),
            answer,
            distractors,
        )
        .code("javascript", code);
    }
    Draft::choice(
        "Чему равно значение выражения? Строковые значения в вариантах указаны в кавычках.",
        "\"53\"",
        ["8", "53", "NaN"].map(str::to_owned),
    )
    .code("javascript", "\"5\" + 3\n")
}

fn js_array(v: &[i64]) -> String {
    format!("[{}]", join(v, ","))
}

fn array_methods(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 => {
            let len = range(rng, 5, 6) as usize;
            let arr: Vec<i64> = pick_n(rng, &(1..=9).collect::<Vec<_>>(), len);
            let k = range(rng, 2, 3);
            let mapped: Vec<i64> = arr.iter().map(|x| x * k).collect();
            let mut sorted = mapped.clone();
            sorted.sort_unstable();
            let t = sorted[range(rng, 1, len as i64 - 2) as usize];
            let answer: Vec<i64> = mapped.iter().copied().filter(|x| *x > t).collect();
            let mut distractors = vec![
                js_array(
                    &arr.iter()
                        .copied()
                        .filter(|x| *x > t)
                        .map(|x| x * k)
                        .collect::<Vec<_>>(),
                ),
                js_array(
                    &mapped
                        .iter()
                        .copied()
                        .filter(|x| *x >= t)
                        .collect::<Vec<_>>(),
                ),
                js_array(
                    &mapped
                        .iter()
                        .copied()
                        .filter(|x| *x <= t)
                        .collect::<Vec<_>>(),
                ),
                js_array(
                    &arr.iter()
                        .copied()
                        .filter(|x| x * k > t)
                        .collect::<Vec<_>>(),
                ),
                js_array(&mapped),
                js_array(&answer.iter().rev().copied().collect::<Vec<_>>()),
            ];
            for skip in 0..answer.len() {
                let mut v = answer.clone();
                v.remove(skip);
                distractors.push(js_array(&v));
            }
            let code = format!(
                "const result = [{}]\n  .map((x) => x * {k})\n  .filter((x) => x > {t});\n\nconsole.log(JSON.stringify(result));\n",
                join(&arr, ", ")
            );
            Draft::choice("Что выведет код?", js_array(&answer), distractors)
                .code("javascript", code)
        }
        2 => {
            let arr: Vec<i64> = (0..range(rng, 5, 7)).map(|_| range(rng, 1, 12)).collect();
            let k = range(rng, 2, 4);
            let init = *pick(rng, &[0, 0, 10, 100]);
            let odd = chance(rng, 0.5);
            let (pred, keep): (&str, fn(i64) -> bool) = if odd {
                ("x % 2 === 1", |x| x % 2 == 1)
            } else {
                ("x % 2 === 0", |x| x % 2 == 0)
            };
            let total: i64 = init
                + arr
                    .iter()
                    .copied()
                    .filter(|x| keep(*x))
                    .map(|x| x * k)
                    .sum::<i64>();
            let code = format!(
                "const total = [{}]\n  .filter((x) => {pred})\n  .map((x) => x * {k})\n  .reduce((acc, x) => acc + x, {init});\n\nconsole.log(total);\n",
                join(&arr, ", ")
            );
            Draft::integer("Что выведет код?", total).code("javascript", code)
        }
        _ => {
            let pool = [1, 2, 3, 5, 7, 9, 10, 12, 21, 25, 30, 100, 101, 250];
            let (arr, numeric, lex) = loop {
                let arr = pick_n(rng, &pool, 5);
                let mut numeric = arr.clone();
                numeric.sort_unstable();
                let mut lex = arr.clone();
                lex.sort_by_key(|v| v.to_string());
                if lex != numeric && arr != lex {
                    break (arr, numeric, lex);
                }
            };
            let reverse = chance(rng, 0.4);
            let rev = |v: &[i64]| v.iter().rev().copied().collect::<Vec<_>>();
            let (answer, call) = if reverse {
                (rev(&lex), ".sort().reverse()")
            } else {
                (lex.clone(), ".sort()")
            };
            let distractors = vec![
                js_array(&if reverse {
                    rev(&numeric)
                } else {
                    numeric.clone()
                }),
                js_array(&if reverse {
                    numeric.clone()
                } else {
                    rev(&numeric)
                }),
                js_array(&arr),
                js_array(&if reverse { lex.clone() } else { rev(&lex) }),
                js_array(&rev(&arr)),
            ];
            let code = format!(
                "const values = [{}];\nconsole.log(JSON.stringify(values{call}));\n",
                join(&arr, ", ")
            );
            Draft::choice("Что выведет код?", js_array(&answer), distractors)
                .code("javascript", code)
        }
    }
}

type Spec = (u8, u8, u8);

fn spec_add(a: Spec, b: Spec) -> Spec {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

struct Names {
    root_id: &'static str,
    root_class: &'static str,
    card: &'static str,
    target_id: &'static str,
    text: &'static str,
    accent: &'static str,
    role: &'static str,
}

struct Rule {
    selector: String,
    spec: Spec,
    matches: bool,
    important: bool,
}

fn compounds(n: &Names, level: u8) -> Vec<(String, Spec, bool)> {
    let mut v = vec![
        ("p".to_owned(), (0, 0, 1), true),
        (format!(".{}", n.text), (0, 1, 0), true),
        (format!(".{}", n.accent), (0, 1, 0), true),
        (format!("p.{}", n.text), (0, 1, 1), true),
        (format!(".{}.{}", n.text, n.accent), (0, 2, 0), true),
        (format!("#{}", n.target_id), (1, 0, 0), true),
        (format!("p#{}", n.target_id), (1, 0, 1), true),
        (format!("[data-role=\"{}\"]", n.role), (0, 1, 0), true),
        ("p:first-child".to_owned(), (0, 1, 1), true),
    ];
    if level >= 2 {
        v.push((format!("p.{}.{}", n.accent, n.text), (0, 2, 1), true));
        v.push((format!(".{}:first-child", n.text), (0, 2, 0), true));
    }
    if level == 3 {
        v.extend([
            (":not(.muted)".to_owned(), (0, 1, 0), true),
            ("p:not(#outro)".to_owned(), (1, 0, 1), true),
            (
                format!(":is(#{}, .{})", n.target_id, n.text),
                (1, 0, 0),
                true,
            ),
            (
                format!(":where(#{}, .{})", n.target_id, n.text),
                (0, 0, 0),
                true,
            ),
            (format!("p:is(.{}, .other)", n.accent), (0, 1, 1), true),
            (format!(".{}:last-child", n.text), (0, 2, 0), false),
            (format!("p#{}.muted", n.target_id), (1, 1, 1), false),
        ]);
    }
    v
}

fn prefixes(n: &Names, level: u8) -> Vec<(String, Spec, bool)> {
    let mut v = vec![(String::new(), (0, 0, 0), true)];
    if level >= 2 {
        v.extend([
            ("section ".to_owned(), (0, 0, 1), true),
            (format!(".{} ", n.card), (0, 1, 0), true),
            ("main ".to_owned(), (0, 0, 1), true),
            (format!("#{} ", n.root_id), (1, 0, 0), true),
            (format!("main .{} ", n.card), (0, 1, 1), true),
            (format!("#{} section ", n.root_id), (1, 0, 1), true),
            (format!(".{} > ", n.card), (0, 1, 0), true),
            (format!(".{} .{} ", n.root_class, n.card), (0, 2, 0), true),
        ]);
    }
    if level == 3 {
        v.extend([
            (format!("#{} > ", n.root_id), (1, 0, 0), false),
            ("main > ".to_owned(), (0, 0, 1), false),
        ]);
    }
    v
}

fn winner(rules: &[Rule]) -> Option<usize> {
    rules
        .iter()
        .enumerate()
        .filter(|(_, r)| r.matches)
        .max_by_key(|(i, r)| (r.important, r.spec, *i))
        .map(|(i, _)| i)
}

fn specificity(level: u8, rng: &mut Rng) -> Draft {
    let names = Names {
        root_id: pick(rng, &["app", "root", "page"]),
        root_class: pick(rng, &["layout", "wrapper", "shell"]),
        card: pick(rng, &["card", "panel", "box"]),
        target_id: pick(rng, &["intro", "lead-text", "first"]),
        text: pick(rng, &["text", "copy", "para"]),
        accent: pick(rng, &["accent", "big", "highlight"]),
        role: pick(rng, &["note", "hint", "tip"]),
    };
    let colors = pick_n(
        rng,
        &[
            "red", "blue", "green", "orange", "purple", "teal", "brown", "gray",
        ],
        4,
    );
    let comps = compounds(&names, level);
    let pres = prefixes(&names, level);
    for _ in 0..128 {
        let mut rules: Vec<Rule> = (0..4)
            .map(|_| {
                let (c, cs, cm) = pick(rng, &comps).clone();
                let (p, ps, pm) = if level >= 2 && chance(rng, 0.6) {
                    pick(rng, &pres).clone()
                } else {
                    pres[0].clone()
                };
                Rule {
                    selector: format!("{p}{c}"),
                    spec: spec_add(ps, cs),
                    matches: cm && pm,
                    important: false,
                }
            })
            .collect();
        if level == 3 && chance(rng, 0.4) {
            let weakest = (0..4).min_by_key(|i| rules[*i].spec).unwrap_or(0);
            if rules[weakest].matches {
                rules[weakest].important = true;
            }
        }
        let mut selectors: Vec<&str> = rules.iter().map(|r| r.selector.as_str()).collect();
        selectors.sort_unstable();
        selectors.dedup();
        if selectors.len() < 4 {
            continue;
        }
        let Some(win) = winner(&rules) else { continue };
        let last_matching = rules.iter().rposition(|r| r.matches);
        let tie = rules.iter().enumerate().any(|(i, r)| {
            i != win
                && r.matches
                && r.spec == rules[win].spec
                && r.important == rules[win].important
        });
        match level {
            1 if tie => continue,
            1 if last_matching == Some(win) && chance(rng, 0.6) => continue,
            2 if !tie && chance(rng, 0.5) => continue,
            3 if rules.iter().all(|r| r.matches) && !rules.iter().any(|r| r.important) => continue,
            _ => {}
        }

        let css: String = rules
            .iter()
            .zip(&colors)
            .map(|(r, c)| {
                let imp = if r.important { " !important" } else { "" };
                format!("{} {{\n  color: {c}{imp};\n}}\n", r.selector)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let html = format!(
            "```html\n<body>\n  <main id=\"{}\" class=\"{}\">\n    <section class=\"{}\">\n      <p id=\"{}\" class=\"{} {}\" data-role=\"{}\">Привет!</p>\n      <p class=\"{}\">Пока!</p>\n    </section>\n  </main>\n</body>\n```",
            names.root_id,
            names.root_class,
            names.card,
            names.target_id,
            names.text,
            names.accent,
            names.role,
            names.text
        );
        let answer = colors[win].to_owned();
        let distractors = colors
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != win)
            .map(|(_, c)| (*c).to_owned());
        return Draft::choice(
            format!(
                "Разметка:\n\n{html}\n\nКакого цвета будет текст «Привет!» с учётом стилей ниже?"
            ),
            answer,
            distractors,
        )
        .code("css", css);
    }
    Draft::choice(
        "Какого цвета будет текст абзаца `<p id=\"intro\" class=\"text\">`, если заданы правила `p { color: red }`, `.text { color: blue }`, `#intro { color: green }`, `p.text { color: orange }`?",
        "green",
        ["red", "blue", "orange"].map(str::to_owned),
    )
}

fn padding_shorthand(rng: &mut Rng) -> (String, i64, i64) {
    let v: Vec<i64> = (0..4).map(|_| range(rng, 1, 6) * 4).collect();
    match range(rng, 1, 4) {
        1 => (format!("{}px", v[0]), v[0], v[0]),
        2 => (format!("{}px {}px", v[0], v[1]), v[1], v[1]),
        3 => (format!("{}px {}px {}px", v[0], v[1], v[2]), v[1], v[1]),
        _ => (
            format!("{}px {}px {}px {}px", v[0], v[1], v[2], v[3]),
            v[3],
            v[1],
        ),
    }
}

fn collapse(a: i64, b: i64) -> i64 {
    let pos = a.max(b).max(0);
    let neg = a.min(b).min(0);
    pos + neg
}

const BOX_CLASSES: &[&str] = &[
    "box", "card", "panel", "tile", "badge", "banner", "button", "modal", "alert", "chip",
];
const BORDER_STYLES: &[&str] = &[
    "solid #333",
    "solid #ccc",
    "solid black",
    "dashed #999",
    "dotted #666",
];
const LIST_CLASSES: &[&str] = &[
    "list", "stack", "column", "feed", "menu", "sidebar", "content", "wrapper",
];
const SIBLING_CLASSES: &[(&str, &str)] = &[
    ("a", "b"),
    ("first", "second"),
    ("top", "bottom"),
    ("heading", "excerpt"),
    ("title", "text"),
    ("card", "note"),
    ("intro", "details"),
    ("upper", "lower"),
    ("item-1", "item-2"),
    ("alert", "form"),
];

fn box_model(level: u8, rng: &mut Rng) -> Draft {
    match level {
        1 | 2 => {
            let class = *pick(rng, BOX_CLASSES);
            let width = range(rng, 20, 60) * 10;
            let (padding, left, right) = padding_shorthand(rng);
            let border = range(rng, 1, 5);
            let border_style = *pick(rng, BORDER_STYLES);
            let margin = range(rng, 1, 5) * 5;
            let border_box = level == 2;
            let sizing = if border_box {
                "  box-sizing: border-box;\n"
            } else {
                ""
            };
            let css = format!(
                ".{class} {{\n{sizing}  width: {width}px;\n  padding: {padding};\n  border: {border}px {border_style};\n  margin: {margin}px;\n}}\n"
            );
            if border_box {
                Draft::integer(
                    format!(
                        "Какой будет ширина области контента элемента `.{class}` (без padding и border), px?"
                    ),
                    width - left - right - 2 * border,
                )
                .code("css", css)
            } else {
                Draft::integer(
                    format!(
                        "Какой будет ширина элемента `.{class}` от внешнего края левой рамки до внешнего края правой (без margin), px?"
                    ),
                    width + left + right + 2 * border,
                )
                .code("css", css)
            }
        }
        _ => {
            let (a, b) = match range(rng, 0, 3) {
                0 => (-range(rng, 1, 10) * 2, range(rng, 12, 32) * 2),
                1 => (range(rng, 12, 32) * 2, -range(rng, 1, 10) * 2),
                _ => (range(rng, 4, 32) * 2, range(rng, 4, 32) * 2),
            };
            let flex = chance(rng, 0.4);
            let gap = if flex { a + b } else { collapse(a, b) };
            let list = *pick(rng, LIST_CLASSES);
            let (first, second) = *pick(rng, SIBLING_CLASSES);
            let outer_tag = *pick(rng, &["div", "section", "main"]);
            let tag = *pick(rng, &["div", "section", "article"]);
            let display = if flex {
                "display: flex;\n  flex-direction: column;"
            } else {
                "display: block;"
            };
            let container = format!(".{list} {{\n  {display}\n}}\n");
            let items = format!(
                ".{first} {{\n  margin-bottom: {a}px;\n}}\n\n.{second} {{\n  margin-top: {b}px;\n}}\n"
            );
            let css = if chance(rng, 0.5) {
                format!("{items}\n{container}")
            } else {
                format!("{container}\n{items}")
            };
            let question = if chance(rng, 0.5) {
                format!(
                    "Каким будет расстояние по вертикали между нижней границей `.{first}` и верхней границей `.{second}`, px?"
                )
            } else {
                format!(
                    "Сколько пикселей по вертикали будет между нижним краем `.{first}` и верхним краем `.{second}`?"
                )
            };
            Draft::integer(
                format!(
                    "Внутри `<{outer_tag} class=\"{list}\">` подряд идут блоки с текстом `<{tag} class=\"{first}\">` и `<{tag} class=\"{second}\">`, у них нет padding и border. {question}"
                ),
                gap,
            )
            .code("css", css)
        }
    }
}

#[derive(Clone, Copy)]
enum Update {
    Value(Op, i64),
    Func(Op, i64),
}

fn apply(op: Op, base: i64, k: i64) -> i64 {
    match op {
        Op::Add => base + k,
        Op::Sub => base - k,
        Op::Mul => base * k,
    }
}

fn run_click(state: i64, updates: &[Update]) -> i64 {
    updates.iter().fold(state, |acc, u| match *u {
        Update::Value(op, k) => apply(op, state, k),
        Update::Func(op, k) => apply(op, acc, k),
    })
}

const COUNTERS: &[(&str, &str, &str)] = &[
    ("count", "setCount", "Counter"),
    ("score", "setScore", "ScoreBoard"),
    ("likes", "setLikes", "LikeButton"),
    ("clicks", "setClicks", "ClickCounter"),
    ("total", "setTotal", "TotalCounter"),
    ("points", "setPoints", "PointsWidget"),
    ("votes", "setVotes", "VoteButton"),
    ("value", "setValue", "Stepper"),
    ("level", "setLevel", "LevelUp"),
    ("stars", "setStars", "StarRating"),
    ("steps", "setSteps", "StepCounter"),
    ("coins", "setCoins", "CoinBank"),
];
const HANDLERS: &[&str] = &[
    "handleClick",
    "handlePress",
    "handleButtonClick",
    "handleTap",
    "onButtonClick",
];
/// Never equal to a state name: shadowing the state inside the updater would be an extra trap.
const UPDATER_ARGS: &[&str] = &["c", "prev", "p", "x", "n", "v", "cur"];

fn state_batching(level: u8, rng: &mut Rng) -> Draft {
    let initial = range(rng, 0, 5);
    let n = range(rng, 2, 4) as usize;
    let mut updates: Vec<Update> = (0..n)
        .map(|_| {
            let op = if chance(rng, 0.25) { Op::Mul } else { Op::Add };
            let k = if op == Op::Mul {
                range(rng, 2, 3)
            } else {
                range(rng, 1, 5)
            };
            if level == 1 || chance(rng, 0.5) {
                Update::Value(op, k)
            } else {
                Update::Func(op, k)
            }
        })
        .collect();
    if level > 1 && !updates.iter().any(|u| matches!(u, Update::Func(..))) {
        updates[n - 1] = Update::Func(Op::Add, range(rng, 1, 5));
    }
    if level > 1 && !updates.iter().any(|u| matches!(u, Update::Value(..))) {
        updates[0] = Update::Value(Op::Add, range(rng, 1, 5));
    }
    let clicks = if level == 3 { 2 } else { 1 };
    let answer = (0..clicks).fold(initial, |s, _| run_click(s, &updates));
    let (state, setter, component) = *pick(rng, COUNTERS);
    let handler = *pick(rng, HANDLERS);
    let arg = *pick(rng, UPDATER_ARGS);
    let calls: String = updates
        .iter()
        .map(|u| match *u {
            Update::Value(op, k) => format!("    {setter}({state} {} {k});\n", op.symbol()),
            Update::Func(op, k) => {
                format!("    {setter}(({arg}) => {arg} {} {k});\n", op.symbol())
            }
        })
        .collect();
    let handler_fn = if chance(rng, 0.5) {
        format!("  function {handler}() {{\n{calls}  }}\n")
    } else {
        format!("  const {handler} = () => {{\n{calls}  }};\n")
    };
    let code = format!(
        "import {{ useState }} from 'react';\n\nexport default function {component}() {{\n  const [{state}, {setter}] = useState({initial});\n\n{handler_fn}\n  return <button onClick={{{handler}}}>{{{state}}}</button>;\n}}\n"
    );
    let clicks_text = if clicks == 1 {
        "одного клика"
    } else {
        "двух кликов подряд (после каждого компонент успевает перерисоваться)"
    };
    let question = match range(rng, 0, 2) {
        0 => format!(
            "Компонент на React 18. Какое значение `{state}` отобразит кнопка после {clicks_text}?"
        ),
        1 => format!(
            "Компонент работает на React 18. Какое число покажет кнопка после {clicks_text}?"
        ),
        _ => format!("Компонент на React 18. Что будет написано на кнопке после {clicks_text}?"),
    };
    Draft::integer(question, answer).code("jsx", code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(n: usize) -> Vec<char> {
        ('A'..='Z').take(n).collect()
    }

    #[test]
    fn event_loop_classic_order() {
        let program = vec![
            Js::Log(0),
            Js::Timeout {
                delay: 0,
                body: vec![Js::Log(1)],
            },
            Js::Then(vec![vec![Js::Log(2)], vec![Js::Log(3)]]),
            Js::Micro(vec![Js::Log(4)]),
            Js::Log(5),
        ];
        assert_eq!(
            EventLoop::run(&labels(6), &program, LoopModel::default()),
            "AFCEDB"
        );
    }

    #[test]
    fn event_loop_async_and_executor() {
        let program = vec![
            Js::Async {
                before: vec![Js::Log(0)],
                after: vec![Js::Log(1)],
            },
            Js::NewPromise {
                executor: vec![Js::Log(2)],
                then: vec![Js::Log(3)],
            },
            Js::Micro(vec![
                Js::Log(4),
                Js::Timeout {
                    delay: 0,
                    body: vec![Js::Log(5)],
                },
            ]),
            Js::Timeout {
                delay: 100,
                body: vec![Js::Log(6)],
            },
            Js::Timeout {
                delay: 0,
                body: vec![Js::Log(7), Js::Micro(vec![Js::Log(8)])],
            },
        ];
        assert_eq!(
            EventLoop::run(&labels(9), &program, LoopModel::default()),
            "ACBDEHIFG"
        );
    }

    #[test]
    fn coercion_rules() {
        let s = |x: &str| JsVal::Str(x.to_owned());
        let n = |x: i64| JsVal::Num(Some(x));
        let eval = |v: &[JsVal], o: &[Op]| js_eval(v, o, CoercionModel::default()).literal();
        assert_eq!(eval(&[s("5"), n(3)], &[Op::Add]), "\"53\"");
        assert_eq!(eval(&[s("5"), n(3)], &[Op::Sub]), "2");
        assert_eq!(eval(&[n(1), n(2), s("3")], &[Op::Add, Op::Add]), "\"33\"");
        assert_eq!(eval(&[s("1"), n(2), n(3)], &[Op::Add, Op::Add]), "\"123\"");
        assert_eq!(
            eval(&[JsVal::Bool(true), n(1), s("2")], &[Op::Add, Op::Add]),
            "\"22\""
        );
        assert_eq!(eval(&[JsVal::Null, n(1)], &[Op::Add]), "1");
        assert_eq!(eval(&[JsVal::Undef, n(1)], &[Op::Add]), "NaN");
        assert_eq!(eval(&[s("3"), s("4"), n(1)], &[Op::Mul, Op::Add]), "13");
        assert_eq!(eval(&[n(1), s("2"), s("3")], &[Op::Add, Op::Mul]), "7");
        assert_eq!(eval(&[JsVal::Null, s("x")], &[Op::Add]), "\"nullx\"");
    }

    #[test]
    fn specificity_cascade() {
        let rule = |spec: Spec, matches: bool, important: bool| Rule {
            selector: String::new(),
            spec,
            matches,
            important,
        };
        let rules = [
            rule((0, 1, 0), true, false),
            rule((1, 0, 0), false, false),
            rule((0, 1, 0), true, false),
            rule((0, 0, 1), true, false),
        ];
        assert_eq!(winner(&rules), Some(2));
        let rules = [rule((1, 0, 1), true, false), rule((0, 0, 1), true, true)];
        assert_eq!(winner(&rules), Some(1));
    }

    #[test]
    fn margins_and_batching() {
        assert_eq!(collapse(20, 30), 30);
        assert_eq!(collapse(-10, 30), 20);
        assert_eq!(collapse(-10, -20), -20);
        let updates = [
            Update::Value(Op::Add, 1),
            Update::Func(Op::Add, 2),
            Update::Value(Op::Add, 5),
            Update::Func(Op::Mul, 2),
        ];
        assert_eq!(run_click(0, &updates), 10);
        assert_eq!(run_click(10, &updates), 30);
    }
}
