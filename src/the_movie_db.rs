use futures::{stream, Future, StreamExt, TryFutureExt, TryStreamExt};
use serde::Deserialize;
use time::Date;

const BASE_URL: &str = "https://api.themoviedb.org/3";
time::serde::format_description!(date, Date, "[year]-[month]-[day]");

pub(crate) struct Client {
    http: reqwest::Client,
    api_key: String,
}

impl Client {
    pub(crate) fn new(api_key: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
        }
    }

    pub(crate) async fn get_tvs(&self, ids: Vec<u32>) -> Result<Vec<TV>, reqwest::Error> {
        stream::iter(ids)
            .map(|x| self.get_tv_future(x))
            .buffered(2)
            .try_collect()
            .await
    }

    fn get_tv_future(&self, id: u32) -> impl Future<Output = Result<TV, reqwest::Error>> {
        let url = format!("{}/tv/{}", BASE_URL, id);
        self.http
            .get(url)
            .bearer_auth(&self.api_key)
            .send()
            .and_then(|r| r.json())
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub(crate) struct TV {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) last_episode_to_air: Option<Episode>,
    pub(crate) next_episode_to_air: Option<Episode>,
    pub(crate) networks: Vec<Network>,
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub(crate) struct Episode {
    #[serde(with = "date")]
    pub(crate) air_date: Date,
    pub(crate) episode_number: u32,
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) overview: String,
    pub(crate) season_number: u32,
    pub(crate) runtime: Option<u32>,
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub(crate) struct Network {
    pub(crate) id: u32,
    pub(crate) name: String,
}
