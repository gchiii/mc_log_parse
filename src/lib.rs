mod parser;
pub use parser::*;

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
use std::str::{self};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Duration};



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

