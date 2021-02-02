pub mod game;
pub mod gameinfo;
pub mod stats;
pub mod team;

pub mod types {
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

    // Used as a key in the Server.schedule hashmap. This key/hash is used to retrieve a HashSet
// of weak references to all GameInfo objects which are played that day, defined by this key/hash
    impl CalendarDate {
        pub fn new(day: u32, month: u32, year: u32) -> CalendarDate { CalendarDate { day, month, year } }
    }
}