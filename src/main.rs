extern crate clap;
extern crate rust_crawler;
extern crate url;

use clap::{Arg, App};
use rust_crawler::Crawler;
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

    if let Ok(mut crawler) = Crawler::new(parsed_url) {
        crawler.crawl();
    }
}
