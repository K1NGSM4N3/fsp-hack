//! CSV input for `validate pairs`: `vacancies.csv`, `candidates.csv` and optional `labels.csv`
//! in one directory. Rows become the same JSON objects the API accepts, so field rules and
//! error messages come from one place: the `matching::pairs` input types.

use std::path::Path;

use matching::pairs::{CandidateInput, LabelInput, PairsInput, VacancyInput};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

#[derive(Debug, thiserror::Error)]
pub enum CsvError {
    #[error("{file}: {source}")]
    Io {
        file: String,
        source: std::io::Error,
    },
    #[error("{file}, строка {line}: незакрытая кавычка")]
    UnclosedQuote { file: String, line: usize },
    #[error("{file}, строка {line}: {column}: {source}")]
    Cell {
        file: String,
        line: usize,
        column: &'static str,
        source: CellError,
    },
    #[error("{file}, строка {line}: {source}")]
    Record {
        file: String,
        line: usize,
        source: serde_json::Error,
    },
    #[error("{file}: нет колонки «{column}»")]
    MissingColumn { file: String, column: &'static str },
}

#[derive(Debug, thiserror::Error)]
pub enum CellError {
    #[error("«{0}» — не число")]
    NotNumber(String),
    #[error("«{0}» — ожидается true или false")]
    NotBool(String),
    #[error("некорректный JSON: {0}")]
    Json(#[source] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    /// Separated by `|`, `;` or `,`.
    List,
    Number,
    Bool,
    Json,
}

use Kind::*;

const VACANCY_COLUMNS: &[(&str, Kind)] = &[
    ("id", Text),
    ("text", Text),
    ("specialization", Text),
    ("grades", List),
    ("allow_adjacent_grades", Bool),
    ("must_skills", List),
    ("nice_skills", List),
    ("salary_min", Number),
    ("salary_max", Number),
    ("work_formats", List),
    ("city", Text),
];

const CANDIDATE_COLUMNS: &[(&str, Kind)] = &[
    ("id", Text),
    ("resume_text", Text),
    ("specialization", Text),
    ("grade", Text),
    ("skills", List),
    ("salary_expectation", Number),
    ("work_formats", List),
    ("city", Text),
    ("relocation", Bool),
    ("test_result", Number),
    ("fsp_achievements", Json),
];

const LABEL_COLUMNS: &[(&str, Kind)] = &[
    ("vacancy_id", Text),
    ("candidate_id", Text),
    ("relevance", Number),
];

/// A record and the line it starts on.
type Record = (usize, Vec<String>);

/// The header decides the separator: whichever of `,` `;` tab occurs most outside quotes.
fn separator(text: &str) -> char {
    let header = text.lines().next().unwrap_or("");
    [',', ';', '\t']
        .into_iter()
        .max_by_key(|&c| header.matches(c).count())
        .unwrap_or(',')
}

/// RFC 4180: quoted fields may hold separators, doubled quotes and line breaks; CRLF and a
/// UTF-8 BOM are accepted. Blank lines are skipped.
fn records(text: &str, file: &str) -> Result<Vec<Record>, CsvError> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let sep = separator(text);
    let mut out: Vec<Record> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut field_started = false;
    let mut line = 1;
    let mut row_line = 1;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    field.push('"');
                    chars.next();
                }
                '"' => quoted = false,
                '\n' => {
                    line += 1;
                    field.push('\n');
                }
                '\r' => {}
                _ => field.push(c),
            }
            continue;
        }
        match c {
            '"' if !field_started => {
                quoted = true;
                field_started = true;
            }
            '\r' => {}
            '\n' => {
                if field_started || !row.is_empty() {
                    row.push(std::mem::take(&mut field));
                    out.push((row_line, std::mem::take(&mut row)));
                }
                field_started = false;
                line += 1;
                row_line = line;
            }
            c if c == sep => {
                row.push(std::mem::take(&mut field));
                field_started = false;
            }
            _ => {
                field.push(c);
                field_started = true;
            }
        }
    }
    if quoted {
        return Err(CsvError::UnclosedQuote {
            file: file.to_owned(),
            line: row_line,
        });
    }
    if field_started || !row.is_empty() {
        row.push(field);
        out.push((row_line, row));
    }
    Ok(out)
}

fn cell(kind: Kind, raw: &str) -> Result<Option<Value>, CellError> {
    let s = raw.trim();
    if s.is_empty() {
        return Ok(None);
    }
    Ok(Some(match kind {
        Text => Value::String(s.to_owned()),
        List => Value::Array(
            s.split(['|', ';', ','])
                .map(str::trim)
                .filter(|x| !x.is_empty())
                .map(|x| Value::String(x.to_owned()))
                .collect(),
        ),
        Number => {
            let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
            let n: f64 = compact
                .replace(',', ".")
                .parse()
                .map_err(|_| CellError::NotNumber(s.to_owned()))?;
            if n.fract() == 0.0 && n.abs() < 1e15 {
                Value::from(n as i64)
            } else {
                Value::from(n)
            }
        }
        Bool => match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "да" => Value::Bool(true),
            "false" | "0" | "no" | "нет" => Value::Bool(false),
            _ => return Err(CellError::NotBool(s.to_owned())),
        },
        Json => serde_json::from_str(s).map_err(CellError::Json)?,
    }))
}

