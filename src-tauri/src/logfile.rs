use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{digit1, line_ending, space1};
use nom::combinator::{map, map_parser, opt};
use nom::multi::{fold_many0, fold_many1, many1};
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
    pub teams: Option<TeamData>,
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
        if other.teams.is_some() {
            self.teams = other.teams;
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
pub struct PlayerData {
    ai: bool,
    faction: String,
    relic_id: u64,
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
        alt((parse_state_transition, parse_players, parse_line)),
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
        terminated(is_not("\r\n"), line_ending),
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
    map(terminated(digit1, space1), |position: &str| {
        position.parse::<u8>().unwrap()
    })(input)
}

fn parse_player_details(input: &str) -> ParserResult<(String, u64, u8, String)> {
    map(
        many1(terminated(is_not(" "), opt(tag(" ")))),
        |tokens: Vec<&str>| {
            let len = tokens.len();
            (
                tokens[0..(len - 3)]
                    .iter()
                    .map(|token| token.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                tokens[len - 3].parse::<u64>().unwrap(),
                tokens[len - 2].parse::<u8>().unwrap(),
                tokens[len - 1].to_string(),
            )
        },
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
