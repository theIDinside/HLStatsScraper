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

use getopts::Options;

use std::path::{Path};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read};
use std::time::Instant;
use std::collections::HashMap;
// Threading stuff
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;


use data::{game::{Game, IntermediateGame}, gameinfo::{InternalGameInfo}, team::{construct_league, write_league_to_file}};
use scrape::export::*;
use processing::{GameInfoScraped};

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

/// db_dir is the folder where results_file and info_file should be opened from/created in. <br>
/// Returns a tuple of the opened file handles
fn open_db_files<'a>(db_asset_path: std::path::PathBuf, info_file: &str, results_file: &str) -> (File, File) {
    let db_dir = db_asset_path.as_path();
    let game_info_path = db_dir.join(info_file);
    let game_results_path = db_dir.join(results_file);
    let game_info = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&game_info_path).expect(format!("Couldn't open/create file {}", game_info_path.display()).as_ref());

    let game_results = OpenOptions::new()
        .read(true)
        .write(true)
        .append(false) // we want to _replace_ the contents if we write to it
        .create(true)
        .open(&game_results_path).expect(format!("Couldn't open/create file {}", game_results_path.display()).as_ref());
    (game_info, game_results)
}


/// Purges the DB and writes a new one with the contents of parameter games
/// Returns a result of (bytes written | error message)
pub fn purgewrite_gameresults(db_asset_path: std::path::PathBuf, results_file: &str, games: Vec<&Game>) -> Result<usize, String> {
    let db_dir = db_asset_path.as_path();
    let game_results_path = db_dir.join(results_file);
    let data = serde_json::to_string(&games).expect("Couldn't serialize game results data");
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

use reqwest::{blocking::{Client}};
pub fn game_info_scrape_all_threaded(scrape_config: &ScrapeConfig) {
    if scrape_config.requested_jobs() == 1 {
        game_info_scrape_all(scrape_config);
    } else {
        let season = scrape_config.season_start();
        let games_in_season = scrape_config.season_games_len();
        let count = games_in_season as u64;
        let mut pb = pbr::ProgressBar::new(count);
        pb.format("╢▌▌░╟");

        let (tx, rx): (Sender<ScrapeResults<InternalGameInfo>>, Receiver<ScrapeResults<InternalGameInfo>>) = mpsc::channel();

        println!("Scraping game info for season {} - {} games, using {} threads", season, games_in_season, scrape_config.requested_jobs());
        // Begin scraping of all game info
        let full_season_ids: Vec<usize> = scrape_config.season_ids_range().collect();
        // make sure, we split it up so all chunks get processed. 
        let jobs_size = ((full_season_ids.len() as f64) / (scrape_config.requested_jobs() as f64)).ceil() as usize;
        let game_id_chunks: Vec<Vec<usize>> = full_season_ids.chunks(jobs_size).map(|chunk| {
            chunk.into_iter().map(|v| *v).collect()
        }).collect();

        let mut threads = vec![];
        for job in game_id_chunks {
            let tx_clone = tx.clone();
            let scraper_thread = thread::spawn(move || {
                let client_inner = Client::new();
                let t = tx_clone;
                for id in job {
                    let url_string = format!("{}/{}", scrape::scrape_config::BASE_URL, id);
                    let r = client_inner.get(&url_string).send();
                    if let Ok(resp) = r {
                        let url = resp.url();
                        let g_info_result = InternalGameInfo::from_url(url);
                        match g_info_result {
                            Ok(res) => {
                                t.send(Ok(res)).expect("Channel send error of valid scrape");
                            },
                            Err(e) => {
                                t.send(Err((id, e))).expect("Channel send error of invalid scrape");
                            }
                        }
                    } else if let Err(e) = r {
                        t.send(Err((id, scrape::errors::BuilderError::from(e)))).expect("Channel send error of scraping client error");
                    }
                }
            });
            threads.push(scraper_thread);
        }

        let mut result = vec![];
        while result.len() < count as usize {
            let msg = rx.recv();
            match msg {
                Ok(item) => result.push(item),
                Err(e) => {
                    println!("Channel error - {}", e);
                    result.push(Err((0, scrape::errors::BuilderError::ChannelError)))
                }
            }
            pb.inc();
        }
        pb.finish_print(format!("Done scraping game info for {} games.\n", count).as_ref());
        for t in threads {
            t.join().expect("Thread panicked");
        }
        let (games, _errors) = process_results(&mut result);
        let data = serde_json::to_string(&games).unwrap();
        let file_path = scrape_config.db_asset_dir().join("gameinfo.db");
        let mut info_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_path).expect(format!("Couldn't open/create file {}", &file_path.display()).as_ref());            
        match info_file.write_all(data.as_bytes()) {
            Ok(_) => {
                println!("Successfully wrote {} game infos to file {}. ({} bytes)", games.len(), &file_path.display(), data.len());
            },
            Err(e) => {
                println!("Failed to write serialized data to {}. OS error message: {}", &file_path.display(), e);
                panic!("Exiting");
            }
        }
        

    }
}

