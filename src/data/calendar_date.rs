use chrono::{Datelike, Utc};
#[derive(Clone, Copy, Debug, Hash, Eq, Deserialize, Serialize)]
pub struct CalendarDate {
    pub year: u32,
    pub month: u32,
    pub day: u32,
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
    pub fn new(day: u32, month: u32, year: u32) -> CalendarDate { CalendarDate { day, month, year } }

    
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
            year: now.year() as u32,
            month: now.month(),
            day: now.day()
        }
    }
}