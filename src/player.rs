use std::collections::HashMap;
use std::fmt;
use std::ops::Range;
use std::str::{self};
use std::sync::{Mutex, Arc, RwLock};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Duration};

use ansi_term::Color;
use flume::{Receiver, Sender};
use futures::sink::Send;


use crate::parser::*;

use flume;

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
    pub fn new(start: NaiveDateTime, stop: NaiveDateTime, duration: Duration) -> Self { Self { start, stop, duration } }

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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Player {
    pdata: Arc<RwLock<PlayerData>>,
    name: String,
    tx: Sender<PlayerEvent>,
}

impl Player {
    pub fn new(name: String) -> Self { 
        let (tx, rx) = flume::unbounded();

        let player = Self { 
            pdata: Arc::new(RwLock::new(PlayerData::new())), 
            name: name,
            tx: tx,
        };

        let mut p = player.clone();
        tokio::spawn(async move {
            while let Ok(event) =  rx.recv_async().await {
                p.add_event(event);
            }
        });
        player
    }

    /// Get the player data's total time.
    pub fn total_time(&self) -> Duration {
        self.pdata.read().unwrap().total_time()
    }

    pub async fn send_event(&mut self, event: PlayerEvent) -> Result<(), flume::SendError<PlayerEvent>> {
        self.tx.send_async(event).await
    }

    /// Set the player data's events.
    fn add_event(&mut self, event: PlayerEvent) {
        match self.pdata.write() {
            Ok(mut pdata) => {
                pdata.add_event(event)
            },
            Err(_) => todo!(),
        }
    }

    pub fn print(self) {
        match self.pdata.read() {
            Ok(data) => {
                let user_total = format!("total time = {}", duration_hhmmss(data.total_time()));
                let user_disp = format!("{}:", self.name());
                println!("{} {}", Color::Yellow.paint(user_disp), Color::Red.paint(user_total));
                
                for day in &data.days {
                    let daily_total = format!("  {} - daily total = {}", day.date, duration_hhmmss(day.total_time));
                    println!("{}", Color::Green.paint(daily_total));
                    let y = &data.sessions[day.range()];
                    for session in y {
                        println!("      {}", session);
                    }
                }
    
            },
            Err(_) => todo!(),
        }
    }


    /// Get a reference to the player's name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

pub type Players = HashMap<String, PlayerData>;

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub players: Arc<RwLock<HashMap<String, Player>>>,
    pub tx: Sender<PlayerEvent>,
}

impl GameInfo {
    pub fn new() -> Self { 
        let (tx, rx) = flume::unbounded();
        let info = Self { 
            players: Arc::new(RwLock::new(HashMap::<String, Player>::new())), 
            tx: tx,
        };
        let mut game_info = info.clone();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv_async().await {
                match game_info.players.write() {
                    Ok(mut players) => {
                        let player_data = players.entry(event.name.clone()).or_insert(Player::new(event.name.clone()));
                        let p = player_data.clone();
                        p.tx.send(event);
                    }
                    _ => (),
                }
            }
        });
        info
    }

    pub fn print(self) {
        match self.players.read() {
            Ok(players) => {
                players.clone().into_values().for_each(|player| {
                    player.print();
                });
            },
            Err(_) => todo!(),
        }
    }

}

