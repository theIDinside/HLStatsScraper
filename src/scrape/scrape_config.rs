use std::path::PathBuf;
use getopts::Options;
use crate::data::calendar_date;
pub const GAMES_IN_SEASON: usize = 1271;
pub const BASE_URL: &'static str = "https://www.nhl.com/gamecenter";
use std::convert::TryFrom;

/// if year=2020, returns like 2020020001
pub fn first_game_id_of_season(year_season_begins: usize) -> usize {   
    1000000 * year_season_begins + 20001
}

const DB_DIR: &'static str = "assets/db";


/// Prints usage and exits (the ! means we never exit, but actually we do, by calling process::exit. This makes this function usable in match arms where we must return a value, where otherwise the compiler would
/// call bullshit, and say we don't return a value, usable in an Err(e) arm for instance, where we don't want to panic! but we want to exit)
fn print_usage(executable_name: &str, opts: &Options) -> ! {
    let help_message = format!("Usage: {} [options]", executable_name);
    println!("{}", opts.usage(&help_message));
    std::process::exit(0);
}

pub fn setup_opts() -> Options {
    let mut opts = Options::new();
    opts.reqopt("d", "dir", "directory where assets/db and assets/db/gi_partials exist, absolute or relative. (required)", "PATH")
        .reqopt("y", "season", "the year, which the season you want to scrape started (required)", "YEAR")
        .optopt("g", "games", "If the season is not of standard length, pass the amount of games via this parameter (optional)", "GAMES_AMOUNT")
        .optopt("h", "help", "Help message for data stats scraper.", "This help message")
        .optopt("j", "jobs", "Set how many threads you want to use while scraping. If 'all' is provided, tool will try establishing cores on cpu and use that", "<#NUM_JOBS | all>")
        .optopt("e", "end", "Scrape up until this date. If no date is provided, the tool will try to establish today's date.", "DATE (in the form yyyymmdd");
    opts
}

#[derive(Clone)]
pub struct ScrapeConfig {
    root_directory: PathBuf,
    season: usize, /// The year the season "started" (or was supposed to start, in this case)
    season_length: usize,
    end_date: calendar_date::CalendarDate,
    jobs: usize
}

impl ScrapeConfig {
    pub fn new(args_: std::env::Args) -> ScrapeConfig {
        let args: Vec<String> = args_.collect();
        let parameter_handler = setup_opts();
        let matches = match parameter_handler.parse(&args[1..]) {
            Ok(m) => m,
            Err(fail) => {
                println!("{}", fail);
                print_usage(&args[0], &parameter_handler);
            }
        };      
        
        if matches.opt_present("h") || (matches.opt_count("d") != 1 && matches.opt_count("y") != 1) {
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

        let jobs = matches.opt_str("j").map(|param| {
            let tmp = param.to_ascii_lowercase();
            if tmp == "all" {
                num_cpus::get()
            } else {
                match tmp.parse::<usize>() {
                    Ok(num) => num,
                    Err(e) => print_usage(&args[0], &parameter_handler)
                }
            }
        }).unwrap_or(1);

        ScrapeConfig {
            root_directory: PathBuf::from(&root_dir),
            season: year.unwrap().parse::<usize>().unwrap(),
            season_length: games.unwrap_or(GAMES_IN_SEASON),
            end_date,
            jobs
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
    pub fn requested_jobs(&self) -> usize { self.jobs }
}