use chrono::{NaiveDate};
use log_parse::*;

use std::{fs::File, collections::HashMap};
use std::io::{prelude::*, BufReader, self};

use flate2::read::GzDecoder;
use glob::glob;

fn main() {

    let mut players: Players = HashMap::new();


    for entry in glob("./logs/*.log.gz").expect("no files") {
        if let Ok(path) = entry {
            let fname = path.file_name().unwrap();
            let (_, mut date) = parse_datelike(fname.to_str().unwrap()).unwrap();
            let display = path.display();
            let file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why),
                Ok(file) => file,
            };
        
            let mut reader = BufReader::new(GzDecoder::new(file));
            
            extract_player_data(&mut players, &mut reader, &mut date).unwrap();
        } else {
            todo!()
        };
    }    

    for (user, events) in &players {
        println!("{}:", user);
        let mut session = Session::new();
        let mut day:Option<NaiveDate> = None;
        for event in events {
            if session.duration().is_zero() == false {                
                println!("    {}", session);
                session.clear();
            }
            if day != Some(event.timestamp.date()) {
                day = Some(event.timestamp.date());
                println!("   {:?}", day.unwrap());
            }
            match event.action {
                PlayerAction::Joined => {
                    session.set_start(event.timestamp);
                },
                PlayerAction::Left => {
                    session.set_stop(event.timestamp);
                },
            }
        }
    }        
}

fn extract_player_data<R: BufRead>(players: &mut Players, reader: &mut R, date: &mut NaiveDate) -> io::Result<()> {
    reader.lines()
    .filter_map(|line| line.ok())
    .for_each(|x| 
        match msg_the_game(x.as_str(), date) {
            Ok((_y,(name, event))) => {
                if players.contains_key(name) {
                    if let Some(events) = players.get_mut(name) {                         
                        events.push(event);
                    }
                } else {
                    let mut events: Events = Vec::new();
                    events.push(event);
                    players.insert(String::from(name), events);
                }
            },
            _ => (),
        }
    );
    Ok(())
}
