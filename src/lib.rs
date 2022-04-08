use std::collections::HashMap;
use std::str;
use nom::bytes::complete::*;
use nom::character::{complete::*};
use nom::combinator::*;
use nom::branch::alt;
use nom::sequence::{separated_pair, delimited, terminated, tuple};
use nom::IResult;
use nom::error::{ErrorKind};

use nom::Err::Error;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
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
    pub timestamp: NaiveTime,
}

pub struct Session {
    pub start: NaiveTime,
    pub stop: NaiveTime,
}

impl PlayerEvent {
    pub fn new(action: PlayerAction, timestamp: NaiveTime) -> Self { Self { action, timestamp} }
}

pub type Events = Vec<PlayerEvent>;

pub type Players = HashMap<String, Events>;

pub fn preamble(input: &str) -> IResult<&str, Preamble> {
    match terminated(separated_pair(timestamp, space1, bracketed), tag(":"))(input) {
        Ok((x, (t, b))) => {                        
            Ok((x, Preamble{timestamp: t, label: b}))
        },
        Err(x) => Err(x),
    }
}

pub fn parse_joined(input: &str) -> IResult<&str, PlayerAction> {
    match tag("joined")(input) {
        Ok((i, _)) => Ok((i,PlayerAction::Joined)),
        Err(x) => Err(x),
    }
}
pub fn parse_left(input: &str) -> IResult<&str, PlayerAction> {
    match tag("left")(input) {
        Ok((i, _)) => Ok((i,PlayerAction::Left)),
        Err(x) => Err(x),
    }
}

pub fn parse_action(input: &str) -> IResult<&str, PlayerAction> {
    alt((parse_joined, parse_left))(input)
}

pub fn msg_the_game(input: &str) -> IResult<&str, (&str, PlayerEvent)> {
    match terminated(
                tuple((preamble, user_name, parse_action)), 
                tag(" the game")
            )(input) {
        Ok((i, (p, user, action ))) => {
            Ok((i, (user, PlayerEvent{ action: action, timestamp: p.timestamp})))
            },
        Err(x) => Err(x),   
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
