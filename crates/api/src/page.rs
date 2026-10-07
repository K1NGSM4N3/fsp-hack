//! Keyset pagination of lists that keep growing (docs/api.md, «Пагинация»): the client passes back an
//! opaque `next_cursor`, the next page starts strictly after that `(time, id)` position.

use chrono::{DateTime, TimeZone, Utc};
use db::types::Cursor;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

pub const DEFAULT: i64 = 30;
pub const MAX: i64 = 100;

#[derive(Deserialize, IntoParams)]
pub struct PageQuery {
    /// `next_cursor` of the previous page.
    pub cursor: Option<String>,
    /// 1–100, 30 by default.
    pub limit: Option<i64>,
}

/// A page of a list, newest first.
#[derive(Serialize, ToSchema)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    /// Pass as `cursor` for the next page; `null` on the last one.
    pub next_cursor: Option<String>,
}

/// `<unix microseconds>_<uuid>`.
pub fn encode(at: DateTime<Utc>, id: Uuid) -> String {
    format!("{}_{}", at.timestamp_micros(), id.simple())
}

pub fn decode(raw: Option<&str>) -> ApiResult<Option<Cursor>> {
    let Some(raw) = raw.filter(|r| !r.is_empty()) else {
        return Ok(None);
    };
    let invalid = || {
        ApiError::bad_request("invalid_query", "Некорректные параметры запроса")
            .with_detail("cursor: возьмите значение `next_cursor` из предыдущего ответа")
    };
    let (micros, id) = raw.split_once('_').ok_or_else(invalid)?;
    let micros: i64 = micros.parse().map_err(|_| invalid())?;
    let at = Utc.timestamp_micros(micros).single().ok_or_else(invalid)?;
    let id: Uuid = id.parse().map_err(|_| invalid())?;
    Ok(Some(Cursor { at, id }))
}

pub fn limit(raw: Option<i64>, default: i64, max: i64) -> i64 {
    raw.unwrap_or(default).clamp(1, max)
}

/// `rows` were fetched with `limit + 1`: keeps `limit` of them and, when there was one more, the
/// cursor after the last one kept.
pub fn split<T>(
    mut rows: Vec<T>,
    limit: i64,
    key: impl Fn(&T) -> (DateTime<Utc>, Uuid),
) -> (Vec<T>, Option<String>) {
    let keep = usize::try_from(limit).unwrap_or(usize::MAX);
    if rows.len() <= keep {
        return (rows, None);
    }
    rows.truncate(keep);
    let next = rows.last().map(|r| {
        let (at, id) = key(r);
        encode(at, id)
    });
    (rows, next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trip() {
        let at = Utc
            .timestamp_micros(1_790_000_000_123_456)
            .single()
            .unwrap();
        let id = Uuid::now_v7();
        let raw = encode(at, id);
        let c = decode(Some(&raw)).unwrap().unwrap();
        assert_eq!((c.at, c.id), (at, id));
        assert!(decode(None).unwrap().is_none());
        assert!(decode(Some("")).unwrap().is_none());
        for bad in ["x", "12_nope", "abc_0191f0000000700080000000000000a1"] {
            assert!(decode(Some(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn split_keeps_the_page_and_points_after_it() {
        let at = |s: i64| Utc.timestamp_opt(1_790_000_000 + s, 0).single().unwrap();
        let rows: Vec<(DateTime<Utc>, Uuid)> = (0..3)
            .map(|i| (at(-i), Uuid::from_u128(i as u128)))
            .collect();
        let (page, next) = split(rows.clone(), 2, |r| *r);
        assert_eq!(page, rows[..2]);
        assert_eq!(next, Some(encode(rows[1].0, rows[1].1)));
        let (page, next) = split(rows.clone(), 3, |r| *r);
        assert_eq!((page.len(), next), (3, None));
        assert_eq!(limit(None, 30, 100), 30);
        assert_eq!(limit(Some(0), 30, 100), 1);
        assert_eq!(limit(Some(500), 30, 100), 100);
    }
}
