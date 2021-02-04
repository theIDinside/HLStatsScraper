use chrono::{Datelike, Utc};
use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Clone, Copy, Debug, Hash, Eq, Deserialize, Serialize)]
pub struct CalendarDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl PartialEq for CalendarDate {
    fn eq(&self, other: &Self) -> bool {
        self.day == other.day && self.month == other.month && self.year == other.year
    }
}

impl std::cmp::Ord for CalendarDate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self.year < other.year {
            return Ordering::Less;
        } else {
            if self.year == other.year {
                if self.month < other.month {
                    return Ordering::Less;
                } else {
                    if self.month == other.month {
                        if self.day < other.day {
                            return Ordering::Less;
                        } else if self.day == other.day {
                            return Ordering::Equal;
                        } else {
                            return Ordering::Greater;
                        }
                    } else {
                        return Ordering::Greater;
                    }
                }
            } else {
                return Ordering::Greater;
            }
        }
    }
}

impl PartialOrd for CalendarDate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


// Used as a key in the Server.schedule hashmap. This key/hash is used to retrieve a HashSet
// of weak references to all GameInfo objects which are played that day, defined by this key/hash
impl CalendarDate {
    pub fn new(day: u8, month: u8, year: u16) -> CalendarDate { CalendarDate { year, month, day } }

    
    /// returns today's date. This must be implemented differently depending on the OS
    #[cfg(target_os = "linux")]
    pub fn get_date_from_os() -> CalendarDate {
        let now = Utc::now();
        CalendarDate {
            year: now.year() as u32,
            month: now.month(),
            day: now.day()
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_date_from_os() -> CalendarDate {
        let now = Utc::now();
        CalendarDate {
            year: now.year() as u16,
            month: now.month() as u8,
            day: now.day() as u8
        }
    }
}

impl TryFrom<String> for CalendarDate {
    type Error = String;
    fn try_from(str: String) -> Result<Self, Self::Error> {
        let year_parse = str[0..4].parse::<u16>();
        let month_parse = str[4..6].parse::<u8>();
        let day_parse = str[6..].parse::<u8>();

        if let (Ok(year), Ok(month), Ok(day)) = (year_parse, month_parse, day_parse) {
            Ok( CalendarDate { year, month, day } )
        } else {
            Err(format!("Parsing of date string failed, must be in format yyyymmdd - provided value was: {}", str))
        }
    }
}

impl Display for CalendarDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //let date_string = format!();
        write!(f, "{}-{:0>2}-{:0>2}", self.year, self.month, self.day)
    }
}