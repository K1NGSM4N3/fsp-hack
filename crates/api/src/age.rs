//! Age gate (docs/architecture.md §9.5): candidates are 18+ by declaration, or 14–17 with the consent
//! of a legal representative; younger people cannot register. Every function takes `today`.
//!
//! SQL derives the same «minor» as `birth_date + interval '18 years' > current_date`. Both chrono
//! and Postgres move 29 February to 28 February in a non-leap year, so the two never disagree.

use chrono::{Datelike, Days, Months, NaiveDate};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const ADULT_AGE: u32 = 18;
pub const MIN_AGE: u32 = 14;
/// The representative's data outlives the 18th birthday by this long: room to settle a dispute
/// about something that happened while their consent was the legal basis.
pub const GUARDIAN_RETENTION_DAYS: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AgeBracket {
    /// 18 or older.
    Adult,
    /// 14–17.
    Minor,
}

impl From<db::types::AgeBracket> for AgeBracket {
    fn from(a: db::types::AgeBracket) -> Self {
        match a {
            db::types::AgeBracket::Adult => AgeBracket::Adult,
            db::types::AgeBracket::Minor => AgeBracket::Minor,
        }
    }
}

impl From<AgeBracket> for db::types::AgeBracket {
    fn from(a: AgeBracket) -> Self {
        match a {
            AgeBracket::Adult => db::types::AgeBracket::Adult,
            AgeBracket::Minor => db::types::AgeBracket::Minor,
        }
    }
}

/// Why a birth date given with «Мне 14–17 лет» is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinorBirthDate {
    InFuture,
    TooYoung,
    AlreadyAdult,
}

impl MinorBirthDate {
    pub fn message(self) -> &'static str {
        match self {
            MinorBirthDate::InFuture => "Дата рождения ещё не наступила",
            MinorBirthDate::TooYoung => "Регистрация открыта с 14 лет",
            MinorBirthDate::AlreadyAdult => {
                "По дате рождения вам уже есть 18 — выберите «Мне есть 18 лет»"
            }
        }
    }
}

/// `years` after `birth`; 29 February becomes 28 February in a non-leap year.
pub fn anniversary(birth: NaiveDate, years: u32) -> NaiveDate {
    birth
        .checked_add_months(Months::new(years * 12))
        .unwrap_or(NaiveDate::MAX)
}

/// Full years on `today`.
pub fn age_on(birth: NaiveDate, today: NaiveDate) -> u32 {
    if today < birth {
        return 0;
    }
    let years = (today.year() - birth.year()) as u32;
    if anniversary(birth, years) > today {
        years - 1
    } else {
        years
    }
}

/// The 18th birthday: from this day the candidate is an adult and every restriction lifts.
pub fn adult_from(birth: NaiveDate) -> NaiveDate {
    anniversary(birth, ADULT_AGE)
}

/// Adults have no birth date on record, so `None` is an adult.
pub fn is_minor(birth: Option<NaiveDate>, today: NaiveDate) -> bool {
    birth.is_some_and(|b| today < adult_from(b))
}

/// The day the representative's name, email, IP and browser are erased, and the birth date with them.
pub fn guardian_data_erased_on(birth: NaiveDate) -> NaiveDate {
    adult_from(birth)
        .checked_add_days(Days::new(u64::from(GUARDIAN_RETENTION_DAYS)))
        .unwrap_or(NaiveDate::MAX)
}

/// A birth date given at registration as «Мне 14–17 лет» must really be 14–17 today.
pub fn check_minor_birth_date(birth: NaiveDate, today: NaiveDate) -> Result<(), MinorBirthDate> {
    if birth > today {
        return Err(MinorBirthDate::InFuture);
    }
    match age_on(birth, today) {
        a if a < MIN_AGE => Err(MinorBirthDate::TooYoung),
        a if a >= ADULT_AGE => Err(MinorBirthDate::AlreadyAdult),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn full_years_change_on_the_birthday() {
        let birth = d(2010, 3, 15);
        assert_eq!(age_on(birth, d(2026, 3, 14)), 15);
        assert_eq!(age_on(birth, d(2026, 3, 15)), 16);
        assert_eq!(age_on(birth, d(2026, 12, 31)), 16);
        assert_eq!(age_on(birth, birth), 0);
        assert_eq!(age_on(birth, d(2000, 1, 1)), 0);
    }

    #[test]
    fn leap_day_birthday_counts_on_28_february() {
        let birth = d(2008, 2, 29);
        assert_eq!(adult_from(birth), d(2026, 2, 28));
        assert_eq!(age_on(birth, d(2026, 2, 27)), 17);
        assert_eq!(age_on(birth, d(2026, 2, 28)), 18);
        assert!(is_minor(Some(birth), d(2026, 2, 27)));
        assert!(!is_minor(Some(birth), d(2026, 2, 28)));
        assert_eq!(anniversary(birth, 4), d(2012, 2, 29));
    }

    #[test]
    fn restrictions_lift_on_the_18th_birthday() {
        let birth = d(2008, 10, 6);
        assert!(is_minor(Some(birth), d(2026, 10, 5)));
        assert!(!is_minor(Some(birth), d(2026, 10, 6)));
        assert!(!is_minor(None, d(2026, 10, 6)), "no birth date means adult");
    }

    #[test]
    fn guardian_data_lives_30_days_past_18() {
        assert_eq!(guardian_data_erased_on(d(2008, 10, 6)), d(2026, 11, 5));
        assert_eq!(guardian_data_erased_on(d(2008, 12, 20)), d(2027, 1, 19));
    }

    #[test]
    fn registration_accepts_exactly_14_to_17() {
        let today = d(2026, 10, 6);
        assert_eq!(
            check_minor_birth_date(d(2012, 10, 7), today),
            Err(MinorBirthDate::TooYoung),
            "a day short of 14"
        );
        assert_eq!(check_minor_birth_date(d(2012, 10, 6), today), Ok(()));
        assert_eq!(check_minor_birth_date(d(2008, 10, 7), today), Ok(()));
        assert_eq!(
            check_minor_birth_date(d(2008, 10, 6), today),
            Err(MinorBirthDate::AlreadyAdult)
        );
        assert_eq!(
            check_minor_birth_date(d(2026, 10, 7), today),
            Err(MinorBirthDate::InFuture)
        );
        assert_eq!(
            check_minor_birth_date(d(1950, 1, 1), today),
            Err(MinorBirthDate::AlreadyAdult)
        );
    }
}
