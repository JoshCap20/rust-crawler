extern crate clap;
extern crate indicatif;
extern crate reqwest;
extern crate scraper;
extern crate url;

use clap::{Arg, App};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;

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

    let progress_bar = create_progress_bar(false, "Crawling...".to_string(), None);

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(parsed_url.clone());

    while let Some(url) = queue.pop_front() {
        if visited.contains(&url) {
            continue;
        }

        progress_bar.set_message(format!("Crawling: {}", url));
        match fetch_and_parse(&url) {
            Ok(links) => {
                for link in links {
                    if !visited.contains(&link) && link.host() == parsed_url.host() {
                        queue.push_back(link);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch {}: {}", url, e);
            }
        }

        visited.insert(url);
    }

    progress_bar.finish_with_message("Crawling complete");
    println!("Visited pages:");
    for url in visited {
        println!("{}", url);
    }
}

fn create_progress_bar(quiet_mode: bool, msg: String, length: Option<u64>) -> ProgressBar {
    let bar = if quiet_mode {
        ProgressBar::hidden()
    } else {
        match length {
            Some(len) => ProgressBar::new(len),
            None => ProgressBar::new_spinner(),
        }
    };

    bar.set_message(msg.clone());
    if let Some(_) = length {
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Failed to set progress style")
                .progress_chars("=> ")
        );
    } else {
        bar.set_style(ProgressStyle::default_spinner());
    }

    bar
}

fn fetch_and_parse(url: &Url) -> Result<HashSet<Url>, Box<dyn std::error::Error>> {
    let body = get(url.as_str())?.text()?;
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