extern crate reqwest;       // used for scraping, for retrieving the actual HTML data
extern crate serde;         // deserialization of json data
extern crate serde_json;    // deserialization of json data
extern crate scraper;       // used for scraping
extern crate select;        // used for scraping
extern crate getopts;       // for getting command line options
extern crate chrono;        // for getting time and date - will eventually be replaced by custom library call to C, which will be OS specific for Linux and Windows. Because fuck Mac.
extern crate num_cpus;      // for determining max cpus
#[macro_use] extern crate serde_derive;
extern crate pbr;           // progress bar in the terminal/console

mod data;
mod scrape;
mod processing;

use std::fs::{OpenOptions};
use std::io::{Write, Read};
use std::time::Instant;
use std::collections::HashMap;

// Threading stuff

use data::{game::{Game, IntermediateGame}, gameinfo::{InternalGameInfo}, team::{construct_league, write_league_to_file}};
use scrape::export::*;

impl processing::FileString for std::fs::File {
    fn string_read(&mut self) -> processing::FileResult {
        let buf_sz = self.metadata().expect("Failed to take metadata").len();
        let mut buf = String::new();
        buf.reserve(buf_sz as usize);
        self.read_to_string(&mut buf)?;
        Ok(buf)
    }
}
/// Utility wrapper around scraping, so that main() doesn't get cluttered with calls to println!
fn scrape_and_log(games: &Vec<&InternalGameInfo>, scrape_config: &ScrapeConfig) -> Vec<ScrapeResults<Game>> {
    println!("Beginning scraping of {} games", games.len());
    let scrape_begin_time = Instant::now();
    // let game_results = scrape::scrape_game_results(&games, scrape_config);
    let game_results = scrape::scrape_game_results_threaded(&games, scrape_config);
    println!("Scraped {} game results", game_results.len());
    let scrape_time_elapsed = scrape_begin_time.elapsed().as_millis();
    println!("Tried scraping {} games in {}ms", game_results.len(), scrape_time_elapsed);
    game_results
}

/// Purges the DB and writes a new one with the contents of parameter games
/// Returns a result of (bytes written | error message)
pub fn purgewrite_gameresults(db_asset_path: std::path::PathBuf, results_file: &str, games: Vec<&Game>) -> Result<usize, String> {
    let db_dir = db_asset_path.as_path();
    let game_results_path = db_dir.join(results_file);
    let deduplicated: HashMap<usize, &Game> = games.iter().map(|g| (g.game_info.gid, *g)).collect();
    let mut final_write: Vec<&Game> = deduplicated.iter().map(|(_, val)| *val).collect();
    final_write.sort_by(|a, b| a.game_info.gid.cmp(&b.game_info.gid));
    let data = serde_json::to_string(&final_write).expect("Couldn't serialize game results data");
    if let Err(e) = std::fs::remove_file(&game_results_path) {
        println!("Removing of file failed: {}", e);
        panic!("Exiting");
    }
    let mut game_results = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&game_results_path).expect(format!("Couldn't open/create file {}", game_results_path.display()).as_ref());

    match game_results.write_all(data.as_bytes()) {
        Ok(_) => {
            println!("Successfully wrote serialized data to file");
            Ok(data.len())
        },
        Err(e) => {
            println!("Could not write serialized data to file. Error: {}", e);
            panic!("Exiting");
        }
    }
}

pub fn handle_serde_json_error(err: serde_json::Error) {
    // do stuff with error
    println!("{}", err);
    panic!("De-serializing data failed. Make sure data is in the correct format, or delete all current data and re-scrape everything (Warning, may take a long time)");
}

