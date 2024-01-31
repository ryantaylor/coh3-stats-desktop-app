use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{alpha1, digit1, line_ending, space1};
use nom::combinator::{map, map_parser, map_res, opt, peek};
use nom::multi::{fold_many0, fold_many1, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

#[derive(Debug, Default)]
pub struct LogfileState {
    pub game_state: Option<GameState>,
    pub teams: Option<TeamData>,
    pub player_relic_id: Option<u64>,
}

impl LogfileState {
    fn merge(&mut self, other: LogfileState) {
        if other.game_state.is_some() {
            self.game_state = other.game_state;
        }
        if other.teams.is_some() {
            self.teams = other.teams;
        }
        if other.player_relic_id.is_some() {
            self.player_relic_id = other.player_relic_id;
        }
    }
}

#[derive(Debug)]
pub enum GameState {
    Closed,
    Menu,
    Loading,
    InGame,
}

impl GameState {
    pub fn from_state(state: &str) -> GameState {
        match state {
            "Frontend" => GameState::Menu,
            "LoadingGame" => GameState::Loading,
            "Game" => GameState::InGame,
            _ => GameState::Closed,
        }
    }
}

#[derive(Debug)]
pub struct PlayerData {
    ai: bool,
    faction: String,
    relic_id: Option<u64>,
    name: String,
    position: u8,
    team: u8,
}

#[derive(Debug, Default)]
pub struct TeamData {
    left: Vec<PlayerData>,
    right: Vec<PlayerData>,
}

pub type ParserResult<'a, T> = IResult<&'a str, T>;

pub fn parse_logfile(input: &str) -> ParserResult<LogfileState> {
    fold_many0(
        alt((
            parse_state_transition,
            parse_player_relic_id,
            parse_players,
            parse_line,
        )),
        LogfileState::default,
        |mut acc: LogfileState, item| {
            if let Some(state) = item {
                acc.merge(state);
            }
            acc
        },
    )(input)
}

fn parse_line(input: &str) -> ParserResult<Option<LogfileState>> {
    map(terminated(opt(is_not("\r\n")), line_ending), |_| None)(input)
}

fn parse_state_transition(input: &str) -> ParserResult<Option<LogfileState>> {
    map_parser(
        take_line,
        map(
            preceded(
                tuple((parse_indicator, parse_timestamp, parse_id)),
                parse_set_state,
            ),
            |(new, _)| {
                Some(LogfileState {
                    game_state: Some(GameState::from_state(new)),
                    ..Default::default()
                })
            },
        ),
    )(input)
}

fn parse_set_state(input: &str) -> ParserResult<(&str, &str)> {
    preceded(
        tag("GameApp::SetState : "),
        tuple((
            preceded(tag("new "), delimited(tag("("), take_until(")"), tag(") "))),
            preceded(tag("old "), delimited(tag("("), take_until(")"), tag(")"))),
        )),
    )(input)
}

fn parse_players(input: &str) -> ParserResult<Option<LogfileState>> {
    map(
        fold_many1(parse_player, TeamData::default, |mut acc, item| {
            match item.team {
                0 => acc.left.push(item),
                1 => acc.right.push(item),
                _ => panic!("invalid team {}", item.team),
            }
            acc
        }),
        |teams| {
            Some(LogfileState {
                teams: Some(teams),
                ..Default::default()
            })
        },
    )(input)
}

fn parse_player(input: &str) -> ParserResult<PlayerData> {
    map_parser(
        take_line,
        preceded(
            tuple((parse_indicator, parse_timestamp, parse_id)),
            parse_game_player,
        ),
    )(input)
}

fn parse_game_player(input: &str) -> ParserResult<PlayerData> {
    map(
        preceded(
            tag("GAME -- "),
            tuple((parse_ai, parse_position, parse_player_details)),
        ),
        |(ai, position, (name, relic_id, team, faction))| PlayerData {
            ai,
            position,
            team,
            name,
            relic_id,
            faction,
        },
    )(input)
}

fn parse_ai(input: &str) -> ParserResult<bool> {
    terminated(
        alt((
            map(tag("Human Player"), |_| false),
            map(tag("AI Player"), |_| true),
        )),
        tag(": "),
    )(input)
}

fn parse_position(input: &str) -> ParserResult<u8> {
    map_res(terminated(digit1, space1), str::parse::<u8>)(input)
}

fn parse_player_details(input: &str) -> ParserResult<(String, Option<u64>, u8, String)> {
    map(
        many1(terminated(
            alt((map(peek(tag(" ")), |_| ""), is_not(" "))),
            opt(tag(" ")),
        )),
        |tokens: Vec<&str>| {
            let len = tokens.len();
            (
                tokens[0..(len - 3)]
                    .iter()
                    .map(|token| token.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                tokens[len - 3].parse::<u64>().ok(),
                tokens[len - 2].parse::<u8>().unwrap(),
                tokens[len - 1].to_string(),
            )
        },
    )(input)
}

fn parse_player_relic_id(input: &str) -> ParserResult<Option<LogfileState>> {
    map_parser(
        take_line,
        map(
            preceded(
                tuple((parse_indicator, parse_timestamp, parse_id)),
                parse_message,
            ),
            |player_relic_id| {
                Some(LogfileState {
                    player_relic_id: Some(player_relic_id),
                    ..Default::default()
                })
            },
        ),
    )(input)
}

fn parse_message(input: &str) -> ParserResult<u64> {
    preceded(
        tuple((
            tag("Read bytes ["),
            digit1,
            tag(","),
            delimited(tag("\""), alpha1, tag("\"")),
            tag(","),
        )),
        map_res(digit1, str::parse::<u64>),
    )(input)
}

fn parse_indicator(input: &str) -> ParserResult<&str> {
    terminated(alt((tag("(I)"), tag("(E)"))), space1)(input)
}

fn parse_timestamp(input: &str) -> ParserResult<&str> {
    terminated(delimited(tag("["), take_until("]"), tag("]")), space1)(input)
}

fn parse_id(input: &str) -> ParserResult<usize> {
    terminated(
        delimited(
            tag("["),
            map_res(take_until("]:"), str::parse::<usize>),
            tag("]:"),
        ),
        space1,
    )(input)
}

fn take_line(input: &str) -> ParserResult<&str> {
    terminated(is_not("\r\n"), line_ending)(input)
}

#[cfg(test)]
mod tests {
    use super::parse_logfile;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_normal_logfile() {
        let mut file = File::open("test/fixtures/warnings.log.test").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let str = String::from_utf8_lossy(&buf);

        let results = parse_logfile(&str);
        assert!(results.is_ok())
    }

    #[test]
    fn test_spaces_in_player_name() {
        let mut file = File::open("test/fixtures/warnings_space_in_name.log.test").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let str = String::from_utf8_lossy(&buf);

        let results = parse_logfile(&str);
        assert!(results.is_ok())
    }

    #[test]
    fn test_problem_log_1() {
        let mut file = File::open("test/fixtures/warnings-1.log.test").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let str = String::from_utf8_lossy(&buf);

        let results = parse_logfile(&str);
        assert!(results.is_ok())
    }

    #[test]
    fn test_problem_log_2() {
        let mut file = File::open("test/fixtures/warnings-2.log.test").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let str = String::from_utf8_lossy(&buf);

        let results = parse_logfile(&str);
        assert!(results.is_ok())
    }

    #[test]
    fn test_problem_log_4() {
        let mut file = File::open("test/fixtures/warnings-4.log.test").unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        let str = String::from_utf8_lossy(&buf);

        let results = parse_logfile(&str);
        assert!(results.is_ok())
    }
}
