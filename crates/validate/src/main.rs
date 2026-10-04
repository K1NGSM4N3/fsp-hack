mod matching_eval;
mod stats;

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
enum ValidateError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("usage: validate [--out DIR] [--seed N] [matching]")]
    Usage,
}

fn main() -> Result<(), ValidateError> {
    let mut out = PathBuf::from("docs/validation");
    let mut seed = 2026u64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next().ok_or(ValidateError::Usage)?.into(),
            "--seed" => {
                seed = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or(ValidateError::Usage)?
            }
            "matching" | "all" => {}
            _ => return Err(ValidateError::Usage),
        }
    }
    std::fs::create_dir_all(&out)?;

    let report = matching_eval::run(seed);
    let md = matching_eval::markdown(&report);
    std::fs::write(out.join("matching.md"), &md)?;
    std::fs::write(
        out.join("matching.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{md}");
    Ok(())
}
