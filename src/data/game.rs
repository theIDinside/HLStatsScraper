use super::gameinfo::{InternalGameInfo};
use super::stats::{Score, Goal, Shots, PowerPlays, TakeAways, GiveAways, FaceOffs, DeserializeGoal};
use crate::data::stats::PowerPlay;
use crate::scrape::errors::BuilderError;

/// S.E = self explanatory

/// This is a real fine hack by me. I haven't set out to learn how to implement a custom Deserialize (yet), so for now I deserialize to this, 
/// then convert to struct Game from that. The problem lies with the goals field, as the final Game struct has Goals represented with 
/// the period field as a Period enum. And custom deserializing of enums is a bit tricky, so for now this will do, as the main purpose
/// of this part of the application is the serialization part. The custom serialization for Period is implemented in stats.rs, 
/// the rest is derived using #[derive(Deserialize)]
#[derive(Debug, Serialize, Deserialize)]
pub struct IntermediateGame {
    /// Holds game ID, home team ID, away team ID and CalendarDate for when it was played
    game_info: InternalGameInfo,
    /// Scoring progression
    goals: Vec<DeserializeGoal>,
    /// Winning team's ID
    winning_team: String,
    /// S.E.
    final_score: Score,
    /// Shots by period
    shots: Vec<Shots>,
    /// Power play results for teams in this game
    power_plays: PowerPlays,
    /// S.E.
    take_aways: TakeAways,
    /// S.E.
    give_aways: GiveAways,
    /// Face off win percentages
    face_offs: FaceOffs
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    /// Holds game ID, home team ID, away team ID and CalendarDate for when it was played
    game_info: InternalGameInfo,
    /// Scoring progression
    goals: Vec<Goal>,
    /// Winning team's ID
    winning_team: String,
    /// S.E.
    final_score: Score,
    /// Shots by period
    shots: Vec<Shots>,
    /// Power play results for teams in this game
    power_plays: PowerPlays,
    /// S.E.
    take_aways: TakeAways,
    /// S.E.
    give_aways: GiveAways,
    /// Face off win percentages
    face_offs: FaceOffs
}

impl From<IntermediateGame> for Game {
    fn from(g: IntermediateGame) -> Game {
        Game {
            game_info: g.game_info,
            goals: g.goals.into_iter().map(|g| Goal::from(g)).collect(),
            winning_team: g.winning_team,
            final_score: g.final_score,
            shots: g.shots,
            power_plays: g.power_plays,
            take_aways: g.take_aways,
            give_aways: g.give_aways,
            face_offs: g.face_offs
        }
    }
}

#[allow(dead_code)]
impl Game {
    /// This struct can be optimized. But there's really no point, as it *will* be serialized and not
    /// used in the front-end which is entirely written in C++
    pub fn size_of(&self) -> usize {
        let mut sz = 0usize;
        sz += std::mem::size_of::<InternalGameInfo>();
        sz += self.goals.len() * std::mem::size_of::<Goal>();
        sz += self.winning_team.len() * std::mem::size_of::<char>();
        sz += std::mem::size_of::<Score>();
        sz += self.shots.len() * std::mem::size_of::<Shots>();
        sz += std::mem::size_of::<PowerPlay>() * 2;
        sz += std::mem::size_of::<TakeAways>();
        sz += std::mem::size_of::<GiveAways>();
        sz += std::mem::size_of::<FaceOffs>();
        sz
    }
}

#[derive(Clone, Debug)]
pub struct ValueHolder<T> {
    away: Option<T>,
    home: Option<T>
}

impl ValueHolder<PowerPlay> {
    pub fn as_power_plays(&self) -> Option<PowerPlays> {
        if let (Some(away), Some(home)) = (self.away, self.home) {
            Some(PowerPlays { away, home })
        } else {
            None
        }
    }
}

impl ValueHolder<usize> {
    pub fn as_take_aways(&self) -> Option<TakeAways> {
        if let (Some(away), Some(home)) = (self.away, self.home) {
            Some(TakeAways { away, home })
        } else {
            None
        }
    }

    pub fn as_give_aways(&self) -> Option<GiveAways> {
        if let (Some(away), Some(home)) = (self.away, self.home) {
            Some(GiveAways { away, home })
        } else {
            None
        }
    }

