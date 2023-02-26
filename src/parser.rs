use std::cmp::Ordering;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::branch::alt;
use nom::sequence::{separated_pair, delimited, terminated, tuple};
use nom::IResult;
use nom::error::{ErrorKind};

use nom::Err::Error;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};


#[derive(Debug)]
pub struct LogHeader<'a> {
    timestamp: NaiveTime,
    _tag: &'a str
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerAction {
    Connect,
    Disconnect,
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
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.action == other.action && self.timestamp == other.timestamp
    }
}

impl PartialOrd for PlayerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

impl Eq for PlayerEvent {}

impl Ord for PlayerEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
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

fn name_with(input: &str) -> IResult<&str, &str> {
    match tuple( (is_not(" "), opt(bracketed)) )(input) {
        Ok((s1,(s2, _))) => Ok((s1, s2)),
        Err(e) => Err(e),
    }
}

pub fn user_name(input: &str) -> IResult<&str, &str> {
    // delimited(space1, is_not(" "), space1)(input)
    delimited(space1, name_with, space1)(input)
}


pub fn parse_log_header(input: &str) -> IResult<&str, LogHeader> {
    match terminated(separated_pair(timestamp, space1, bracketed), tag(":"))(input) {
        Ok((x, (t, b))) => {
            Ok((x, LogHeader{timestamp: t, _tag: b}))
        },
        Err(x) => Err(x),
    }
}

fn parse_connected(input: &str) -> IResult<&str, PlayerAction> {
    match tag("logged in with")(input) {
        Ok((i, _)) => Ok((i,PlayerAction::Connect)),
        Err(x) => Err(x),
    }
}
fn parse_disconnected(input: &str) -> IResult<&str, PlayerAction> {
    match tag("lost connection:")(input) {
        Ok((i, _)) => Ok((i,PlayerAction::Disconnect)),
        Err(x) => Err(x),
    }
}

pub fn parse_action(input: &str) -> IResult<&str, PlayerAction> {
    alt((parse_connected, parse_disconnected))(input)
}

fn a_name(input: &str) -> IResult<&str, &str>{
    delimited(char(' '), is_not(" "), char(' '))(input)
}

//[01:19:56] [User Authenticator #1/INFO]: UUID of player C4charlieh is 385ac9cd-4e1a-312c-b90c-67b9e947e7ca
fn parse_login(input: &str) -> IResult<&str, (&str, PlayerAction)> {
    let (i, name) = delimited(
        tag(" UUID of player"),
        a_name,
        tag("is"))(input)?;
    Ok((i, (name, PlayerAction::Connect)))
}

//[01:38:25] [Server thread/INFO]: C4charlieh lost connection: Disconnected
fn parse_disconnect(input: &str) -> IResult<&str, (&str, PlayerAction)> {
    let (i, (name, _)) = tuple( (a_name, tag("lost connection: Disconnected")))(input)?;
    Ok((i, (name, PlayerAction::Disconnect)))
}

pub fn parse_event<'a>(input: &'a str, date: &'a mut NaiveDate) -> IResult<&'a str, (&'a str, PlayerEvent)> {
    let (i, (p, (user, action))) = tuple((parse_log_header, alt((parse_login, parse_disconnect)))) (input)?;
    // println!("event=> {:?} - {} - {:?}", p, user, action);
    Ok((i, (user, PlayerEvent{name: user.to_string(),  action, timestamp: date.and_time(p.timestamp)})))
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
