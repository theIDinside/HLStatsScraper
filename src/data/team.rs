use std::path::Path;
use std::fs::OpenOptions;
use std::io::{Write};

#[derive(Debug, Deserialize, Serialize)]
pub struct Team {
    id: usize,
    name: String,
    abbreviations: Vec<String>
}

pub fn construct_league() -> Vec<Team> {
    let teams = (1u8 .. 32).map(|value| {
        let id = value as usize;
        let name = get_team_name(value as usize).expect(format!("Couldn't get team name with id {}", value).as_ref()).to_owned();
        let abbreviations: Vec<String> =
            get_team_abbreviations(id)
                .expect(format!("Couldn't get abbreviations for team with id {}", value).as_ref())
                .into_iter().map(|s| s.to_owned()).collect();

        Team {
            id, name, abbreviations
        }
    }).collect();
    teams
}

pub fn write_league_to_file(teams: Vec<Team>, file_path: &Path) -> Result<usize, std::io::Error> {
    assert_eq!(teams.len(), 31);
    let mut file = OpenOptions::new().truncate(true).write(true).create(true).open(file_path).expect(&format!("Couldn't open or create file {}", file_path.display()));
    let serialized_data = serde_json::to_string(&teams).expect("Couldn't serialize team data");
    file.write_all(serialized_data.as_bytes())?;
    Ok(serialized_data.len())
}

/// Returns all abbreviations used by the NHL for team with id team_id. <br>
/// Returns an option of vector of string references, Option<Vec<&str>>
pub fn get_team_abbreviations(team_id: usize) -> Option<Vec<&'static str>> {
    match team_id {
        1   => Some(vec!["ANA"]),
        2   => Some(vec!["DET"]),
        3   => Some(vec!["COL"]),
        4   => Some(vec!["SJS", "S.J"]),
        5   => Some(vec!["WSH"]),
        6   => Some(vec!["VGK"]),
        7   => Some(vec!["MIN"]),
        8   => Some(vec!["DAL"]),
        9   => Some(vec!["EDM"]),
        10  => Some(vec!["VAN"]),
        11  => Some(vec!["NYR"]),
        12  => Some(vec!["NYI"]),
        13  => Some(vec!["NJD", "N.J"]),
        14  => Some(vec!["BOS"]),
        15  => Some(vec!["BUF"]),
        16  => Some(vec!["NSH"]),
        17  => Some(vec!["CGY", "CAL"]),
        18  => Some(vec!["TOR"]),
        19  => Some(vec!["ARI"]),
        20  => Some(vec!["WPG"]),
        21  => Some(vec!["CBJ"]),
        22  => Some(vec!["LAK", "L.A"]),
        23  => Some(vec!["OTT"]),
        24  => Some(vec!["MTL"]),
        25  => Some(vec!["CHI"]),
        26  => Some(vec!["PHI"]),
        27  => Some(vec!["PIT"]),
        28  => Some(vec!["CAR"]),
        29  => Some(vec!["TBL", "T.B"]),
        30  => Some(vec!["FLO", "FLA"]),
        31  => Some(vec!["STL"]),
        _ => None
    }
}
/// Returns ID of team with name or abbreviation of parameter team_name of type &String <br>
/// Returns an option of usize, of the team ID: Option<usize>
pub fn get_id(team_name: &String) -> Option<usize> {
    match team_name.to_uppercase().as_ref() {
        "ANAHEIM DUCKS" | "ANA"                         => Some(1),
        "DETROIT RED WINGS" | "DET"                     => Some(2),
        "COLORADO AVALANCHE" | "COL"                    => Some(3),
        "SAN JOSE SHARKS" | "SJS" | "S.J"               => Some(4),
        "WASHINGTON CAPITALS" | "WSH"                   => Some(5),
        "VEGAS KNIGHTS" | "VEGAS GOLDEN KNIGHTS" |"VGK" => Some(6),
        "MINNESOTA WILD" | "MIN"                        => Some(7),
        "DALLAS STARS" | "DAL"                          => Some(8),
        "EDMONTON OILERS" | "EDM"                       => Some(9),
        "VANCOUVER CANUCKS" | "VAN"                     => Some(10),
        "NEW YORK RANGERS" | "NYR"                      => Some(11),
        "NEW YORK ISLANDERS" | "NYI"                    => Some(12),
        "NEW JERSEY DEVILS" | "NJD" | "N.J"             => Some(13),
        "BOSTON BRUINS" | "BOS"                         => Some(14),
        "BUFFALO SABRES" | "BUF"                        => Some(15),
        "NASHVILLE PREDATORS" | "NSH"                   => Some(16),
        "CALGARY FLAMES" | "CGY" | "CAL"                => Some(17),
        "TORONTO MAPLE LEAFS" | "TOR"                   => Some(18),
        "ARIZONA COYOTES" | "ARI"                       => Some(19),
        "WINNIPEG JETS" | "WPG"                         => Some(20),
        "COLUMBUS BLUE JACKETS" | "CBJ"                 => Some(21),
        "LOS ANGELES KINGS" | "LAK" | "L.A"             => Some(22),
        "OTTAWA SENATORS" | "OTT"                       => Some(23),
        "MONTREAL CANADIENS" | "MTL"                    => Some(24),
        "CHICAGO BLACKHAWKS" | "CHI"                    => Some(25),
        "PHILADELPHIA FLYERS" | "PHI"                   => Some(26),
        "PITTSBURGH PENGUINS" | "PIT"                   => Some(27),
        "CAROLINA HURRICANES" | "CAR"                   => Some(28),
        "TAMPA BAY LIGHTNING" | "TBL" | "T.B"           => Some(29),
        "FLORIDA PANTHERS" | "FLO" | "FLA"              => Some(30),
        "ST LOUIS BLUES" | "STL"                        => Some(31),
        _ => None
    }
}

