use reqwest::Url;
use crate::scrape::{convert_fwd_slashes, _BASE};
use crate::scrape::errors::BuilderError;
use super::calendar_date::CalendarDate;
/// The actual GameInfo object that we send to the client "over the wire".
#[derive(Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct GameInfo {
    home_team:  String,
    away_team:  String,
    game_id:    usize,
    date:       CalendarDate,
}

impl PartialEq for GameInfo {
    fn eq(&self, other: &Self) -> bool {
        self.game_id == other.game_id
    }
}

/// The data type used for our "In memory" database. It only uses unsigned integers
/// because that will save a lot of memory, instead of having each of the 1270 game info objects
/// contain 2 heap allocated strings, we now only contain 6 usize numbers, and everything can be
/// stack allocated, or behind 1 heap allocation (in a vector)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InternalGameInfo {
    home:  String,
    away:  String,
    pub gid:   usize,
    pub date:  CalendarDate
}

impl Eq for InternalGameInfo {}

impl std::cmp::Ord for InternalGameInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        let gid_cmp = self.gid.cmp(&other.gid);
        let date_cmp = self.date.cmp(&other.date);
        match gid_cmp {
            Ordering::Less if date_cmp == Ordering::Greater => {
                Ordering::Greater
            },
            Ordering::Greater if date_cmp == Ordering::Less => {
                Ordering::Less
            }
            _ => gid_cmp,
        }
    }
}

impl PartialEq for InternalGameInfo {
    fn eq(&self, other: &Self) -> bool {
        self.gid == other.gid
    }
}

impl PartialOrd for InternalGameInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn format_season_summary(season: usize) -> usize {
    season * 1000000
}

impl InternalGameInfo {

    pub fn from_url(url: &Url) -> Result<InternalGameInfo, BuilderError> {

        let url_string = url.clone().into_string();
        // url = "https://www.nhl.com/gamecenter/ari-vs-van/2020/01/16/2019020739#game=2019020739,game_state=final"
        let (_, data) = url_string.split_at(_BASE.len());
        // data = "ari-vs-van/2020/01/16/2019020739#game=2019020739,game_state=final"
        //               pos=^    r_pos=^
        if let Some(pos) = data.find('/') {
            let (teams_str, date_id_info) = data.split_at(pos);
            // teams_str = ari-vs-van
            // date_id_info = /2020/01/16/2019020739#game=2019020739,game_state=final
            if let Some(r_pos) = date_id_info.rfind('/') {
                let (date_tmp,game_id_tmp) = date_id_info.split_at(r_pos);
                // date_tmp = /2020/01/16
                // game_id_tmp = /2019020739#game=2019020739,game_state=final
                let date = &date_tmp[1..];          // we need to remove the first / or -
                let game_id = &game_id_tmp[1..];    // ...
                let teams: Vec<&str> = teams_str.split("-vs-").collect();
                let away_team = teams[0].to_string().to_uppercase();
                let home_team = teams[1].to_string().to_uppercase();
                let date_string = date.chars().into_iter().map(convert_fwd_slashes).collect::<String>();
                let date_fields: Vec<u32> = date_string.split("-").into_iter().map(|field| field.parse::<u32>().unwrap()).collect::<Vec<u32>>().into_iter().rev().collect();
                let date = CalendarDate::new(date_fields[0] as u8, date_fields[1] as u8, date_fields[2] as u16);
                let game_id = game_id.parse::<usize>().ok();
                Ok(InternalGameInfo::new(home_team, away_team, game_id.unwrap(), date))
            } else {
                Err(BuilderError::WrongURLStringFormat(data.to_owned()))
            }
        } else {
            Err(BuilderError::WrongURLStringFormat(data.to_owned()))
        }
    }

    pub fn new(home_team: String, away_team: String, game_id: usize, date: CalendarDate) -> InternalGameInfo {
        InternalGameInfo {
            home: home_team,
            away: away_team,
            gid: game_id,
            date
        }
    }

    pub fn make_clone(&self) -> InternalGameInfo {
        let InternalGameInfo{home, away, gid, date} = self;
        InternalGameInfo {
            home: home.clone(),
            away: away.clone(),
            gid: *gid,
            date: date.clone()
        }
    }

    pub fn get_id(&self) -> usize { self.gid }

    pub fn get_event_summary_url(&self, season: usize) -> String {
        format!("http://www.nhl.com/scores/htmlreports/{}{}/ES0{}.HTM", season, season + 1, self.gid - format_season_summary(season))
    }

    pub fn get_game_summary_url(&self, season: usize) -> String {  
        format!("http://www.nhl.com/scores/htmlreports/{}{}/GS0{}.HTM",  season, season + 1, self.gid - format_season_summary(season))
    }

    pub fn get_shot_summary_url(&self, season: usize) -> String {
        format!("http://www.nhl.com/scores/htmlreports/{}{}/SS0{}.HTM",  season, season + 1, self.gid - format_season_summary(season)) 
    }

    pub fn get_home_team(&self) -> &String { &self.home }
    pub fn get_away_team(&self) -> &String { &self.away }
}