use std::collections::HashMap;
use std::fmt;
use std::str::{self};
use nom::bytes::complete::*;
use nom::character::{complete::*};
use nom::combinator::*;
use nom::branch::alt;
use nom::sequence::{separated_pair, delimited, terminated, tuple};
use nom::IResult;
use nom::error::{ErrorKind};

use nom::Err::Error;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Duration};
// use chrono::format::ParseError as ChronoParseError;


pub fn bracketed(input: &str) -> IResult<&str, &str> {
    delimited(char('['), is_not("]"), char(']'))(input)
}

fn hh_mm_ss(input: &str) -> IResult<&str, &str> {
    recognize(tuple((digit1, tag(":"), digit1, tag(":"), digit1)))(input)
}

pub fn ts(input: &str) -> IResult<&str, &str> {
    hh_mm_ss(input)
}

pub fn user_name(input: &str) -> IResult<&str, &str> {
    delimited(space1, is_not(" "), space1)(input)
}

pub struct Preamble<'a> {
    timestamp: NaiveTime,
    label: &'a str
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAction {
    Joined,
    Left,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerEvent {
    pub action: PlayerAction,
    pub timestamp: NaiveDateTime,
}

pub struct Session {
    pub start: Option<NaiveDateTime>,
    pub stop: Option<NaiveDateTime>,
    duration: Duration,
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.duration.num_seconds() % 60;
        let minutes = (self.duration.num_seconds() / 60) % 60;
        let hours = (self.duration.num_seconds() / 60) / 60;
        write!(f, "@ {} - duration: {:02}:{:02}:{:02}", self.start.unwrap().time(), hours, minutes, seconds)
    }
}
impl Session {
    pub fn new() -> Self { Self { start: None, stop: None, duration: Duration::zero() } }

    /// Set the session's start.
    pub fn set_start(&mut self, start: NaiveDateTime) {
        self.start = Some(start);
    }

    /// Set the session's stop.
    pub fn set_stop(&mut self, stop: NaiveDateTime) {
        self.stop = Some(stop);
        if let Some(start) = self.start {
            self.duration = stop - start;
        }
    }

    pub fn clear(&mut self) {
        self.stop = None;
        self.start = None;
        self.duration = Duration::zero();
    }


    /// Get the session's duration.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

impl PlayerEvent {
    pub fn new(action: PlayerAction, timestamp: NaiveDateTime) -> Self { Self { action, timestamp} }
}

pub type Events = Vec<PlayerEvent>;

pub type Players = HashMap<String, Events>;
pub struct PlayerData {
    pub sessions: Vec<Session>,
    pub events: Events,
    total_time: Duration,
}

impl PlayerData {
    pub fn new(sessions: Vec<Session>, events: Events, total_time: Duration) -> Self { Self { sessions: Vec::new(), events: Events::new(), total_time: Duration::zero() } }

    /// Set the player data's events.
    pub fn add_event(&mut self, event: PlayerEvent) {
        match event.action {
            PlayerAction::Joined => self.events.push(event),
            PlayerAction::Left => {
                if let Some(start) = self.events.pop() {
                    let mut session = Session::new();
                    session.set_start(start.timestamp);
                    session.set_stop(event.timestamp);
                    self.total_time = self.total_time + session.duration();
                    self.sessions.push(session);
                } else {
                    self.events.push(event);
                }
            },
        }
    }
}

pub fn preamble(input: &str) -> IResult<&str, Preamble> {
    match terminated(separated_pair(timestamp, space1, bracketed), tag(":"))(input) {
        Ok((x, (t, b))) => {                        
            Ok((x, Preamble{timestamp: t, label: b}))
        },
        Err(x) => Err(x),
    }
}

fn parse_joined(input: &str) -> IResult<&str, PlayerAction> {
    match tag("joined")(input) {
        Ok((i, _)) => Ok((i,PlayerAction::Joined)),
        Err(x) => Err(x),
    }
}
fn parse_left(input: &str) -> IResult<&str, PlayerAction> {
    match tag("left")(input) {
        Ok((i, _)) => Ok((i,PlayerAction::Left)),
        Err(x) => Err(x),
    }
}

pub fn parse_action(input: &str) -> IResult<&str, PlayerAction> {
    alt((parse_joined, parse_left))(input)
}

pub fn msg_the_game<'a>(input: &'a str, date: &'a mut NaiveDate) -> IResult<&'a str, (&'a str, PlayerEvent)> {
    match terminated(
                tuple((preamble, user_name, parse_action)), 
                tag(" the game")
            )(input) {
        Ok((i, (p, user, action ))) => {
            Ok((i, (user, PlayerEvent{ action: action, timestamp: date.and_time(p.timestamp)})))
            },
        Err(x) => Err(x),   
    }
}

pub fn parse_datelike(input: &str) -> IResult<&str, NaiveDate> {
    match tuple((i32, tag("-"), u32, tag("-"), u32))(input) {
        Ok((i,(year, _, month, _, day))) => Ok((i, NaiveDate::from_ymd(year, month, day))),
        Err(e) => Err(e),
    }
}

pub fn timestamp(input: &str) -> IResult<&str, NaiveTime> {
    match  bracketed(input) {
        Ok((i, o)) => {
            match NaiveTime::parse_from_str(o, "%H:%M:%S") {
                Ok(t) => Ok((i, t)),
                Err(_) => Err(Error(nom::error::Error { input: i, code: ErrorKind::Alpha })),
            }
        },
        Err(x) => Err(x),
    }
}