/// Returns full name of team with ID team_id. Returns an option of a static string.
pub fn get_team_name(team_id: usize) -> Option<&'static str> {
    match team_id {
        1   => Some("ANAHEIM DUCKS"),
        2   => Some("DETROIT RED WINGS"),
        3   => Some("COLORADO AVALANCHE"),
        4   => Some("SAN JOSE SHARKS"),
        5   => Some("WASHINGTON CAPITALS"),
        6   => Some("VEGAS GOLDEN KNIGHTS"),
        7   => Some("MINNESOTA WILD"),
        8   => Some("DALLAS STARS"),
        9   => Some("EDMONTON OILERS"),
        10  => Some("VANCOUVER CANUCKS"),
        11  => Some("NEW YORK RANGERS"),
        12  => Some("NEW YORK ISLANDERS"),
        13  => Some("NEW JERSEY DEVILS"),
        14  => Some("BOSTON BRUINS"),
        15  => Some("BUFFALO SABRES"),
        16  => Some("NASHVILLE PREDATORS"),
        17  => Some("CALGARY FLAMES"),
        18  => Some("TORONTO MAPLE LEAFS"),
        19  => Some("ARIZONA COYOTES"),
        20  => Some("WINNIPEG JETS"),
        21  => Some("COLUMBUS BLUE JACKETS"),
        22  => Some("LOS ANGELES KINGS"),
        23  => Some("OTTAWA SENATORS"),
        24  => Some("MONTREAL CANADIENS"),
        25  => Some("CHICAGO BLACKHAWKS"),
        26  => Some("PHILADELPHIA FLYERS"),
        27  => Some("PITTSBURGH PENGUINS"),
        28  => Some("CAROLINA HURRICANES"),
        29  => Some("TAMPA BAY LIGHTNING"),
        30  => Some("FLORIDA PANTHERS"),
        31  => Some("ST LOUIS BLUES"),
        _ => None
    }
}

