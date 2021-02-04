pub mod errors;
pub mod scrape_config;

pub mod export {
    pub use super::ScrapeResults;
    pub use super::scrape_game_infos;
    pub use super::process_results;
    pub use super::scrape_config::first_game_id_of_season;
    pub use super::scrape_config::ScrapeConfig;
}



use reqwest::{blocking::{Client}};
use crate::data::gameinfo::{InternalGameInfo};
use crate::data::game::{Game, GameBuilder, TeamValue};
use crate::data::stats::{GoalBuilder, Period, Time, GoalStrength, PowerPlay, Shots};
use crate::data::team::{get_id};

use std::convert::TryFrom;
use crate::scrape::errors::BuilderError;

pub type ScrapeResults<T> = Result<T, (usize, BuilderError)>;
pub type GameResult = Result<Game, BuilderError>;

pub const _BASE: &'static str = "https://www.nhl.com/gamecenter/";
pub const _VS: &'static str = "-vs-";

pub fn convert_fwd_slashes(ch: char) -> char {
    if ch == '/' {
        '-'
    } else {
        ch
    }
}

/// Returns tuple of PowerPlay stats from game, where the returning tuple contains (away, home) stats
fn process_pp_summary(penalty_summary: &Vec<String>) -> (TeamValue<PowerPlay>, TeamValue<PowerPlay>) {

    let (a_pps, _) = penalty_summary[0].split_at(penalty_summary[0].find("/").expect("Data for Power plays is not formatted correctly"));
    let (h_pps, _) = penalty_summary[1].split_at(penalty_summary[1].find("/").expect("Data for Power plays is not formatted correctly"));

    let a_data = a_pps.split("-").collect::<Vec<&str>>();
    let h_data = h_pps.split("-").collect::<Vec<&str>>();
    let (a_goals, a_total) =
        (a_data[0].parse::<u32>().expect(format!("Could not parse PP goals (Away): {}", a_data[0]).as_ref()),
         a_data[1].parse::<u32>().expect(format!("Could not parse PP total (Away): {}", a_data[1]).as_ref()));
    let (h_goals, h_total) =
        (h_data[0].parse::<u32>().expect(format!("Could not parse PP goals (Home): {}", h_data[0]).as_ref()),
         h_data[1].parse::<u32>().expect(format!("Could not parse PP total (Home): {}", h_data[1]).as_ref()));
    let away_pp = TeamValue::Away(PowerPlay { goals: a_goals, total: a_total });
    let home_pp = TeamValue::Home(PowerPlay { goals: h_goals, total: h_total });
    (away_pp, home_pp)
}