fn main() {
    let params_handler = scrape::scrape_config::setup_opts();
    let args: Vec<String> = std::env::args().collect();
    let scrape_config = ScrapeConfig::new(&args, &params_handler);
    scrape_config.display_user_config();
    let dbroot = scrape_config.db_asset_dir();
    let db_root_dir = dbroot.as_path();
    if !db_root_dir.exists() {
        std::fs::create_dir_all(db_root_dir).expect(&format!("Failed to create directory {}", db_root_dir.to_str().unwrap()));
    }

    match write_league_to_file(construct_league(), scrape_config.db_asset_dir().join("teams.db").as_path()) {
        Ok(bytes_written) => {
            println!("Wrote teams to DB file, {} bytes", bytes_written);
        },
        Err(write_error) => {
            panic!("Failed to write teams db: {}", write_error);
        }
    }
    
    if !scrape_config.db_asset_dir().join("gameinfo.db").exists() {
        scrape::game_info_scrape_all_threaded(&scrape_config);
    }

    if scrape_config.db_asset_dir().join("gameinfo.db").exists() {
        let season = verify_game_infos(&scrape_config).expect("Could not de-serialize content from GameInfo DB");
        let game_results_path = scrape_config.db_asset_dir().join("gameresults.db");
        let mut game_results_file = OpenOptions::new().read(true).write(true).append(false).create(true).open(&game_results_path).expect(format!("Couldn't open/create file {}", game_results_path.display()).as_ref());
        let mut buf = String::new();
        let bytes_read = game_results_file.read_to_string(&mut buf).expect("Could not read game results file.");
        if bytes_read <= 2 { // database is empty
            let mut refs: Vec<_> = season.iter().map(|x| x).filter(|game| game.date.cmp(scrape_config.scrape_until_date()) == std::cmp::Ordering::Less).collect();
            refs.sort_by(|a, b| a.cmp(b));
            println!("Scraping {} game results", refs.len());
            let result = scrape_and_log(&refs, &scrape_config);
            let (game_results, _errors) = scrape::process_gr_results(&result);
            let postponed = _errors.iter().fold(0, |acc, item| {
                let (_, err) = item;
                match err {
                    scrape::errors::BuilderError::GamePostponed => {
                        acc + 1
                    },
                    _ => acc
                }
            });
            println!("Total game results scraped: {}. Scrape errors: {} ({} soft errors, i.e. game was postponed)", &game_results.len(), _errors.len(), postponed);
            assert_eq!(game_results.len(), refs.len() - postponed);
            let data = serde_json::to_string(&game_results).expect("Couldn't serialize game results data");
            match game_results_file.write_all(data.as_bytes()) {
                Ok(_) => {
                    println!("Successfully wrote serialized data to file");
                },
                Err(e) => {
                    println!("Could not write serialized data to file. Error: {}", e);
                    panic!("Exiting");
                }
            }
        } else { //
            let data: Vec<IntermediateGame> = serde_json::from_str(&buf).expect("Couldn't de-serialize data for Game results");
            let games: Vec<Game> = data.into_iter().map(|im_game| Game::from(im_game)).collect();
            // we need this, to be able to append the game results, as they are returned as a vector of references
            let mut games_ref: Vec<&Game> = games.iter().map(|x| x).collect(); 
            let refs: Vec<_> = season.iter().map(|x| x).filter(|game| game.date.cmp(scrape_config.scrape_until_date()) == std::cmp::Ordering::Less).collect();
            if games.len() < refs.len() {
                println!("{} games have not been scraped. Scraping remaining", refs.len() - games.len());
                let remaining: Vec<_> = refs.into_iter().skip(games.len()).collect();
                let result = scrape_and_log(&remaining, &scrape_config);
                let (mut game_results, errors) = scrape::process_gr_results(&result);
                let postponed = errors.iter().fold(0, |acc, item| {
                    let (_, err) = item;
                    match err {
                        scrape::errors::BuilderError::GamePostponed => {
                            acc + 1
                        },
                        _ => acc
                    }
                });
                println!("Total game results scraped: {}. Scrape errors: {} ({} soft errors, i.e. game was postponed)", &game_results.len(), errors.len(), postponed);
                let scraped = game_results.len();
                println!("Total game results scraped: {}", &scraped);
                games_ref.append(&mut game_results);
                match purgewrite_gameresults(scrape_config.db_asset_dir(), "gameresults.db", games_ref) {
                    Ok(bytes_written) => {
                        println!("Successfully serialized database ({} bytes written)", bytes_written);
                    },
                    Err(err_msg) => {
                        println!("Serialization failed: {}", err_msg);
                    }
                }
            }
            println!("Games de-serialized: {}. No more games to scrape from this regular season.", games.len());        
        }
    }
}