    pub fn as_final_score(&self) -> Option<Score> {
        if let(Some(away), Some(home)) = (self.away, self.home) {
            Some(Score { away, home })
        } else {
            None
        }
    }
}

impl ValueHolder<f32> {
    pub fn as_face_offs(&self) -> Option<FaceOffs> {
        if let (Some(away), Some(home)) = (self.away, self.home) {
            Some(FaceOffs { away, home })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum TeamValue<T> {
    Away(T),
    Home(T)
}

#[derive(Debug)]
pub struct GameBuilder {
    game_info:      Option<InternalGameInfo>,
    goals:          Option<Vec<Goal>>,
    winning_team:   Option<usize>,
    final_score:    ValueHolder<usize>,
    shots:          Option<Vec<Shots>>,
    power_plays:    ValueHolder<PowerPlay>,
    take_aways:     ValueHolder<usize>,
    give_aways:     ValueHolder<usize>,
    face_offs:      ValueHolder<f32>
}

impl GameBuilder {
    pub fn new() -> GameBuilder {
        GameBuilder {
            game_info:      None,
            goals:          None,
            winning_team:   None,
            final_score:    ValueHolder { away: None, home: None },
            shots:          None,
            power_plays:    ValueHolder { away: None, home: None },
            take_aways:     ValueHolder { away: None, home: None },
            give_aways:     ValueHolder { away: None, home: None },
            face_offs:      ValueHolder { away: None, home: None }
        }
    }

    pub fn game_info(&mut self, game_info: InternalGameInfo) {
        self.game_info = Some(game_info);
    }

    pub fn add_goal(&mut self, goal: Goal) {
        if let Some(ref mut goals) = self.goals {
            goals.push(goal);
        } else {
            let mut goals = Vec::new();
            goals.push(goal);
            self.goals = Some(goals);
        }
    }

    pub fn shots(&mut self, shots: Vec<Shots>) {
        self.shots = Some(shots);
    }

    pub fn power_plays(&mut self, power_plays: TeamValue<PowerPlay>) {
        match power_plays {
            TeamValue::Away(away) => self.power_plays.away = Some(away),
            TeamValue::Home(home) => self.power_plays.home = Some(home)
        }
    }

    pub fn take_aways(&mut self, take_aways: TeamValue<usize>) {
        match take_aways {
            TeamValue::Away(away) => self.take_aways.away = Some(away),
            TeamValue::Home(home) => self.take_aways.home = Some(home)
        }
    }

    pub fn give_aways(&mut self, give_aways: TeamValue<usize>) {
        match give_aways {
            TeamValue::Away(away) => self.give_aways.away = Some(away),
            TeamValue::Home(home) => self.give_aways.home = Some(home)
        }
    }

    pub fn face_offs(&mut self, face_offs: TeamValue<f32>) {
        match face_offs {
            TeamValue::Away(away) => self.face_offs.away = Some(away),
            TeamValue::Home(home) => self.face_offs.home = Some(home)
        }
    }

    pub fn set_final_score(&mut self) {
        let home_id = self.game_info.as_ref().unwrap().get_home_team();
        let away_id = self.game_info.as_ref().unwrap().get_away_team();

        let mut home = 0usize;
        let mut away = 0usize;

        if self.goals.is_none() {
            println!("Game had no registered goals... this is an error");
            println!("Game ID: {:?}", self.game_info.as_ref().unwrap());
        }
        
        for goal in self.goals.as_ref().unwrap() {
            if goal.by_team(home_id) {
                home += 1;
            } else if goal.by_team(away_id) {
                away += 1;
            } else {
                panic!(format!("This goal is registered by a team not playing in this game. This is a forced panic (logic error). Away: {} - Home {}. Goal by {}", away_id, home_id, goal.team));
            }
        }
        self.final_score = ValueHolder { away: Some(away), home: Some(home) }
    }

    pub fn finalize(&self) -> Option<Game> {
        let fscore = self.final_score.as_final_score().unwrap().clone();

        let winning_team = if fscore.home > fscore.away {
            self.game_info.as_ref().unwrap().get_home_team().clone()
        } else {
            self.game_info.as_ref().unwrap().get_away_team().clone()
        };

        if let (Some(game_info), Some(goals), Some(final_score), Some(shots),
            Some(power_plays), Some(take_aways), Some(give_aways), Some(face_offs)) =
        (*&self.game_info.as_ref(), &self.goals, *&self.final_score.as_final_score(), &self.shots, *&self.power_plays.as_power_plays(), *&self.take_aways.as_take_aways(), *&self.give_aways.as_give_aways(), *&self.face_offs.as_face_offs()) {
            Some(Game {
                game_info: game_info.clone(),
                goals: goals.clone(),
                winning_team,
                final_score,
                shots: shots.clone(),
                power_plays,
                take_aways,
                give_aways,
                face_offs
            })
        } else {
            None
        }
    }

    pub fn get_error(&self) -> BuilderError {

        let mut err_fields = Vec::new();
        if self.game_info.is_none() {
            err_fields.push(format!("self.game_info not set"));
        }
        if self.goals.is_none() {
            err_fields.push("self.goals not set".to_owned());
        }
        if self.winning_team.is_none() {
            err_fields.push(String::from("self.winning_team not set"));
        }
        if self.final_score.away.is_none() || self.final_score.home.is_none() {
            err_fields.push(format!("self.final_score incorrect: {:?}", self.final_score));
        }
        if self.shots.is_none() {
            err_fields.push(String::from("self.shots not set"));
        }
        if self.power_plays.home.is_none() || self.power_plays.away.is_none() {
            err_fields.push(format!("self.power_plays incorrect: {:?}", self.power_plays));
        }
        if self.take_aways.home.is_none() || self.take_aways.away.is_none() {
            err_fields.push(format!("self.take_aways incorrect: {:?}", self.take_aways));
        }
        if self.give_aways.home.is_none() || self.give_aways.away.is_none() {
            err_fields.push(format!("self.give_aways incorrect: {:?}", self.give_aways));
        }
        if self.face_offs.home.is_none() || self.face_offs.away.is_none() {
            err_fields.push(format!("self.face_offs incorrect: {:?}", self.face_offs));
        }
        BuilderError::GameIncomplete(self.game_info.as_ref().unwrap().get_id(), err_fields)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_deserializing_game() {
        let game_data = r#"{"game_info":{"home":18,"away":23,"gid":2019020001,"date":{"year":2019,"month":10,"day":2}},
            "goals":[{"goal_number":1,"player":"7 B.TKACHUK(1)","team":23,"period":{"First":{"minutes":0,"seconds":25}},"strength":"Even"},
                    {"goal_number":2,"player":"33 F.GAUTHIER(1)","team":18,"period":{"Second":{"minutes":2,"seconds":20}},"strength":"Even"},
                    {"goal_number":3,"player":"42 T.MOORE(1)","team":18,"period":{"Second":{"minutes":4,"seconds":42}},"strength":"Even"},
                    {"goal_number":4,"player":"49 S.SABOURIN(1)","team":23,"period":{"Second":{"minutes":5,"seconds":51}},"strength":"Even"},
                    {"goal_number":5,"player":"34 A.MATTHEWS(1)","team":18,"period":{"Second":{"minutes":8,"seconds":2}},"strength":"Even"},
                    {"goal_number":6,"player":"34 A.MATTHEWS(2)","team":18,"period":{"Second":{"minutes":14,"seconds":50}},"strength":"PowerPlay"},
                    {"goal_number":7,"player":"65 I.MIKHEYEV(1)","team":18,"period":{"Third":{"minutes":9,"seconds":43}},"strength":"Even"},
                    {"goal_number":8,"player":"9 B.RYAN(1)","team":23,"period":{"Third":{"minutes":17,"seconds":45}},"strength":"Even"}],
            "winning_team":18,
            "final_score":{"away":3,"home":5},
            "shots":[{"away":12,"home":14},
                     {"away":3,"home":17},
                     {"away":11,"home":11}],
            "power_plays":{"away":{"goals":0,"total":3},
            "home":{"goals":1,"total":5}},
            "take_aways":{"away":7,"home":13},
            "give_aways":{"away":8,"home":12},
            "face_offs":{"away":58.0,"home":42.0}}"#;
    }
}