fn scrape_game(client: &reqwest::blocking::Client, game_info: &InternalGameInfo, scrape_config: &scrape_config::ScrapeConfig) -> GameResult {
        use select::document::Document;
        use select::predicate::{Class, Name, And, Attr};
        let season = scrape_config.season_start();
        let gs_url = game_info.get_game_summary_url(season);
        let evt_url = game_info.get_event_summary_url(season);
        let sh_url = game_info.get_shot_summary_url(season);
        // println!("Urls for game {}\n\t{} \n\t{}\n\t{}", game_info.get_id(), gs_url, evt_url, sh_url);
        let (gs, evt, sh) = (client.get(&gs_url).send(),
                           client.get(&evt_url).send(),
                           client.get(&sh_url).send());
        
        let (game_html_data, event_html_data, shots_html_data) =
        match (gs, evt, sh) {
            (Ok(a), Ok(b), Ok(c)) => {
                let (a_status, b_status, c_status) = (a.status(), b.status(), c.status());
                if a_status == reqwest::StatusCode::OK && b_status == reqwest::StatusCode::OK && c_status == reqwest::StatusCode::OK {
                    (a.text().unwrap(), b.text().unwrap(), c.text().unwrap())
                } else {
                    println!("Game {} was postponed", game_info.get_id());
                    return Err(BuilderError::GamePostponed);
                }
            },
            (Err(e), _, _) => {
                return Err(BuilderError::REQWEST(e));
            },
            (_, Err(e), _) => {
                return Err(BuilderError::REQWEST(e));
            },
            (_, _, Err(e)) => {
                return Err(BuilderError::REQWEST(e));
            },
        };

        let evt_doc = Document::from(event_html_data.as_ref());
        let game_doc = Document::from(game_html_data.as_ref());
        let shots_doc = Document::from(shots_html_data.as_ref());
        let evt_pred = And(Class("bold"), Name("tr"));
        let gs_table_predicate = Name("table");

        let mut gb = GameBuilder::new();
        gb.game_info(game_info.clone());

        let nodes: Vec<select::node::Node> = evt_doc.find(evt_pred).filter(|n|{
            n.text().contains("TEAM TOTALS")
        }).map(|x| {
            x.clone()
        }).collect();

        for (index, node) in nodes.iter().enumerate() {
            node.find(Name("td")).enumerate().skip(1).for_each(|(i, stat)| {
                match i {
                    // 1 if index == 0 => gb.final_score(TeamValue::Away(stat.text().parse::<usize>().expect("Couldn't parse away score"))),
                    // 1 if index == 1 => gb.final_score(TeamValue::Home(stat.text().parse::<usize>().expect("Couldn't parse away score"))),
                    5 =>    {},                                            // PN (Number of Penalties)
                    6 =>    {},                                            // PIM (Penalty Infraction Minutes)
                    13 =>   {},                                           // Shots
                    17 if index == 0 => gb.give_aways(TeamValue::Away(stat.text().parse::<usize>().expect(&format!("Could not parse give aways for away team. Game id: {}", game_info.get_id())))),   // GV Give aways
                    17 if index == 1 => gb.give_aways(TeamValue::Home(stat.text().parse::<usize>().expect("Could not parse give aways for home team"))),
                    18 if index == 0 => gb.take_aways(TeamValue::Away(stat.text().parse::<usize>().expect("Could not parse give aways for away team"))),
                    18 if index == 1 => gb.take_aways(TeamValue::Home(stat.text().parse::<usize>().expect("Could not parse give aways for home team"))),
                    22 if index == 0 => gb.face_offs(TeamValue::Away(stat.text().parse::<f32>().expect("Couldn't parse face off value for away team"))), // Faceoff win %
                    22 if index == 1 => gb.face_offs(TeamValue::Home(stat.text().parse::<f32>().expect("Couldn't parse face off value for home team"))),
                    // F% Faceoff win percentage
                    _ =>    {}
                }
            })
        }
     game_doc.find(gs_table_predicate).enumerate().for_each(|(i, node)| {
            match i {
                /* Goal summary table */
                9 => {
                    node.find(Name("tr")).enumerate().for_each(|(idx, tr_node)| {
                        let mut goal_builder = GoalBuilder::new();
                        if idx > 0 {
                            let mut period = String::new();

                            tr_node.find(Name("td")).enumerate().for_each(|(td_index, goal_node)| {
                                let nodestr = goal_node.text().trim().to_owned();
                                match td_index {
                                    0 => {
                                        if nodestr != "-" {
                                            goal_builder.goal_number(nodestr.parse::<usize>().expect("Could not parse goal number"))
                                        } else { // Means we have an unsuccessful penalty shot. Set number => 0 and handle later
                                            goal_builder.goal_number(0);
                                        }
                                    },
                                    1 => { period = nodestr; },
                                    2 => {
                                        if period == "SO" {
                                            let p = Period::new(&period, None).expect("Could not parse period");
                                            goal_builder.period(p);
                                        } else {
                                            let time_components: Vec<&str> = nodestr.split(":").collect();
                                            let (min, sec) = (time_components[0].parse::<u16>().expect(format!("Could not parse minutes: {}", nodestr).as_ref()), time_components[1].parse::<u16>().expect("Could not parse seconds"));
                                            let time = Time::new(min, sec);
                                            let p = Period::new(&period, Some(time)).expect("Could not parse period");
                                            goal_builder.period(p)
                                        }
                                    },
                                    3 => {
                                        if !goal_builder.is_shootout() {
                                            let strength = GoalStrength::try_from(&nodestr).ok().expect("Could not parse strength");
                                            goal_builder.strength(strength);
                                        } else {
                                            goal_builder.strength(GoalStrength::Shootout);
                                        }
                                    },
                                    4 => {
                                        let team_id = get_id(&nodestr).expect(format!("Could not find a team with that name: {}", &nodestr).as_ref());
                                        goal_builder.team(team_id);
                                    },
                                    5 => {
                                        goal_builder.player(nodestr);
                                    },
                                    _ => {}
                                }
                            });
                            if let Some(goal) = goal_builder.finalize() {
                                gb.add_goal(goal);
                            } else {
                                /*
                                if goal_builder.is_unsuccessful_ps() {
                                    println!("'Goal' stat was recorded for an unsuccessful penalty shot. Discarding data.");
                                } else {
                                    println!("Error in goal builder. Data: {:?}", &goal_builder);
                                    panic!("Could not add goal stat");
                                }
                                */
                            }
                        }
                    });
                },
                _ => {}
            }
        });
        gb.set_final_score();
        let mut penalty_summary = vec![String::from("TOT (PN-PIM)")];
        game_doc.find(Attr("id", "PenaltySummary")).for_each(|v| {
            let s = v.text();
            let mut totals = 0;
            let collected: Vec<&str> = s.lines().rev().take_while(|line| {
                if line.contains("TOT") {
                    totals += 1;
                    if totals == 2 {
                        return false;
                    }
                }
                return true;
            }).collect();

            let values: Vec<&str> = collected.into_iter().rev().filter(|str| {
                str.len() >= 2 && *str != "\u{a0}"
            }).collect();

            for item in values {
                penalty_summary.push(item.to_owned());
            }
        });

        let indicator = "Power Plays (Goals-Opp./PPTime)";
        let mut indices = vec![];

        for (index, item) in penalty_summary.iter().enumerate() {
            if item == indicator {
                indices.push(index+1);
            }
        }

        let data = indices.iter().map(|index| {
            penalty_summary.get(*index).unwrap().clone()
        }).collect();

        // println!("{:?}", penalty_summary);
        // println!("Penalty summary: {:?}", penalty_summary);

        let (away_pp, home_pp) = process_pp_summary(&data);
        gb.power_plays(away_pp);
        gb.power_plays(home_pp);

        let shots_table = shots_doc.find(Attr("id", "ShotsSummary")).next().unwrap();
        let mut home_shots = Vec::new();
        let mut away_shots = Vec::new();
        shots_table.find(Name("table")).enumerate().for_each(|(table_index, node)| {
            if table_index == 3 {
                let data = node.text();
                let s: Vec<&str> = data.lines().filter(|line| {
                    line.len() != 0
                }).collect();
                let mut periods = Vec::new();
                for row in s.chunks(5).skip(1) {
                    periods.push(row[4].trim());
                }
                periods.pop(); // Make sure we remove the TOTALS row
                for (_, shot_stats) in periods.iter().enumerate() {
                    if shot_stats.is_empty() {
                        away_shots.push(0);
                    } else {
                        // println!("shot_stats: {}", shot_stats);
                        let (_goals, shots) = shot_stats.split_at(shot_stats.find("-").unwrap());
                        let _shots = shots[1..].parse::<usize>().expect("Couldn't parse shots");
                        away_shots.push(_shots);
                    }
                }
            } else if table_index == 8 {
                let data = node.text();
                let s: Vec<&str> = data.lines().filter(|line| {
                    line.len() != 0
                }).collect();
                let mut periods = Vec::new();
                for row in s.chunks(5).skip(1) {
                    periods.push(row[4].trim());
                }
                periods.pop(); // Make sure we remove the TOTALS row
                for (_period_index, shot_stats) in periods.iter().enumerate() {
                    if shot_stats.is_empty() {
                        home_shots.push(0);
                    } else {
                        let (_goals, shots) = shot_stats.split_at(shot_stats.find("-").unwrap());
                        let _shots = shots[1..].parse::<usize>().expect("Couldn't parse shots");
                        home_shots.push(_shots);
                    }
                }
            }
        });

        let shots: Vec<Shots> = away_shots.into_iter().zip(home_shots.into_iter()).map(|(away, home)| {
            Shots { away, home }
        }).collect();
        gb.shots(shots);
        let g_res = gb.finalize();

        if let Some(game) = g_res {
            Ok(game)
        } else {
            Err(gb.get_error())
            // Err(BuilderError::GameIncomplete(game_info.get_id(), vec!["Some field not parsed / added".to_owned()]))
        }
}

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