// TODO: since games totals for a season has been messed up for covid, we now currently (until next season) use a public static mutable, 
//  that gets set for amount of games, which can be read from, on a project-wide level (PROVIDED_GAMES_IN_SEASON). This will change.
// This is to turn off warning for games_total
pub fn game_info_scrape_all(scrape_config: &ScrapeConfig) {
    let season = scrape_config.season_start();
    let games_in_season = scrape_config.season_games_len();

    println!("Scraping game info for season {} - {} games", season, games_in_season);
    // Begin scraping of all game info
    let full_season_ids: Vec<usize> = scrape_config.season_ids_range().collect();
    println!("There is no saved Game Info data. Begin scraping of Game Info DB of season of {} games", full_season_ids.len());
    // We split the games into 100-game chunks, so if anything goes wrong, we at least write 100 games to disk at a time
    let game_id_chunks: Vec<Vec<usize>> = full_season_ids.chunks(100).map(|chunk| {
        chunk.into_iter().map(|v| *v).collect()
    }).collect();
    assert_eq!(game_id_chunks[0].len(), 100);
    // we scrape and save 100 game infos at a time. that way if something goes wrong, it doesn't go wrong at 1100 games, and then blow up only having to restart
    // possibly could be multi threaded / co-routined using tokio or whatever it's called

    for (index, game_ids) in game_id_chunks.iter().enumerate() {
        println!("Scraping game info for games {}-{}", game_ids[0], game_ids[game_ids.len()-1]);
        let file_name = format!("gameinfo_partial-{}.db", index);
        let file_path = scrape_config.db_asset_dir().join(&file_name);
        let mut result = scrape_game_infos(&game_ids);
        let (games, _errors) = process_results(&mut result);
        for g in &games {
            print!("{}: {:?}", g.get_id(), g.get_date_tuple());
        }
        let data = serde_json::to_string(&games).unwrap();
        let mut info_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_path).expect(format!("Couldn't open/create file {}", &file_path.display()).as_ref());            
        match info_file.write_all(data.as_bytes()) {
            Ok(_) => {
                println!("Successfully wrote {} game infos to file {}. ({} bytes)", games.len(), &file_path.display(), data.len());
            },
            Err(e) => {
                println!("Failed to write serialized data to {}. OS error message: {}", &file_path.display(), e);
                panic!("Exiting");
            }
        }
    }
}

