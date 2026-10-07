mod assessment_eval;
mod csv;
mod matching_eval;
mod model;
mod models;
mod pairs;
mod paraphrase;
mod texts;

use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, thiserror::Error)]
enum ValidateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Pairs(#[from] pairs::PairsCliError),
    #[error(
        "usage: validate [--out DIR] [--seed N] [--model NAME|none] [--models DIR] [all|matching|assessment|models]\n       validate pairs --input FILE.json|DIR [--k N] [--now YYYY-MM-DD] [--out REPORT.md] [--model NAME|none] [--models DIR]"
    )]
    Usage,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), ValidateError> {
    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_some_and(|a| a == "pairs") {
        args.next();
        return Ok(pairs::run(args)?);
    }
    let mut out = PathBuf::from("docs/validation");
    let mut seed = 2026u64;
    let mut choice = model::Choice::default();
    let (mut matching, mut assessment, mut compare) = (true, true, false);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next().ok_or(ValidateError::Usage)?.into(),
            "--model" => choice.name = args.next().ok_or(ValidateError::Usage)?,
            "--models" => choice.dir = args.next().ok_or(ValidateError::Usage)?.into(),
            "--seed" => {
                seed = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or(ValidateError::Usage)?
            }
            "all" => {}
            "matching" => assessment = false,
            "assessment" => matching = false,
            "models" => {
                (matching, assessment, compare) = (false, false, true);
            }
            _ => return Err(ValidateError::Usage),
        }
    }
    std::fs::create_dir_all(&out)?;

    if compare {
        let report = models::run(seed, &choice.dir);
        let md = models::markdown(&report);
        std::fs::write(out.join("embeddings.md"), &md)?;
        std::fs::write(
            out.join("embeddings.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("{md}");
    }
    if assessment {
        let report = assessment_eval::run(seed);
        let md = assessment_eval::markdown(&report);
        std::fs::write(out.join("assessment.md"), &md)?;
        std::fs::write(
            out.join("assessment.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("{md}");
    }
    if matching {
        let loaded = choice.load();
        let report = matching_eval::run(seed, loaded.as_ref());
        let md = matching_eval::markdown(&report);
        std::fs::write(out.join("matching.md"), &md)?;
        std::fs::write(
            out.join("matching.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("{md}");
        if let Some(report) = loaded.as_ref().and_then(paraphrase::run) {
            let md = paraphrase::markdown(&report);
            std::fs::write(out.join("paraphrase.md"), &md)?;
            std::fs::write(
                out.join("paraphrase.json"),
                serde_json::to_string_pretty(&report)?,
            )?;
            println!("{md}");
        }
    }
    Ok(())
}
