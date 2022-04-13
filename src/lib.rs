use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
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

pub struct LogHeader<'a> {
    timestamp: NaiveTime,
    _tag: &'a str
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

impl PlayerEvent {
    pub fn new(action: PlayerAction, timestamp: NaiveDateTime) -> Self { Self { action, timestamp} }
}

pub type Events = Vec<PlayerEvent>;

#[derive(Debug, Clone, Copy)]
pub struct Session {
    pub start: NaiveDateTime,
    pub stop: NaiveDateTime,
    duration: Duration,
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.duration.num_seconds() % 60;
        let minutes = (self.duration.num_seconds() / 60) % 60;
        let hours = (self.duration.num_seconds() / 60) / 60;
        write!(f, "@ {} - duration: {:02}:{:02}:{:02}", self.start.time(), hours, minutes, seconds)
    }
}

impl Session {
    pub fn build(joined: &PlayerEvent, left: &PlayerEvent) -> Session {
        Session { start: joined.timestamp, stop: left.timestamp, duration: left.timestamp - joined.timestamp }
    }

    /// Get the session's duration.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

pub fn duration_hhmmss(duration: Duration) -> String {
    let seconds = duration.num_seconds() % 60;
    let minutes = (duration.num_seconds() / 60) % 60;
    let hours = (duration.num_seconds() / 60) / 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerDay {
    pub date: NaiveDate,
    first_index: usize, 
    last_index: usize,
    pub total_time: Duration,
}

impl PlayerDay {
    pub fn new(session: &Session, index: usize) -> Self { 
        Self { 
            date: session.start.date(),
            first_index: index,
            last_index: index+1, 
            total_time: session.duration(),
        } 
    }

    pub fn add_session(&mut self, session: &Session) {
        self.last_index += 1;
        self.total_time = self.total_time + session.duration;
    }

    pub fn range(self) -> Range<usize> {
        Range { start: self.first_index, end: self.last_index }
    }
}

pub struct PlayerData {
    pub sessions: Vec<Session>,
    pub events: Events,
    pub days: Vec<PlayerDay>,
    pub total_time: Duration,
}

impl PlayerData {
    pub fn new() -> Self { Self { 
        sessions: Vec::new(), 
        events: Events::new(), 
        days: Vec::<PlayerDay>::new(), 
        total_time: Duration::zero() 
    } }

    fn add_session(&mut self, session: Session) {
        let first_index = self.sessions.len();
        if self.days.is_empty() {
            self.days.push(PlayerDay::new(&session, first_index));
        } else {
            if let Some(d) = self.days.last_mut() {
                if d.date == session.start.date() {
                    d.add_session(&session);
                } else {
                    self.days.push(PlayerDay::new(&session, first_index));
                }
            }
        }
        self.total_time = self.total_time + session.duration();
        self.sessions.push(session);
    }

    /// Set the player data's events.
    pub fn add_event(&mut self, event: PlayerEvent) {
        match event.action {
            PlayerAction::Joined => self.events.push(event),
            PlayerAction::Left => {
                if let Some(start) = self.events.pop() {
                    let session: Session = Session::build(&start, &event);
                    self.add_session(session);
                } else {
                    self.events.push(event);
                }
            },
        }
    }

    /// Get the player data's total time.
    #[must_use]
    pub fn total_time(&self) -> Duration {
        self.total_time
    }
}

pub type Players = HashMap<String, PlayerData>;


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


pub fn parse_event<'a>(input: &'a str, date: &'a mut NaiveDate) -> IResult<&'a str, (&'a str, PlayerEvent)> {
    match terminated(
                tuple((parse_log_header, user_name, parse_action)), 
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
