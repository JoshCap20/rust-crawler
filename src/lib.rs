extern crate indicatif;
extern crate reqwest;
extern crate scraper;
extern crate url;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;
use std::error::Error;

pub struct Crawler {
    client: Client,
    visited: HashSet<Url>,
    queue: VecDeque<Url>,
    base_domain: String,
    progress_bar: ProgressBar,
}

impl Crawler {
    pub fn new(base_url: Url) -> Result<Self, Box<dyn Error>> {
        let client = Client::builder().build()?;
        let progress_bar = create_progress_bar(false, "Crawling...".to_string(), None);
        let base_domain = base_url.domain().ok_or("Invalid domain")?.to_string();
        Ok(Self {
            client,
            visited: HashSet::new(),
            queue: VecDeque::from([base_url]),
            base_domain,
            progress_bar,
        })
    }

    pub fn crawl(&mut self) {
        while let Some(url) = self.queue.pop_front() {
            if self.visited.contains(&url) {
                continue;
            }

            self.progress_bar.set_message(format!("Crawling: {}", url));
            if let Ok(links) = self.fetch_and_parse(&url) {
                for link in links {
                    if !self.visited.contains(&link) && self.is_same_domain(&link) {
                        self.queue.push_back(link);
                    }
                }
            }

            self.visited.insert(url);
        }
        self.progress_bar.finish_with_message("Crawling complete");
        self.display_visited();
    }

    fn fetch_and_parse(&self, url: &Url) -> Result<HashSet<Url>, Box<dyn Error>> {
        let body = self.client.get(url.as_str()).send()?.text()?;
        let document = Html::parse_document(&body);
        let selector = Selector::parse("a").unwrap();
        let mut links = HashSet::new();

        for element in document.select(&selector) {
            if let Some(link) = element.value().attr("href") {
                if let Ok(link_url) = url.join(link) {
                    links.insert(link_url);
                }
            }
        }

        Ok(links)
    }

    fn is_same_domain(&self, url: &Url) -> bool {
        url.domain().map_or(false, |domain| domain == self.base_domain)
    }

    fn display_visited(&self) {
        println!("Visited pages:");
        for url in &self.visited {
            println!("{}", url);
        }
    }
}

fn create_progress_bar(quiet_mode: bool, msg: String, length: Option<u64>) -> ProgressBar {
    let bar = if quiet_mode {
        ProgressBar::hidden()
    } else {
        let mut builder = ProgressBar::new_spinner();
        if let Some(len) = length {
            builder = ProgressBar::new(len);
        }
        builder.set_message(msg.clone());
        builder.set_style(ProgressStyle::default_spinner());
        builder
    };
    bar
}
