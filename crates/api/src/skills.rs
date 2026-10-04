use std::borrow::Cow;

pub fn is_known(slug: &str) -> bool {
    matching::lookup(slug).is_some()
}

pub fn display_name(slug: &str) -> &str {
    matching::lookup(slug).map_or(slug, |d| d.name)
}

/// Maps aliases ("постгрес", "k8s") to canonical slugs, keeping order and dropping duplicates.
/// Unknown entries are returned as the error so the caller can report them.
pub fn canonicalize(input: &[String]) -> Result<Vec<String>, Vec<String>> {
    let mut out: Vec<String> = Vec::with_capacity(input.len());
    let mut unknown = Vec::new();
    for raw in input {
        match matching::lookup(raw).or_else(|| matching::normalize(raw)) {
            Some(def) if !out.iter().any(|s| s == def.slug) => out.push(def.slug.to_owned()),
            Some(_) => {}
            None => unknown.push(raw.clone()),
        }
    }
    if unknown.is_empty() {
        Ok(out)
    } else {
        Err(unknown)
    }
}

pub fn unknown_message(unknown: &[String]) -> Cow<'static, str> {
    Cow::Owned(format!("Неизвестные навыки: {}", unknown.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_aliases() {
        let input = vec![
            "постгрес".to_owned(),
            "postgresql".to_owned(),
            "k8s".to_owned(),
        ];
        assert_eq!(
            canonicalize(&input).unwrap(),
            vec!["postgresql", "kubernetes"]
        );
        assert_eq!(
            canonicalize(&["нечто".to_owned()]).unwrap_err(),
            vec!["нечто"]
        );
    }
}
