use std::path::PathBuf;
use crate::{print_usage, setup_opts};
pub const GAMES_IN_SEASON: usize = 1271;
pub const BASE_URL: &'static str = "https://www.nhl.com/gamecenter";

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

        ScrapeConfig {
            root_directory: PathBuf::from(&root_dir),
            season: year.unwrap().parse::<usize>().unwrap(),
            season_length: games.unwrap_or(GAMES_IN_SEASON)
        }
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
}