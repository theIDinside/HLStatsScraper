use super::scrape_config::ScrapeConfig;
use std::path::{Path};
use std::time::Instant;

use crate::data::team::{write_league_to_file, construct_league};

pub fn benchmark_game_info_scraping(config: &ScrapeConfig, tests_per_thread: usize) {
    let run_tests_dir = config.db_asset_dir();
    println!("Run test dir {}", run_tests_dir.display());
    let dbroot = config.db_asset_dir();
    let db_root_dir = dbroot.as_path();
    if !db_root_dir.exists() {
        std::fs::create_dir_all(db_root_dir).expect(&format!("Failed to create directory {}", db_root_dir.to_str().unwrap()));
    }
    match write_league_to_file(construct_league(), run_tests_dir.join("teams.db").as_path()) {
        Ok(bytes_written) => {
            println!("Wrote teams to DB file, {} bytes", bytes_written);
        },
        Err(write_error) => {
            panic!("Failed to write teams db: {}", write_error);
        }
    }

    let (season, season_length, end_date) = config.test_params();
    // (thread_count, slowest, fastest, total, average)
    let mut timings: Vec<(_, _, _, _, f64)> = vec![];
    
    for thread_count in (0 .. 12).rev() {
        let scrape_config = ScrapeConfig::new_with_args(Path::new("benchmark").to_path_buf(), season, season_length, end_date, thread_count + 1);
        scrape_config.display_user_config();
        let mut slowest = 0;
        let mut fastest = 100000000000; 
        let mut total = 0;
        for _ in 0 .. tests_per_thread {
            let scrape_begin_time = Instant::now();
            let _ = crate::scrape::scrape_game_infos(&scrape_config);
            let elapsed = scrape_begin_time.elapsed().as_millis();
            slowest = std::cmp::max(slowest, elapsed);
            fastest = std::cmp::min(fastest, elapsed);
            total += elapsed;
        }
        let avg: f64 = total as f64 / tests_per_thread as f64;
        timings.push((thread_count + 1, slowest, fastest, total, avg));
    }
    println!(" -------------- Results table -------------- ");
    for (thread_count, slow, fast, total, avg) in timings {
        println!("Threads: {:>2} - Slowest: {:>10} - Fastest: {:>10} - Total: {:>30} - Average: {:>14}", thread_count, slow, fast, total, avg);
    }
}

/// Utility wrapper around scraping, so that main() doesn't get cluttered with calls to println!
pub fn benchmark_scrapes(games: &Vec<&crate::data::gameinfo::InternalGameInfo>, scrape_config: &ScrapeConfig) -> Vec<super::ScrapeResults<crate::data::game::Game>> {
    // let game_results = scrape::scrape_game_results(&games, scrape_config);
    super::scrape_game_results_threaded(&games, scrape_config)
}

pub fn benchmark_game_result_scraping(config: &ScrapeConfig, tests_per_thread: usize) {
    let run_tests_dir = config.db_asset_dir();
    println!("Run test dir {}", run_tests_dir.display());
    let dbroot = config.db_asset_dir();
    let db_root_dir = dbroot.as_path();
    if !db_root_dir.exists() {
        std::fs::create_dir_all(db_root_dir).expect(&format!("Failed to create directory {}", db_root_dir.to_str().unwrap()));
    }
    match write_league_to_file(construct_league(), run_tests_dir.join("teams.db").as_path()) {
        Ok(bytes_written) => {
            println!("Wrote teams to DB file, {} bytes", bytes_written);
        },
        Err(write_error) => {
            panic!("Failed to write teams db: {}", write_error);
        }
    }

    let (season, season_length, end_date) = config.test_params();
    // (thread_count, slowest, fastest, total, average)
    let mut timings: Vec<(_, _, _, _, f64)> = vec![];
    
    super::game_info_scrape_all_threaded(config);
    if let Ok(seasonids) = super::verify_game_infos(&config) {
        let refs: Vec<_> = seasonids.iter().map(|x| x).filter(|game| game.date.cmp(config.scrape_until_date()) == std::cmp::Ordering::Less).collect();
        for thread_count in 0 .. 10 {
            println!("Begin scraping benchmark - {} threads. Game info count: {}", thread_count + 1, refs.len());
            let scrape_config = ScrapeConfig::new_with_args(Path::new("benchmark").to_path_buf(), season, season_length, end_date, thread_count + 1);
            scrape_config.display_user_config();
            let mut slowest = 0;
            let mut fastest = 100000000000; 
            let mut total = 0;
            for _ in 0 .. tests_per_thread {
                let scrape_begin_time = Instant::now();
                let _ = benchmark_scrapes(&refs, &scrape_config);
                let elapsed = scrape_begin_time.elapsed().as_millis();
                slowest = std::cmp::max(slowest, elapsed);
                fastest = std::cmp::min(fastest, elapsed);
                total += elapsed;
            }
            let avg: f64 = total as f64 / tests_per_thread as f64;
            timings.push((thread_count + 1, slowest, fastest, total, avg));
        }
    }
    println!(" -------------- Results table -------------- ");
    for (thread_count, slow, fast, total, avg) in timings {
        println!("Threads: {:>2} - Slowest: {:>10} - Fastest: {:>10} - Total: {:>30} - Average: {:>14}", thread_count, slow, fast, total, avg);
    }

}