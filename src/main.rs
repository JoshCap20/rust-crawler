extern crate clap;
extern crate indicatif;
extern crate reqwest;
extern crate scraper;
extern crate url;

use clap::{Arg, App};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;
use std::error::Error;

struct Crawler {
    client: Client,
    visited: HashSet<Url>,
    queue: VecDeque<Url>,
    base_host: String,
    progress_bar: ProgressBar,
}

impl Crawler {
    fn new(base_url: Url) -> Result<Self, Box<dyn Error>> {
        let client = Client::builder().build()?;
        let progress_bar = create_progress_bar(false, "Crawling...".to_string(), None);
        Ok(Crawler {
            client,
            visited: HashSet::new(),
            queue: VecDeque::from([base_url.clone()]),
            base_host: base_url.host_str().ok_or("Invalid host")?.to_string(),
            progress_bar,
        })
    }

    fn crawl(&mut self) {
        while let Some(url) = self.queue.pop_front() {
            if self.visited.contains(&url) {
                continue;
            }

            self.progress_bar.set_message(format!("Crawling: {}", url));
            if let Ok(links) = self.fetch_and_parse(&url) {
                for link in links {
                    if !self.visited.contains(&link) && link.host_str() == Some(&self.base_host) {
                        self.queue.push_back(link);
                    }
                }
            }

            self.visited.insert(url);
        }
        self.progress_bar.finish_with_message("Crawling complete");
        println!("Visited pages:");
        for url in &self.visited {
            println!("{}", url);
        }
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

fn main() {
    let matches = App::new("rust-crawler")
        .version("0.1.0")
        .author("Josh Caponigro <jcaponigro20@gmail.com>")
        .about("A simple website crawler")
        .arg(Arg::with_name("URL")
            .required(true)
            .takes_value(true)
            .index(1)
            .help("URL to start crawling"))
        .get_matches();

    let start_url = matches.value_of("URL").unwrap();
    let parsed_url = Url::parse(start_url).expect("Invalid URL");

    if let Ok(mut crawler) = Crawler::new(parsed_url) {
        crawler.crawl();
    }
}
