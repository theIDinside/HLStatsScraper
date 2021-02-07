use std::convert::TryFrom;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use crate::data::team::{get_abbreviated_name};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Time {
    minutes: u8,
    seconds: u8
}

impl Time {
    pub fn new(minutes: u16, seconds: u16) -> Time {
        Time {minutes: minutes as u8, seconds: seconds as u8}
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum Period {
    First(Time),
    Second(Time),
    Third(Time),
    OT(Time),
    SO
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct DeserializePeriod {
    number: usize, 
    time: Time
}

impl Serialize for Period {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer
    {
        let mut state = serializer.serialize_struct("Period", 2)?;
        match self {
            Period::First(t) => {
                state.serialize_field("number", &1)?;
                state.serialize_field("time", &t)?;
            },
            Period::Second(t) => {
                state.serialize_field("number", &2)?;
                state.serialize_field("time", &t)?;
            },
            Period::Third(t) => {
                state.serialize_field("number", &3)?;
                state.serialize_field("time", &t)?;
            },
            Period::OT(t) => {
                state.serialize_field("number", &4)?;
                state.serialize_field("time", &t)?;
            },
            Period::SO => {
                let t = Time::new(0, 0); // Shootout does not have time. So we set it to an "invalid" time, 0:00
                state.serialize_field("number", &5)?;
                state.serialize_field("time", &t)?;
            }
        }
        state.end()
    }
}

impl Period {
    pub fn new(period_string: &str, time: Option<Time>) -> Option<Period> {
        match period_string {
            "1" => Some(Period::First(time.unwrap())),
            "2" => Some(Period::Second(time.unwrap())),
            "3" => Some(Period::Third(time.unwrap())),
            "OT" => Some(Period::OT(time.unwrap())),
            "SO" => Some(Period::SO),
            _ => None
        }
    }
}


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum GoalStrength {
    Even,
    EvenPenaltyShot,
    PenaltyShot,
    EvenEmptyNet,
    PowerPlay,
    ShortHanded,
    ShortHandedEmptyNet,
    ShortHandedPenaltyShot,
    PowerPlayEmptyNet,
    PowerPlayPenaltyShot,
    Shootout
}

impl TryFrom<&String> for GoalStrength {
    type Error = &'static str;
    fn try_from(s: &String) -> Result<Self, Self::Error> {
        type GS = GoalStrength;
        match s.as_ref() {
            "EV"        => Ok(GS::Even),
            "EV-EN"     => Ok(GS::EvenEmptyNet),
            "PP"        => Ok(GS::PowerPlay),
            "EV-PS"     => Ok(GS::EvenPenaltyShot),
            "PS"        => Ok(GS::PenaltyShot),
            "SH"        => Ok(GS::ShortHanded),
            "SH-PS"     => Ok(GS::ShortHandedPenaltyShot),
            "SH-EN"     => Ok(GS::ShortHandedEmptyNet),
            "PP-EN"     => Ok(GS::PowerPlayEmptyNet),
            "PP-PS"     => Ok(GS::PowerPlayPenaltyShot),
            "SO"        => Ok(GS::Shootout),
            _ => Err("Could not parse string into GoalStrength type")
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct FaceOffs {
    pub away: f32,
    pub home: f32
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PowerPlay {
    /// Power play goals made in this game
    pub goals: u32,
    /// Power play attempts in this game
    pub total: u32,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PowerPlays {
    pub away: PowerPlay,
    pub home: PowerPlay
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Goal {
    /// The nth goal scored in game (1-indexed)
    goal_number: u8,
    /// Player name
    player: String,
    /// Scoring team
    pub team: String,
    /// Period & time in period
    period: Period,
    /// Team strengths
    strength: GoalStrength
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeserializeGoal {
    /// The nth goal scored in game (1-indexed)
    goal_number: u8,
    /// Player name
    player: String,
    /// Scoring team
    pub team: String,
    period: DeserializePeriod,
    /// Team strengths
    strength: GoalStrength
}


impl From<DeserializeGoal> for Goal {
    fn from(g: DeserializeGoal) -> Goal {
        Goal {
            goal_number: g.goal_number,
            player: g.player,
            team: g.team,
            period: Period::from(g.period),
            strength: g.strength
        }
    }
}

impl From<DeserializePeriod> for Period {
    fn from(p: DeserializePeriod) -> Period {
        if p.number == 1 {
            Period::First(p.time)
        } else if p.number == 2 {
            Period::Second(p.time)
        } else if p.number == 3 {
            Period::Third(p.time)
        } else if p.number == 4 {
            Period::OT(p.time)
        } else if p.number == 5 {
            Period::SO
        } else {
            panic!("Period number is erroneous.");
        }
    }
}

impl Goal {

    pub fn new_as_opt(goal_number: u8, player: String, team: String, period: Period,strength: GoalStrength) -> Option<Goal> {
        Some(Goal { goal_number, player, team, period,strength })
    }

    pub fn by_team(&self, team_name: &String) -> bool { &self.team == team_name }
}

#[derive(Debug)]
pub struct GoalBuilder {
    goal_number: Option<u8>,
    player: Option<String>,
    team: Option<usize>,
    period: Option<Period>,
    strength: Option<GoalStrength>
}

impl GoalBuilder {
    pub fn new() -> GoalBuilder {
        GoalBuilder {
            goal_number: None, player: None, team: None, period: None, strength: None
        }
    }

    pub fn goal_number(&mut self, number: u8) { self.goal_number = Some(number); }
    pub fn player(&mut self, player: String) { self.player = Some(player); }
    pub fn team(&mut self, team: usize) { self.team = Some(team); }
    pub fn period(&mut self, period: Period) { self.period = Some(period); }
    pub fn strength(&mut self, strength: GoalStrength) { self.strength = Some(strength); }

    pub fn finalize(&self) -> Option<Goal> {
        if let (Some(goal_number), Some(player), Some(team_id), Some(period), Some(strength)) =
        (self.goal_number, self.player.clone(), self.team, self.period, self.strength) {
            if goal_number != 0 {
                let team = get_abbreviated_name(team_id).unwrap();
                Goal::new_as_opt(goal_number, player, team.to_owned(), period, strength)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn is_shootout(&self) -> bool {
        if let Some(period) = self.period {
            match period {
                Period::SO => true,
                _ => false
            }
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PIM {
    pub away: usize,
    pub home: usize
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Score {
    pub away: usize,
    pub home: usize
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Shots {
    pub away: usize,
    pub home: usize,
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TakeAways {
    pub away: usize,
    pub home: usize
}
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GiveAways {
    pub away: usize,
    pub home: usize
}