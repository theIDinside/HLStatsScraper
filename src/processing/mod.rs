use std::collections::HashMap;
use super::data::gameinfo::InternalGameInfo;
use std::io::{Read};
use std::fs::OpenOptions;

use crate::GAMES_IN_SEASON;

pub type FileResult = Result<String, std::io::Error>;

pub trait FileString {
    fn string_read(&mut self) -> FileResult;
}

pub enum GameInfoScraped {
    All(Vec<InternalGameInfo>),
    Partial(Vec<InternalGameInfo>, Option<Vec<usize>>),
    None(Option<serde_json::Error>)
}

impl From<serde_json::Error> for GameInfoScraped {
    fn from(e: serde_json::Error) -> GameInfoScraped {
        GameInfoScraped::None(Some(e))
    }
}

use crate::ScrapeConfig;

fn check_missing(games: &Vec<InternalGameInfo>, scrape_config: &ScrapeConfig) -> Option<Vec<usize>> {
    let mut missing_games = Vec::new();
    let ids: Vec<usize> = games.iter().map(|x| x.get_id()).collect();

    let ALL_GAMES = scrape_config.season_ids_range();

    for game in ALL_GAMES {
        if !ids.contains(&game) {
            missing_games.push(game);
        }
    }

    if missing_games.len() == 0 {
        None
    } else {
        Some(missing_games)
    }
}

fn verify_deserialized_content(data: &Vec<InternalGameInfo>, scrape_config: &ScrapeConfig) -> bool {
    data.len() == scrape_config.season_games_len()
}

pub fn process_game_infos(scrape_config: &ScrapeConfig) -> GameInfoScraped {
    let db_dir = scrape_config.db_asset_dir();
    assert_eq!(db_dir.exists(), true);
    let compiled_file_name = "gameinfo.db";
    let partials_dir = db_dir.join("gi_partials");
    
    let f = db_dir.join(compiled_file_name);
    let mut gameinfo_buf = String::new();
    let mut gameinfo_file = std::fs::File::open(f).expect("Couldn't open gameinfo.db file");
    gameinfo_file.read_to_string(&mut gameinfo_buf).expect("Couldn't read contents of gameinfo.db file");

    println!("Compiled file size: {}", gameinfo_buf.len());

    if gameinfo_buf.len() >= 2 {
        let contents: Vec<InternalGameInfo> = serde_json::from_str(&gameinfo_buf).expect("Couldn't de-serialize Game Info db");
        if verify_deserialized_content(&contents, scrape_config) {
            return GameInfoScraped::All(contents);
        } else {
            println!("De-serialized content does not match {} games. Found games: {}", GAMES_IN_SEASON, contents.len());
            panic!("Exiting");
        }
    }

    if partials_dir.exists() { // means we haven't compiled a full DB yet.
        println!("\nGame Info partials directory exists... Scanning contents");
        let mut all_partials = Vec::new();
        let mut count = 0;
        for entry in std::fs::read_dir(&partials_dir).expect(format!("Couldn't read directory {} or it's contents", partials_dir.display()).as_ref()) {
            let file_entry = entry.expect("Couldn't unwrap iterator for file");
            let mut buf = String::new();
            let mut f_handle = OpenOptions::new()
                                            .read(true)
                                            .write(false)
                                            .create(false)
                                            .open(file_entry.path())
                                            .expect(format!("Couldn't open file {} for processing", file_entry.path().display()).as_ref());                                            
            let bytes_read = f_handle.read_to_string(&mut buf)
                .expect(format!("Couldn't read file contents of {} to char buffer for processing", file_entry.path().display()).as_ref());
            if bytes_read <= 2 { // means we have EOF or just empty [] json array or empty json object {}
                // skip this file
            } else {
                let partials: Vec<InternalGameInfo> = serde_json::from_str(&buf)
                        .expect(format!("Couldn't de-serialize contents from file {}", file_entry.path().display()).as_ref());
                all_partials.push(partials);
            }
            count += 1;
        }
        if count == 0 {
            return GameInfoScraped::None(None);
        }
        let flattened: Vec<InternalGameInfo> = all_partials.into_iter().flatten().collect();
        let de_duped: HashMap<usize, &InternalGameInfo> = flattened.iter().map(|gi| (gi.get_id(), gi)).collect();
        assert_eq!(flattened.len(), de_duped.len());
        if let Some(missing) = check_missing(&flattened, scrape_config) {
            GameInfoScraped::Partial(flattened, Some(missing))
        } else { // we have _all_ gameinfos saved as partials. Save it to compiled file.
            let mut full = flattened.clone();
            full.sort_by(|a,b| a.get_id().partial_cmp(&b.get_id()).unwrap());
            GameInfoScraped::All(full)
        }
    } else {
        let game_info_full = db_dir.join(compiled_file_name);
        if game_info_full.exists() {
            let mut buf = String::new();
            let mut f_handle = 
                OpenOptions::new().read(true).write(false).create(false).open(&game_info_full)
                    .expect(format!("Couldn't open file {} for processing", game_info_full.display()).as_ref());
            let _bytes_read = f_handle.read_to_string(&mut buf).expect(format!("Couldn't read contents of {}", game_info_full.display()).as_ref());
            if _bytes_read == 0 {
                return GameInfoScraped::None(None)
            }
            let data: Vec<InternalGameInfo> = serde_json::from_str(&buf).expect("Couldn't de-serialize data from Game Info DB file");
            let de_duped_data: HashMap<usize, &InternalGameInfo> = data.iter().map(|gi| (gi.get_id(), gi)).collect();
            assert_eq!(data.len(), de_duped_data.len());

            if let Some(missing_games) = check_missing(&data, scrape_config) {
                GameInfoScraped::Partial(data, Some(missing_games))
            } else {
                let mut full = data.clone();
                full.sort_by(|a,b| a.get_id().partial_cmp(&b.get_id()).unwrap());
                GameInfoScraped::All(full)
            }
        } else {
            GameInfoScraped::None(None)        
        }
    }
}
