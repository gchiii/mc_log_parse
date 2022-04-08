use chrono::NaiveTime;
use log_parse::*;

use std::{fs::File, collections::HashMap};
use std::io::{prelude::*, BufReader, self};
use std::path::Path;
use flate2::read::GzDecoder;
use glob::glob;

fn main() {

    // println!("{:?}", std::env::current_dir());
    // let mut fname = "./logs/2022-03-29-1.log.gz";
    let mut players: Players = HashMap::new();

    for entry in glob("./logs/*.log.gz").expect("no files") {
        if let Ok(path) = entry {
            let display = path.display();
            println!("file -> {}", display);
            // Open the path in read-only mode, returns `io::Result<File>`
            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why),
                Ok(file) => file,
            };
        
            let mut reader = BufReader::new(GzDecoder::new(file));
    
            extract_player_data(&mut players, &mut reader).unwrap();
        } else {
            todo!()
        };
    }    

    for (user, events) in &players {
        println!("{}", user);
        let mut session: Session = Session { start: NaiveTime::from_hms(0, 0, 0), stop: NaiveTime::from_hms(0, 0, 0) };
        // let mut sessions: Vec<Session> = Vec::new();
        for event in events {
            match event.action {
                PlayerAction::Joined => {
                    session.start = event.timestamp;
                },
                PlayerAction::Left => {
                    session.stop = event.timestamp;
                    let x = session.stop - session.start;
                    println!("\tduration: {:?}", x.num_minutes())
                },
            }
        }
    }        
}

fn extract_player_data<R: BufRead>(players: &mut Players, reader: &mut R) -> io::Result<()> {
    reader.lines()
    .filter_map(|line| line.ok())
    .for_each(|x| 
        match msg_the_game(x.as_str()) {
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
