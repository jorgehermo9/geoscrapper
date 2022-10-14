use tokio::time::Duration;

use tokio::time::Instant;

use reqwest::Client;
use scraper::{Html, Selector};

const BASE_URL: &str = "https://en.wikipedia.org";
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
// Request delay to fullfill wikipedia policy
const REQ_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Country {
    name: String,
}

impl Country {
    pub async fn get_all() -> Result<Vec<Country>, reqwest::Error> {
        let client = Client::builder().user_agent(APP_USER_AGENT).build()?;
        let body = client
            .get(format!("{BASE_URL}/wiki/List_of_sovereign_states"))
            .send()
            .await?
            .text()
            .await?;
        let document = Html::parse_document(&body);
        let country_table_selector = Selector::parse("table.sortable").unwrap();
        let country_table = document.select(&country_table_selector).next().unwrap();

        // Ugly as fuck selector. It will break with minor wikipedia changes...
        let country_cell_selector = Selector::parse("tr > td > b > a").unwrap();

        let countries_iter = country_table
            .select(&country_cell_selector)
            .into_iter()
            .map(|country| country.value().attr("href").unwrap().to_string());

        let mut countries = Vec::new();
        let mut previous_req = Instant::now();
        for country_url in countries_iter {
            let country = Country::new(&client, format!("{BASE_URL}/{country_url}")).await?;
            countries.push(country);
            // 1 request per second
            if previous_req.elapsed() < REQ_DELAY {
                let next_req = previous_req + REQ_DELAY;
                tokio::time::sleep_until(next_req).await;
            }
            previous_req = Instant::now();
        }
        // let countries = join_all(countries_iter)
        //     .await
        //     .into_iter()
        //     .filter_map(|country| country.ok())
        //     .collect();

        Ok(countries)
    }
    pub async fn new(client: &Client, url: String) -> Result<Country, reqwest::Error> {
        let body = client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&body);

        // The main title of the page is the country name
        let selector = Selector::parse(".mw-page-title-main").unwrap();
        let country_name = document
            .select(&selector)
            .into_iter()
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()[0]
            .to_string();
        dbg!(&country_name);
        Ok(Country { name: country_name })
    }
}
