//! Bridge to the skill dictionary of the `matching` crate. Until that crate lands, accept
//! lowercase ASCII slugs.

pub fn is_known(slug: &str) -> bool {
    !slug.is_empty() && slug.len() <= 40 && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

pub fn display_name(slug: &str) -> &str {
    slug
}
