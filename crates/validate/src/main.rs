mod assessment_eval;
mod csv;
mod matching_eval;
mod pairs;

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
        "usage: validate [--out DIR] [--seed N] [all|matching|assessment]\n       validate pairs --input FILE.json|DIR [--k N] [--now YYYY-MM-DD] [--out REPORT.md]"
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
    let (mut matching, mut assessment) = (true, true);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next().ok_or(ValidateError::Usage)?.into(),
            "--seed" => {
                seed = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or(ValidateError::Usage)?
            }
            "all" => {}
            "matching" => assessment = false,
            "assessment" => matching = false,
            _ => return Err(ValidateError::Usage),
        }
    }
    std::fs::create_dir_all(&out)?;

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
        let report = matching_eval::run(seed);
        let md = matching_eval::markdown(&report);
        std::fs::write(out.join("matching.md"), &md)?;
        std::fs::write(
            out.join("matching.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("{md}");
    }
    Ok(())
}