/// Returns full name of team with ID team_id. Returns an option of a static string.
pub fn get_abbreviated_name(team_id: usize) -> Option<&'static str> {
    match team_id {
        1   => Some("ANA"),
        2   => Some("DET"),
        3   => Some("COL"),
        4   => Some("SJS"),
        5   => Some("WSH"),
        6   => Some("VGK"),
        7   => Some("MIN"),
        8   => Some("DAL"),
        9   => Some("EDM"),
        10  => Some("VAN"),
        11  => Some("NYR"),
        12  => Some("NYI"),
        13  => Some("NJD"),
        14  => Some("BOS"),
        15  => Some("BUF"),
        16  => Some("NSH"),
        17  => Some("CGY"),
        18  => Some("TOR"),
        19  => Some("ARI"),
        20  => Some("WPG"),
        21  => Some("CBJ"),
        22  => Some("LAK"),
        23  => Some("OTT"),
        24  => Some("MTL"),
        25  => Some("CHI"),
        26  => Some("PHI"),
        27  => Some("PIT"),
        28  => Some("CAR"),
        29  => Some("TBL"),
        30  => Some("FLA"),
        31  => Some("STL"),
        _ => None
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn league_construction() {
        let teams = construct_league();
        assert_eq!(teams.len(), 31);
        for (index, team) in teams.iter().enumerate() {
            assert_eq!(index + 1, team.id);
            assert_eq!(get_id(&team.name).unwrap(), team.id);
            assert_eq!(get_team_name(team.id).unwrap(), team.name);
        }
    }

    #[test]
    fn check_teamnames_and_ids() {
        let id_range = 1..32usize; // Rust ranges are end-excl
        for id in id_range {
            let abbreviations = get_team_abbreviations(id).expect("No team with that id");
            for abbr in abbreviations {
                assert_eq!(get_id(&abbr.to_owned()).unwrap(), id);
            }
        }
    }

    #[test]
    fn serializing_of_teams() {
        let path = Path::new("./tmp");
        let _ = std::fs::create_dir_all(path);
        let time = Instant::now();
        let tmp_file_name = path.join(format!("{}.teams.tmp", time.elapsed().as_nanos()));
        let teams = construct_league();
        match write_league_to_file(teams, &tmp_file_name) {
            Ok(_) => {
                let _ = std::fs::remove_file(tmp_file_name);
                let _ = std::fs::remove_dir(path);
            },
            _ => panic!("")
        }
    }

    #[test]
    fn deserialize_teams() {
        let path = Path::new("./tmp");
        match std::fs::create_dir_all(path) {
            Ok(_) => {},
            Err(e) => {}
        }
        let time = Instant::now();
        let tmp_file_name = path.join(format!("{}.teams.tmp", time.elapsed().as_nanos()));
        let t = construct_league();

        match write_league_to_file(t, &tmp_file_name) {
            Ok(_) => {},
            _ => panic!("")
        };
        let mut teams_file = File::open(tmp_file_name.clone()).expect("Couldn't open file for testing");
        let mut data = String::new();
        match teams_file.read_to_string(&mut data) {
            Ok(bytes) => {},
            Err(e) => {}
        }

        let teams: Vec<Team> = serde_json::from_str(&data).expect("Couldn't deserialize teams");

        let team_count = teams.len();
        let t = construct_league();
        match write_league_to_file(t, &tmp_file_name) {
            Ok(bytes) => {},
            Err(e) => {}
        }

        let mut teams_file2 = File::open(tmp_file_name).expect("Couldn't open file for testing");
        let mut data2 = String::new();
        let _ = teams_file2.read_to_string(&mut data2);
        let teams2: Vec<Team> = serde_json::from_str(&data2).expect("Couldn't deserialize teams");

        assert_eq!(teams2.len(), teams.len());

        assert_eq!(31, teams.len());
        for (index, team) in teams.iter().enumerate() {
            assert_eq!(index + 1, team.id);
            assert_eq!(get_id(&team.name).unwrap(), team.id);
            assert_eq!(get_team_name(team.id).unwrap(), team.name);
        }


    }

}