pub fn game_info_scrape_missing(scrape_config: &ScrapeConfig, missing_games: Vec<usize>) -> Vec<InternalGameInfo> {
    println!("Scraping remaining {} game info items", missing_games.len());
    let chunks: Vec<Vec<usize>> = missing_games.chunks(100).map(|chunk| chunk.iter().map(|val| *val).collect()).collect();
    let mut scraped_missing: Vec<Vec<InternalGameInfo>> = Vec::new();
    let partials_dir = scrape_config.db_asset_dir().join("gi_partials");
    let partials_file_count = std::fs::read_dir(&partials_dir).expect("Could not open gi_partials directory").count();
    
    for (index, game_ids) in chunks.iter().enumerate() {
        let file_name = format!("gameinfo_partial-{}.db", index + partials_file_count);
        let file_path = &partials_dir.join(&file_name);
        let mut result = scrape_game_infos(&game_ids);
        let (games, _errors) = process_results(&mut result);
        let data = serde_json::to_string(&game_ids).unwrap();
        let mut info_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&file_path).expect(format!("Couldn't open/create file {}", file_path.display()).as_ref());            
        match info_file.write_all(data.as_bytes()) {
            Ok(_) => {
                println!("Successfully wrote {} game infos to file {}. ({} bytes)", games.len(), file_path.display(), data.len());
            },
            Err(_e) => {}
        }
        scraped_missing.push(games.into_iter().map(|x| x.clone()).collect::<Vec<InternalGameInfo>>());
    }
    scraped_missing.into_iter().flatten().collect()
}

pub fn handle_serde_json_error(err: serde_json::Error) {
    // do stuff with error
    println!("{}", err);
    panic!("De-serializing data failed. Make sure data is in the correct format, or delete all current data and re-scrape everything (Warning, may take a long time)");
}

