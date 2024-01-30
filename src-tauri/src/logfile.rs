use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{line_ending, space1};
use nom::combinator::{map, map_parser};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

#[derive(Debug, Default)]
pub struct LogfileState {
    pub game_state: Option<GameState>,
    pub game_type: Option<GameType>,
    pub timestamp: Option<String>,
    pub duration: Option<u64>,
    pub map: Option<String>,
    pub win_condition: Option<String>,
    pub left: Option<TeamData>,
    pub right: Option<TeamData>,
    pub player_name: Option<String>,
    pub player_steam_id: Option<String>,
    pub language_code: Option<String>,
}

impl LogfileState {
    fn merge(&mut self, other: LogfileState) {
        if other.game_state.is_some() {
            self.game_state = other.game_state;
        }
        if other.game_type.is_some() {
            self.game_type = other.game_type;
        }
        if other.timestamp.is_some() {
            self.timestamp = other.timestamp;
        }
        if other.duration.is_some() {
            self.duration = other.duration;
        }
        if other.map.is_some() {
            self.map = other.map;
        }
        if other.win_condition.is_some() {
            self.win_condition = other.win_condition;
        }
        if other.left.is_some() {
            self.left = other.left;
        }
        if other.right.is_some() {
            self.right = other.right;
        }
        if other.player_name.is_some() {
            self.player_name = other.player_name;
        }
        if other.player_steam_id.is_some() {
            self.player_steam_id = other.player_steam_id;
        }
        if other.language_code.is_some() {
            self.language_code = other.language_code;
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
pub enum GameType {
    Classic,
    AI,
    Custom,
}

#[derive(Debug)]
pub enum TeamSide {
    Axis,
    Allies,
    Mixed,
}

#[derive(Debug)]
pub struct PlayerData {
    ai: bool,
    faction: String,
    relic_id: String,
    name: String,
    position: u8,
    steam_id: String,
    rank: i64,
}

#[derive(Debug)]
pub struct TeamData {
    players: Vec<PlayerData>,
    side: TeamSide,
}

pub type ParserResult<'a, T> = IResult<&'a str, T>;

pub fn parse_logfile(input: &str) -> ParserResult<LogfileState> {
    fold_many0(
        alt((parse_state_transition, parse_line, parse_blank)),
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
    map(terminated(is_not("\r\n"), line_ending), |_| None)(input)
}

fn parse_blank(input: &str) -> ParserResult<Option<LogfileState>> {
    map(line_ending, |_| None)(input)
}

fn parse_state_transition(input: &str) -> ParserResult<Option<LogfileState>> {
    map_parser(
        terminated(is_not("\r\n"), line_ending),
        map(
            tuple((parse_indicator, parse_timestamp, parse_id, parse_set_state)),
            |(_, _, _, (new, _))| {
                Some(LogfileState {
                    game_state: Some(GameState::from_state(new)),
                    ..Default::default()
                })
            },
        ),
    )(input)
}

fn parse_set_state(input: &str) -> ParserResult<(&str, &str)> {
    map(
        preceded(
            tag("GameApp::SetState : "),
            tuple((
                preceded(tag("new "), delimited(tag("("), take_until(")"), tag(") "))),
                preceded(tag("old "), delimited(tag("("), take_until(")"), tag(")"))),
            )),
        ),
        |(new, old)| (new, old),
    )(input)
}

fn parse_indicator(input: &str) -> ParserResult<&str> {
    terminated(alt((tag("(I)"), tag("(E)"))), space1)(input)
}

fn parse_timestamp(input: &str) -> ParserResult<&str> {
    terminated(delimited(tag("["), take_until("]"), tag("]")), space1)(input)
}

fn parse_id(input: &str) -> ParserResult<usize> {
    map(
        terminated(delimited(tag("["), take_until("]:"), tag("]:")), space1),
        |id: &str| id.parse::<usize>().unwrap(),
    )(input)
}