pub fn scrape_game_results_threaded(games: &Vec<&InternalGameInfo>, scrape_config: &scrape_config::ScrapeConfig) -> Vec<ScrapeResults<Game>> {
    
    use pbr::ProgressBar;
    // returns a vector of tuple of two links, one to the game summary and one to the event summary
    let mut result = Vec::new();
    let mut pb = ProgressBar::new(games.len() as u64);
    let (tx, rx): (Sender<ScrapeResults<Game>>, Receiver<ScrapeResults<Game>>) = mpsc::channel();
    let amount = games.len();
    let divisor = if amount < 8 {
        1
    } else if amount < 16 {
        2
    } else  {
        4
    };
    println!("Running game scraping in {} threads", divisor);
    let mut jobs: Vec<Vec<InternalGameInfo>> = vec![];
    let chunks = games.chunks_exact(divisor);
    let jobs_per_thread = games.len() / divisor;
    for chunk in chunks {
        let mut v = vec![];
        for item in chunk {
            v.push(item.make_clone());
        }
        jobs.push(v);
    }

    for item in games.iter().skip(jobs_per_thread * divisor) {
        jobs[divisor-1].push(item.make_clone());
    }
    let mut totals_assert = 0;
    for j in &jobs {
        totals_assert += j.len();
    }
    assert_eq!(totals_assert, games.len());
    
    pb.format("╢▌▌░╟");
    let mut workers = vec![];
    for jobs_range in jobs {
        let tx_clone = tx.clone();
        // let games = jobs_range.clone();
        let cfg = scrape_config.clone();
        
        let scraper_thread = thread::spawn(move || {
            let client_inner = Client::new();
            let scrape_config = cfg;
            let t = tx_clone;
            for game_info in jobs_range {
                let res = scrape_game(&client_inner, &game_info, &scrape_config);
                match res {
                    Ok(game) => {
                        t.send(Ok(game)).expect("Channel TX error");
                    },
                    Err(e) => {
                        t.send(Err((game_info.get_id(), e))).expect("Channel TX error");
                    }
                }
            }
        });
        workers.push(scraper_thread);
    }

    while result.len() < games.len() {
        let item = rx.recv();
        match item {
            Ok(res) => {
                result.push(res);
            },
            Err(e) => {
                panic!("Channel error: {}", e);
            }
        }
        pb.inc();
    }
    for worker in workers {
        worker.join().expect("Thread panicked!");
    }

    pb.finish_print(format!("Done scraping game results for {} games.", games.len()).as_ref());
    result
}

