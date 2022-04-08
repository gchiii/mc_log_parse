use chrono::{NaiveTime, NaiveDateTime, NaiveDate};
use log_parse::*;

// use std::intrinsics::discriminant_value;
use std::{fs::File, collections::HashMap};
use std::io::{prelude::*, BufReader, self};
use std::path::Path;
use flate2::read::GzDecoder;
use glob::glob;

fn main() {

    let mut players: Players = HashMap::new();

    // let blah = "logs/2022-03-11-2.log.gz";
    // println!("{:?}", parse_datelike(blah));

    for entry in glob("./logs/*.log.gz").expect("no files") {
        if let Ok(path) = entry {
            let fname = path.file_name().unwrap();
            // let mut date = NaiveDate::from_ymd(2022, 1, 1);
            let (_, mut date) = parse_datelike(fname.to_str().unwrap()).unwrap();
            // if let Some(fname) = path.file_name() {
                // Ok(x,date) = parse_datelike(fname.to_str().unwrap());
            // }
            let display = path.display();
            // println!("file -> {:?}", display);
            // Open the path in read-only mode, returns `io::Result<File>`
            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why),
                Ok(file) => file,
            };
        
            let mut reader = BufReader::new(GzDecoder::new(file));
            // let date: NaiveDateTime = NaiveDateTime
            
            extract_player_data(&mut players, &mut reader, &mut date).unwrap();
        } else {
            todo!()
        };
    }    

    for (user, events) in &players {
        println!("{}", user);
        let mut session = Session::new(None, None);
        for event in events {
            match event.action {
                PlayerAction::Joined => {
                    session.start = Some(event.timestamp);
                },
                PlayerAction::Left => {
                    session.stop = Some(event.timestamp);
                },
            }
            if let Some(duration) = session.duration() {                
                let seconds = duration.num_seconds() % 60;
                let minutes = (duration.num_seconds() / 60) % 60;
                let hours = (duration.num_seconds() / 60) / 60;
                println!("\tduration: {}:{}:{}", hours, minutes, seconds);
                // println!("\tduration: {:?}", duration.num_minutes());
                session.start = None;
                session.stop = None;
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
                // println!("{:?}", y)
            },
            _ => (),
        }
    );
    Ok(())
}
