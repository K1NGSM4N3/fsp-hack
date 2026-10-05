//! Salary range sanity check against the category market: what candidates of the category expect.
//! A flag never blocks publishing; it warns the employer and feeds the trust level.

use catalog::Category;

use super::TrustFlag;

/// Below this many stated expectations a category's percentiles are noise: no market check.
pub const MIN_MARKET_SAMPLES: u32 = 5;
/// The upper bound may exceed the category's 90th percentile of expectations by half. Offers
/// usually sit 10–30 % above what candidates ask, so even a generous range stays under it; a
/// range above it promises more than almost anyone in the category expects — the bait of fake
/// vacancies.
pub const ABOVE_MARKET_FACTOR: f64 = 1.5;
/// `max ≥ 3 × min` spans junior to senior pay (110 000 → 380 000 ₽ is ×3.5): that is not one
/// position, and "от 50 000 до 500 000" ranges are a classic fake.
pub const WIDE_RANGE_RATIO: f64 = 3.0;

/// Stated expectations (`salary_expectation_min`) of the category's searchable candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketBand {
    pub samples: u32,
    pub median: i64,
    pub p90: i64,
}

pub fn salary_flags(
    category: Category,
    min: i64,
    max: i64,
    market: Option<MarketBand>,
) -> Vec<TrustFlag> {
    let mut flags = Vec::new();
    if let Some(m) = market.filter(|m| m.samples >= MIN_MARKET_SAMPLES && m.p90 > 0)
        && max as f64 > m.p90 as f64 * ABOVE_MARKET_FACTOR
    {
        flags.push(TrustFlag::SalaryAboveMarket {
            category,
            max,
            p90: m.p90,
        });
    }
    if min > 0 && max as f64 >= min as f64 * WIDE_RANGE_RATIO {
        flags.push(TrustFlag::SalaryWideRange { min, max });
    }
    flags
}

#[cfg(test)]
mod tests {
    use catalog::{Grade, Specialization};

    use super::*;

    const MIDDLE: Category = Category::new(Specialization::Backend, Grade::Middle);

    fn band(p90: i64, samples: u32) -> Option<MarketBand> {
        Some(MarketBand {
            samples,
            median: p90 * 4 / 5,
            p90,
        })
    }

    #[test]
    fn generous_but_plausible_range_passes() {
        assert!(salary_flags(MIDDLE, 280_000, 360_000, band(290_000, 12)).is_empty());
        assert!(salary_flags(MIDDLE, 300_000, 435_000, band(290_000, 12)).is_empty());
    }

    #[test]
    fn upper_bound_far_above_p90_is_flagged() {
        let flags = salary_flags(MIDDLE, 300_000, 436_000, band(290_000, 12));
        assert_eq!(
            flags,
            vec![TrustFlag::SalaryAboveMarket {
                category: MIDDLE,
                max: 436_000,
                p90: 290_000
            }]
        );
    }

    #[test]
    fn thin_market_is_not_judged() {
        assert!(salary_flags(MIDDLE, 600_000, 900_000, band(290_000, 4)).is_empty());
        assert!(salary_flags(MIDDLE, 600_000, 900_000, None).is_empty());
    }

    #[test]
    fn three_fold_range_is_wide() {
        assert!(salary_flags(MIDDLE, 100_000, 299_000, None).is_empty());
        assert_eq!(
            salary_flags(MIDDLE, 100_000, 300_000, None),
            vec![TrustFlag::SalaryWideRange {
                min: 100_000,
                max: 300_000
            }]
        );
        assert_eq!(
            salary_flags(MIDDLE, 50_000, 900_000, band(290_000, 12)).len(),
            2
        );
    }
}