pub fn scrape_game_results(games: &Vec<&InternalGameInfo>, scrape_config: &scrape_config::ScrapeConfig) -> Vec<ScrapeResults<Game>> {
    println!("Running game scraping...");
    use pbr::ProgressBar;
    // returns a vector of tuple of two links, one to the game summary and one to the event summary
    let mut result = Vec::new();
    let client = Client::new();
    let mut pb = ProgressBar::new(games.len() as u64);
    
    for game_info in games {
        let res = scrape_game(&client, &game_info, scrape_config);
        match res {
            Ok(game) => {
                result.push(Ok(game));
            },
            Err(e) => {
                result.push(Err((game_info.get_id(), e)));
            }
        }
        pb.inc();
    }
    pb.finish_print(format!("Done scraping game results for {} games.", games.len()).as_ref());
    result
}

pub fn scrape_game_infos(game_ids: &Vec<usize>) -> Vec<ScrapeResults<InternalGameInfo>> {
    // Rust ranges are end-exclusive. So we have to add 1
    use pbr::ProgressBar;
    let count = game_ids.len() as u64;
    let mut pb = ProgressBar::new(count);
    pb.format("╢▌▌░╟");
    let mut result = Vec::new();
    let client = Client::new();
    for id in game_ids {
        let url_string = format!("{}/{}", scrape_config::BASE_URL, id);
        let r = client.get(&url_string).send();
        if let Ok(resp) = r {
            let url = resp.url();
            let g_info_result = InternalGameInfo::from_url(url);
            match g_info_result {
                Ok(res) => {
                    result.push(Ok(res))
                },
                Err(e) => {
                    result.push(Err((*id, e)))
                }
            }
        } else if let Err(e) = r {
            result.push(Err((*id, BuilderError::from(e))));
        }
        pb.inc();
    }
    pb.finish_print(format!("Done scraping game info for {} games. Sorting", count).as_ref());
    result
}

/// Returns a tuple of "ok" scrapes and error's
/// The "ok" scrapes, are sorted by Game ID & CalendarDate. Since some games can be postponed, they are sorted by CalendarDate then GameID
pub fn process_results(results: &mut Vec<ScrapeResults<InternalGameInfo>>) -> (Vec<&InternalGameInfo>, Vec<&(usize, BuilderError)>)
{
    let errors: Vec<&(usize, BuilderError)> = results.iter().filter_map(|f| f.as_ref().err()).collect();
    let mut games: Vec<&InternalGameInfo> = results.iter().filter_map(|f| {
        f.as_ref().ok()
    }).collect();
    games.sort_by(|a, b| {
        a.cmp(b)
    });
    (games, errors)
}

pub fn process_gr_results(results: &Vec<ScrapeResults<Game>>) -> (Vec<&Game>, Vec<&(usize, BuilderError)>) {
    let errors: Vec<&(usize, BuilderError)> = results.iter().filter_map(|f| f.as_ref().err()).collect();
    let games: Vec<&Game> = results.iter().filter_map(|f| {
        f.as_ref().ok()
    }).collect();
    (games, errors)
}

#[cfg(test)]
mod tests {

}