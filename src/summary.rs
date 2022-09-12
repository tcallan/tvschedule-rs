use itertools::Itertools;
use time::{Date, Duration};

use std::collections::BTreeMap;

use crate::{
    schedule,
    the_movie_db::{Episode, TV},
};

#[derive(Debug)]
pub(crate) struct Summary {
    pub(crate) this_week: BTreeMap<Date, Vec<WeeklyEpisode>>,
    pub(crate) all: Vec<SeriesSummary>,
}

impl Summary {
    pub(crate) fn new(date: Date, mut tv: Vec<TV>, streaming_networks: &[u32]) -> Summary {
        tv.sort_by(|e1, e2| e1.name.cmp(&e2.name));
        Summary {
            this_week: to_this_week(date, &tv, streaming_networks),
            all: tv.iter().map(to_serires_summary).collect(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct WeeklyEpisode {
    pub(crate) series_name: String,
    pub(crate) air_date: Date,
    pub(crate) season: u32,
    pub(crate) episode: u32,
}

#[derive(Debug, PartialEq)]
pub(crate) struct SeriesSummary {
    pub(crate) name: String,
    pub(crate) last_air: Option<Date>,
    pub(crate) next_air: Option<Date>,
}

fn to_serires_summary(tv: &TV) -> SeriesSummary {
    SeriesSummary {
        name: tv.name.to_owned(),
        last_air: tv.last_episode_to_air.as_ref().map(|e| e.air_date),
        next_air: tv.next_episode_to_air.as_ref().map(|e| e.air_date),
    }
}

fn to_this_week(
    date: Date,
    tv: &[TV],
    streaming_networks: &[u32],
) -> BTreeMap<Date, Vec<WeeklyEpisode>> {
    let (start, end) = schedule::get_week(date);
    tv.iter()
        .flat_map(|tv| to_weekly_episodes(tv, streaming_networks))
        .filter(|e| start <= e.air_date && e.air_date <= end)
        .sorted_by(|a, b| Ord::cmp(&(a.air_date, &a.series_name), &(b.air_date, &b.series_name)))
        .group_by(|e| e.air_date)
        .into_iter()
        .map(|(date, group)| (date, group.collect()))
        .collect()
}

fn to_weekly_episodes(tv: &TV, streaming_networks: &[u32]) -> Vec<WeeklyEpisode> {
    vec![
        tv.last_episode_to_air.as_ref(),
        tv.next_episode_to_air.as_ref(),
    ]
    .iter()
    .filter_map(|&e| e)
    .map(|e| to_weekly_episode(tv, e, streaming_networks))
    .collect()
}

fn to_weekly_episode(tv: &TV, ep: &Episode, streaming_networks: &[u32]) -> WeeklyEpisode {
    let air_date = if tv
        .networks
        .iter()
        .any(|n| streaming_networks.contains(&n.id))
    {
        ep.air_date
    } else {
        ep.air_date.saturating_add(Duration::days(1))
    };
    WeeklyEpisode {
        series_name: tv.name.to_owned(),
        air_date,
        season: ep.season_number,
        episode: ep.episode_number,
    }
}

#[cfg(test)]
mod tests {
    use time::Month;

    use crate::the_movie_db::Network;

    use super::*;

    fn mk_tv(name: &str, last: Option<Episode>, next: Option<Episode>) -> TV {
        TV {
            id: 0,
            name: name.to_string(),
            last_episode_to_air: last,
            next_episode_to_air: next,
            networks: vec![Network {
                id: 123,
                name: String::new(),
            }],
        }
    }

    fn mk_ep(air_date: Date, season_number: u32, episode_number: u32) -> Episode {
        Episode {
            air_date,
            episode_number,
            id: 0,
            name: String::new(),
            overview: String::new(),
            season_number,
            runtime: None,
        }
    }

    #[test]
    fn to_weekly_episodes_no_last_or_next() {
        let tv = mk_tv("X", None, None);

        let result = to_weekly_episodes(&tv, &vec![123]);

        assert!(result.is_empty());
    }

    #[test]
    fn to_weekly_episodes_only_next() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let next = mk_ep(date, 1, 2);
        let tv = mk_tv("X", None, Some(next));

        let result = to_weekly_episodes(&tv, &vec![123]);

        assert_eq!(
            result,
            vec![WeeklyEpisode {
                series_name: "X".to_string(),
                air_date: date,
                season: 1,
                episode: 2,
            }]
        );
    }

    #[test]
    fn to_weekly_episodes_only_previous() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let last = mk_ep(date, 1, 1);
        let tv = mk_tv("X", Some(last), None);

        let result = to_weekly_episodes(&tv, &vec![123]);

        assert_eq!(
            result,
            vec![WeeklyEpisode {
                series_name: "X".to_string(),
                air_date: date,
                season: 1,
                episode: 1,
            }]
        );
    }

    #[test]
    fn to_weekly_episodes_both() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let last = mk_ep(date, 1, 1);
        let next = mk_ep(date, 1, 2);
        let tv = mk_tv("X", Some(last), Some(next));

        let result = to_weekly_episodes(&tv, &vec![123]);

        assert_eq!(
            result,
            vec![
                WeeklyEpisode {
                    series_name: "X".to_string(),
                    air_date: date,
                    season: 1,
                    episode: 1,
                },
                WeeklyEpisode {
                    series_name: "X".to_string(),
                    air_date: date,
                    season: 1,
                    episode: 2,
                },
            ]
        );
    }

    #[test]
    fn to_weekly_episode_streaming() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let ep = mk_ep(date, 1, 2);
        let tv = mk_tv("X", None, None);

        let result = to_weekly_episode(&tv, &ep, &vec![123]);

        assert_eq!(date, result.air_date);
    }

