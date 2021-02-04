use std::path::PathBuf;
use crate::{print_usage, setup_opts};
use crate::data::calendar_date;
pub const GAMES_IN_SEASON: usize = 1271;
pub const BASE_URL: &'static str = "https://www.nhl.com/gamecenter";
use std::convert::TryFrom;

/// if year=2020, returns like 2020020001
pub fn first_game_id_of_season(year_season_begins: usize) -> usize {   
    1000000 * year_season_begins + 20001
}

const DB_DIR: &'static str = "assets/db";

#[derive(Clone)]
pub struct ScrapeConfig {
    root_directory: PathBuf,
    season: usize, /// The year the season "started" (or was supposed to start, in this case)
    season_length: usize,
    end_date: calendar_date::CalendarDate
}

impl ScrapeConfig {
    pub fn new(args_: std::env::Args) -> ScrapeConfig {
        let args: Vec<String> = args_.collect();
        let parameter_handler = setup_opts();
        let matches = match parameter_handler.parse(&args[1..]) {
            Ok(m) => m,
            Err(_) => {
                print_usage(&args[0], &parameter_handler);
            }
        };      
    
        if matches.opt_present("h") {
            print_usage(&args[0], &parameter_handler);
        }
    
        if matches.opt_count("d") != 1 && matches.opt_count("y") != 1 {
            print_usage(&args[0], &parameter_handler);
        }
    
        let year = matches.opt_str("y");
        let root_dir = matches.opt_str("d").unwrap();
        let games = matches.opt_str("g").map(|g| g.parse::<usize>().ok()).unwrap();
        let end_date = matches.opt_str("e").map(|date_opt| {
            match calendar_date::CalendarDate::try_from(date_opt) {
                Ok(d) => {
                    d
                },
                Err(e) => {
                    println!("Error: {}", e);
                    print_usage(&args[0], &parameter_handler);
                }
            }
        }).unwrap_or(calendar_date::CalendarDate::get_date_from_os()); // this branch, will only happen if a date _wasn't_ provided, as, if an error occurs when parsing the date, will cause the application to quit
        ScrapeConfig {
            root_directory: PathBuf::from(&root_dir),
            season: year.unwrap().parse::<usize>().unwrap(),
            season_length: games.unwrap_or(GAMES_IN_SEASON),
            end_date
        }
    }

    pub fn display_user_config(&self) {
        println!("Configuration set to: \nRoot directory {}\nSeason: {}\nGames in season: {}\nScrap until date {}", self.root_directory.display(), self.season, self.season_games_len(), self.end_date);
    }

    pub fn db_asset_dir(&self) -> PathBuf {
        self.root_directory.join(DB_DIR)
    }

    pub fn season_ids_range(&self) -> std::ops::Range<usize> {
        let begin = first_game_id_of_season(self.season_start());
        let end = first_game_id_of_season(self.season_start()) + self.season_games_len();
        begin .. end
    }

    pub fn season_start(&self) -> usize { self.season }
    pub fn season_games_len(&self) -> usize { self.season_length }
    pub fn scrape_until_date(&self) -> &calendar_date::CalendarDate { &self.end_date }
}