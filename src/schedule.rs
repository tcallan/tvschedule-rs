use time::Date;
use time::Duration;
use time::OffsetDateTime;
use time::Weekday;

pub(crate) fn now() -> Date {
    OffsetDateTime::now_utc().date()
}

pub(crate) fn get_week(date: Date) -> (Date, Date) {
    let duration = if date.weekday() == Weekday::Sunday {
        Duration::days(1)
    } else {
        Duration::days(0)
    };
    let (year, week, _day) = date.saturating_add(duration).to_iso_week_date();
    let start = Date::from_iso_week_date(year, week, Weekday::Monday)
        .unwrap()
        .saturating_sub(Duration::days(1));
    let end = Date::from_iso_week_date(year, week, Weekday::Sunday)
        .unwrap()
        .saturating_sub(Duration::days(1));
    (start, end)
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;
    use time::Month;

    use super::*;

    #[test]
    fn get_week_sunday_start() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let (start, _end) = get_week(date);
        assert_eq!(start, date);
    }

    #[test]
    fn get_week_saturday_end() {
        let date = Date::from_calendar_date(2022, Month::May, 7).unwrap();
        let (_start, end) = get_week(date);
        assert_eq!(end, date);
    }

    #[quickcheck]
    fn get_week_starts_on_sunday(date: Date) -> bool {
        let (start, _end) = get_week(date);
        // NOTE: start gets clamped to Date::MIN
        start == Date::MIN || start.weekday() == Weekday::Sunday
    }

    #[quickcheck]
    fn get_week_ends_ons_saturday(date: Date) -> bool {
        let (_start, end) = get_week(date);
        // NOTE: end gets clamped to Date::MAX
        end == Date::MAX || end.weekday() == Weekday::Saturday
    }

    #[quickcheck]
    fn get_week_contains_requested_date(date: Date) -> bool {
        let (start, end) = get_week(date);
        start <= date && date <= end
    }

    #[quickcheck]
    fn get_week_start_is_before_end(date: Date) -> bool {
        let (start, end) = get_week(date);
        start <= end
    }

    #[quickcheck]
    fn get_week_is_proper_length(date: Date) -> bool {
        let (start, end) = get_week(date);
        start == Date::MIN || end == Date::MAX || end - start == Duration::days(6)
    }
}
