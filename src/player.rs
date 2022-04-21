use std::collections::{HashMap};
use std::fmt::{self};
use std::str::{self};
use chrono::{NaiveDate, NaiveDateTime, Duration};
use ansi_term::Color;
use crate::parser::*;


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

#[derive(Debug, Clone)]
pub struct PlayerDay {
    date: NaiveDate,
    sessions: Vec<Session>,
    total_time: Duration,
}

impl PlayerDay {
    pub fn new(date: NaiveDate) -> Self { 
        Self { 
            date: date,
            sessions: Vec::<Session>::new(), 
            total_time: Duration::zero(),
        } 
    }

    pub fn add_session(&mut self, session: Session) -> Result<(), Session> {
        if self.date == session.start.date() {
            self.total_time = self.total_time + session.duration();
            self.sessions.push(session);
            Ok(())
        } else {
            Err(session)
        }
    }

    pub fn print_day_total(&self) {
        let daily_total = format!("  {} - daily total = {}", self.date, duration_hhmmss(self.total_time));
        let thing = Color::Green.paint(daily_total);
        println!("{}", thing);        
    }

    pub fn print_day_sessions(&self) {
        for session in &self.sessions {
            println!("      {}", session);
        }
    }
}

impl PartialEq for PlayerDay {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
    }
}

impl Eq for PlayerDay {}
impl Ord for PlayerDay {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
    }
}

impl PartialOrd for PlayerDay {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.date.partial_cmp(&other.date)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerData {
    name: String,
    pub events: Events,
    pub days: Vec<PlayerDay>,
    pub total_time: Duration,
}

impl PlayerData {
    pub fn new(name: &str) -> Self { 
        Self { 
            name: name.to_string(),
            events: Events::new(), 
            days: Vec::new(), 
            total_time: Duration::zero(),
        } 
    }

    fn add_day(&mut self, date: NaiveDate) {
        let day = PlayerDay::new(date);
        self.days.push(day);
    }

    fn add_session(&mut self, session: Session) {
        let date = session.start.date();

        let day = {
            if self.days.is_empty() {
                self.add_day(date)
            }
    
            if self.days.last().expect("wow, where is it").date < date {
                self.add_day(date)
            }
            self.days.last_mut().expect("msg")
        };
        self.total_time = self.total_time + session.duration();
        match day.add_session(session) {
            Ok(_) => (),
            Err(_) => (),
        }
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

    pub fn print(&self) {
        let user_total = format!("total time = {}", duration_hhmmss(self.total_time()));
        let user_disp = format!("{}:", self.name);
        println!("{} {}", Color::Yellow.paint(user_disp), Color::Red.paint(user_total));
        
        for day in &self.days {
            day.print_day_total();
            day.print_day_sessions();
        }
    }
}

pub type Players = HashMap<String, PlayerData>;

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub players: Players,
}

impl GameInfo {
    pub fn new() -> Self { 
        let info = Self { players: Players::new(), };
        info
    }


    pub fn add_event(&mut self, event: PlayerEvent) {
        let player_data = self.players.entry(event.name.clone()).or_insert(PlayerData::new(&event.name));
        player_data.add_event(event);
    }

    pub fn print_players(self) {
        for (_user, player_data) in &self.players {
            player_data.print();
        }    
    }
}