fn read_table<T: DeserializeOwned>(
    dir: &Path,
    name: &str,
    columns: &[(&'static str, Kind)],
    required: &[&'static str],
    warnings: &mut Vec<String>,
) -> Result<Vec<T>, CsvError> {
    let path = dir.join(name);
    let text = std::fs::read_to_string(&path).map_err(|source| CsvError::Io {
        file: path.display().to_string(),
        source,
    })?;
    let mut rows = records(&text, name)?.into_iter();
    let Some((_, header)) = rows.next() else {
        return Err(CsvError::MissingColumn {
            file: name.to_owned(),
            column: required.first().copied().unwrap_or("id"),
        });
    };
    let mut layout: Vec<Option<(&'static str, Kind)>> = Vec::with_capacity(header.len());
    for h in &header {
        let h = h.trim().to_lowercase();
        let known = columns.iter().find(|(c, _)| *c == h).copied();
        if known.is_none() && !h.is_empty() {
            warnings.push(format!("{name}: колонка «{h}» не используется"));
        }
        layout.push(known);
    }
    for &r in required {
        if !layout.iter().flatten().any(|(c, _)| *c == r) {
            return Err(CsvError::MissingColumn {
                file: name.to_owned(),
                column: r,
            });
        }
    }
    rows.map(|(line, row)| {
        let mut obj = Map::new();
        for (raw, col) in row.iter().zip(&layout) {
            if let Some((key, kind)) = col
                && let Some(v) = cell(*kind, raw).map_err(|source| CsvError::Cell {
                    file: name.to_owned(),
                    line,
                    column: key,
                    source,
                })?
            {
                obj.insert((*key).to_owned(), v);
            }
        }
        serde_json::from_value(Value::Object(obj)).map_err(|source| CsvError::Record {
            file: name.to_owned(),
            line,
            source,
        })
    })
    .collect()
}

/// Reads a CSV directory; `warnings` collects ignored columns.
pub fn load_dir(dir: &Path, warnings: &mut Vec<String>) -> Result<PairsInput, CsvError> {
    let vacancies: Vec<VacancyInput> = read_table(
        dir,
        "vacancies.csv",
        VACANCY_COLUMNS,
        &["id", "text"],
        warnings,
    )?;
    let candidates: Vec<CandidateInput> = read_table(
        dir,
        "candidates.csv",
        CANDIDATE_COLUMNS,
        &["id", "resume_text"],
        warnings,
    )?;
    let labels: Vec<LabelInput> = if dir.join("labels.csv").exists() {
        read_table(
            dir,
            "labels.csv",
            LABEL_COLUMNS,
            &["vacancy_id", "candidate_id", "relevance"],
            warnings,
        )?
    } else {
        Vec::new()
    };
    Ok(PairsInput {
        now: None,
        k: None,
        vacancies,
        candidates,
        labels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(text: &str) -> Vec<Record> {
        records(text, "t.csv").unwrap()
    }

    #[test]
    fn quoted_fields_hold_separators_quotes_and_newlines() {
        let r = rows("id,text\r\n1,\"a, \"\"b\"\"\r\nc\"\r\n\r\n2,plain\n");
        assert_eq!(r.len(), 3);
        assert_eq!(r[1], (2, vec!["1".into(), "a, \"b\"\nc".into()]));
        assert_eq!(r[2], (5, vec!["2".into(), "plain".into()]));
    }

    #[test]
    fn semicolon_header_and_bom() {
        let r = rows("\u{feff}id;text;city\n1;Go, Kafka;\n");
        assert_eq!(r[1].1, vec!["1", "Go, Kafka", ""]);
        assert!(records("a,b\n1,\"open", "t.csv").is_err());
    }

    #[test]
    fn cells_are_typed() {
        assert_eq!(
            cell(List, "go| Kafka ;PostgreSQL").unwrap(),
            Some(serde_json::json!(["go", "Kafka", "PostgreSQL"]))
        );
        assert_eq!(cell(Number, "250 000").unwrap(), Some(Value::from(250_000)));
        assert_eq!(cell(Number, "82,5").unwrap(), Some(Value::from(82.5)));
        assert_eq!(cell(Bool, "Да").unwrap(), Some(Value::Bool(true)));
        assert_eq!(cell(Text, "  ").unwrap(), None);
        assert!(cell(Number, "много").is_err());
    }
}