    #[test]
    fn to_weekly_episode_non_streaming() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let ep = mk_ep(date, 1, 2);
        let tv = mk_tv("X", None, None);

        let result = to_weekly_episode(&tv, &ep, &vec![456]);

        assert_ne!(date, result.air_date);
        assert_eq!(result.air_date - date, Duration::days(1));
    }

    #[test]
    fn summary_all_contains_sorted_series() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let tvs = vec![mk_tv("B", None, None), mk_tv("A", None, None)];

        let summary = Summary::new(date, tvs, &vec![123]);

        assert_eq!(
            summary.all,
            vec![
                SeriesSummary {
                    name: "A".to_string(),
                    last_air: None,
                    next_air: None,
                },
                SeriesSummary {
                    name: "B".to_string(),
                    last_air: None,
                    next_air: None,
                }
            ]
        );
    }

    #[test]
    fn summary_this_week_sorts_within_day() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let tvs = vec![
            mk_tv("B", None, Some(mk_ep(date, 2, 1))),
            mk_tv("A", None, Some(mk_ep(date, 3, 7))),
        ];

        let summary = Summary::new(date, tvs, &vec![123]);

        let (_date, eps) = summary.this_week.iter().exactly_one().unwrap();
        assert_eq!(
            eps,
            &vec![
                WeeklyEpisode {
                    series_name: "A".to_string(),
                    air_date: date,
                    season: 3,
                    episode: 7,
                },
                WeeklyEpisode {
                    series_name: "B".to_string(),
                    air_date: date,
                    season: 2,
                    episode: 1,
                },
            ]
        )
    }

    #[test]
    fn summary_this_week_collects_disordered_dates() {
        let date = Date::from_calendar_date(2022, Month::May, 1).unwrap();
        let other = Date::from_calendar_date(2022, Month::May, 2).unwrap();

        let tvs = vec![
            mk_tv("A", None, Some(mk_ep(date, 3, 7))),
            mk_tv("B", None, Some(mk_ep(other, 2, 1))),
            mk_tv("C", None, Some(mk_ep(date, 2, 6))),
        ];

        let summary = Summary::new(date, tvs, &vec![123]);

        let (_date, eps) = summary.this_week.iter().next().unwrap();
        assert_eq!(
            eps,
            &vec![
                WeeklyEpisode {
                    series_name: "A".to_string(),
                    air_date: date,
                    season: 3,
                    episode: 7,
                },
                WeeklyEpisode {
                    series_name: "C".to_string(),
                    air_date: date,
                    season: 2,
                    episode: 6,
                },
            ]
        )
    }
}