fn main() {
    let scrape_config = ScrapeConfig::new(std::env::args());
    scrape_config.display_user_config();
    let dbroot = scrape_config.db_asset_dir();
    let db_root_dir = dbroot.as_path();
    if !db_root_dir.exists() {
        std::fs::create_dir_all(db_root_dir).expect(&format!("Failed to create directory {}", db_root_dir.to_str().unwrap()));
    }

    println!("Set database root directory: {}", db_root_dir.display());

    
    match write_league_to_file(construct_league(), scrape_config.db_asset_dir().join("teams.db").as_path()) {
        Ok(bytes_written) => {
            println!("Wrote teams to DB file, {} bytes", bytes_written);
        },
        Err(_write_error) => {

        }
    }
    let (mut game_info_file, mut game_results_file) =
        open_db_files(scrape_config.db_asset_dir(),
                      "gameinfo.db",
                      "gameresults.db");
    
    let game_info_db = processing::scrape_and_process_game_infos(&scrape_config);
    match game_info_db {
        GameInfoScraped::None(None) => {
            println!("No game infos has been scraped... Begin scraping....");
            game_info_scrape_all_threaded(&scrape_config);
            // game_info_scrape_all(&scrape_config);

        },
        GameInfoScraped::None(Some(serde_err)) => {
            // An error occured while trying to de-serialize stored data. we just panic for now
            handle_serde_json_error(serde_err);
        },
        GameInfoScraped::Partial(partial_data, Some(missing_games)) => {
            let missing_scraped = game_info_scrape_missing(&scrape_config, missing_games);
            let total: Vec<&InternalGameInfo> = partial_data.iter().chain(missing_scraped.iter()).collect();
            let de_duplicated: HashMap<usize, &InternalGameInfo> = 
                total.iter().map(|val| (val.get_id(), *val)).collect();

            assert_eq!(de_duplicated.len(), total.len());
            if total.len() == scrape_config.season_games_len() { 
                let data = serde_json::to_string(&total).expect("Could not de-serialize fully compiled Game Info db to file");
                match game_info_file.write_all(data.as_bytes()) {
                    Ok(_) => {
                        println!("Successfully wrote serialized data of fully compiled Game Info db");
                    },
                    Err(e) => {
                        panic!("Failed to write serialized data to disk. Error message returned from OS: {}", e);
                    }
                }
            } else {
                panic!("Still haven't scraped all Game Info objects. Run hockeyscraper again.");
            }

        },
        GameInfoScraped::Partial(all_data_as_partials, None) => { // Means we have all the data, it is just not compiled to a single file yet.
            let data = serde_json::to_string(&all_data_as_partials).expect("Could not de-serialize fully compiled Game Info db to file");
            match game_info_file.write_all(data.as_bytes()) {
                Ok(_) => {
                    println!("Successfully compiled partial data to single file & write to disk!");
                },
                Err(e) => {
                    println!("Could not compile partial data and write to file: {}", e);
                }
            }
        },
        GameInfoScraped::All(mut season) => {
            // We first must make sure all games are sorted by data & game id. 
            // This is so we don't get errors when we try scraping a game with a low Game ID that has been postponed
            season.sort_by(|a, b| a.cmp(b));
            assert_eq!(season.len(), scrape_config.season_games_len());
            println!("All game infos are scraped & serialized to 1 file. Begin scraping of results...");
            let mut buf = String::new();
            match game_results_file.read_to_string(&mut buf) {
                Ok(bytes) => {
                    if bytes <= 2 { // database is empty
                        let refs: Vec<_> = season.iter().map(|x| x).filter(|game| game.date.cmp(scrape_config.scrape_until_date()) == std::cmp::Ordering::Less).collect();
                        println!("Scraping {} game results", refs.len());
                        let result = scrape_and_log(&refs, &scrape_config);
                        let (game_results, _errors) = scrape::process_gr_results(&result);
                        println!("Total game results scraped: {}", &game_results.len());
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
                            let (mut game_results, _errors) = scrape::process_gr_results(&result);
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
                },
                Err(e) => {
                    println!("Could not read game results file: {}", e);
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::scrape::{scrape_game_infos, process_results};
    use std::path::{Path};
    use std::fs::OpenOptions;
    use std::io::Write;

    #[test]
    fn scrape_and_serialize_5_games() {    
        let r = (2019020001 .. (2019020001 + 5)).collect();
        let p = Path::new("./tests/scrape_and_serialize_5_games.db");
        let mut f = OpenOptions::new().write(true).create(true).open(p).expect("Couldn't create file ./tests/scrape_and_serialize_5_games.db");
        let results = scrape_game_infos(&r);
        let (games, _errors) = process_results(&results);
        let data = serde_json::to_string(&games).unwrap();
        match f.write_all(data.as_bytes()) {
            Ok(_) => {
                println!("Successfully wrote serialized data to file");
            },
            Err(e) => {
                panic!("Failed to write serialized data to file.");
            }
        }
    }

    #[test]
    fn deserialize_game() {
        let _data = 
        r#"{"game_info":{"home":"TOR","away":"OTT","gid":2019020001,"date":{"year":2019,"month":10,"day":2}},
            "goals":
                [{"goal_number":1,"player":"7 B.TKACHUK(1)","team":"OTT","period":{"number":1,"time":{"minutes":0,"seconds":25}},"strength":"Even"},
                {"goal_number":2,"player":"33 F.GAUTHIER(1)","team":"TOR","period":{"number":2,"time":{"minutes":2,"seconds":20}},"strength":"Even"},
                {"goal_number":3,"player":"42 T.MOORE(1)","team":"TOR","period":{"number":2,"time":{"minutes":4,"seconds":42}},"strength":"Even"},
                {"goal_number":4,"player":"49 S.SABOURIN(1)","team":"OTT","period":{"number":2,"time":{"minutes":5,"seconds":51}},"strength":"Even"},
                {"goal_number":5,"player":"34 A.MATTHEWS(1)","team":"TOR","period":{"number":2,"time":{"minutes":8,"seconds":2}},"strength":"Even"},
                {"goal_number":6,"player":"34 A.MATTHEWS(2)","team":"TOR","period":{"number":2,"time":{"minutes":14,"seconds":50}},"strength":"PowerPlay"},
                {"goal_number":7,"player":"65 I.MIKHEYEV(1)","team":"TOR","period":{"number":3,"time":{"minutes":9,"seconds":43}},"strength":"Even"},
                {"goal_number":8,"player":"9 B.RYAN(1)","team":"OTT","period":{"number":3,"time":{"minutes":17,"seconds":45}},"strength":"Even"}],
                "winning_team":"TOR",
                "final_score":{"away":3,"home":5},
                "shots":[ {"away":12,"home":14},
                        {"away":3,"home":17},
                        {"away":11,"home":11}],
                "power_plays":{"away":{"goals":0,"total":3},
                                "home":{"goals":1,"total":5}},
                "take_aways":{"away":7,"home":13},
                "give_aways":{"away":8,"home":12},
                "face_offs":{"away":58.0,"home":42.0}}"#;

    }


}