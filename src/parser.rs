use std::cmp::Ordering;

use nom::bytes::complete::*;
use nom::character::{complete::*};
use nom::combinator::*;
use nom::branch::alt;
use nom::sequence::{separated_pair, delimited, terminated, tuple};
use nom::IResult;
use nom::error::{ErrorKind};

use nom::Err::Error;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Duration};


pub struct LogHeader<'a> {
    timestamp: NaiveTime,
    _tag: &'a str
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerAction {
    Joined,
    Left,
}


#[derive(Debug, Clone)]
pub struct PlayerEvent {
    pub name: String,
    pub action: PlayerAction,
    pub timestamp: NaiveDateTime,
}

impl PlayerEvent {
    pub fn new(name: String, action: PlayerAction, timestamp: NaiveDateTime) -> Self { Self { action, timestamp, name } }
}

impl PartialEq for PlayerEvent {

    fn ne(&self, other: &Self) -> bool {
        self.name == other.name && self.action == other.action && self.timestamp == other.timestamp
    }

    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.action == other.action && self.timestamp == other.timestamp
    }
}
impl Eq for PlayerEvent {}

impl Ord for PlayerEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}
impl PartialOrd for PlayerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.timestamp.cmp(&other.timestamp))
    }
}


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


pub fn parse_log_header(input: &str) -> IResult<&str, LogHeader> {
    match terminated(separated_pair(timestamp, space1, bracketed), tag(":"))(input) {
        Ok((x, (t, b))) => {                        
            Ok((x, LogHeader{timestamp: t, _tag: b}))
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


pub fn parse_event<'a>(input: &'a str, date: &'a NaiveDate) -> IResult<&'a str, (&'a str, PlayerEvent)> {
    match terminated(
                tuple((parse_log_header, user_name, parse_action)), 
                tag(" the game")
            )(input) {
        Ok((i, (p, user, action ))) => {
            Ok((i, (user, PlayerEvent{name: user.to_string(),  action: action, timestamp: date.and_time(p.timestamp)})))
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
