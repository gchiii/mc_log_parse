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

    for (user, player_data) in &players {
        println!("{}: total time = {}", user, duration_hhmmss(player_data.total_time()));
        
        for day in &player_data.days {
            println!("  {} - daily total = {}", day.date, duration_hhmmss(day.total_time));
            let y = &player_data.sessions[day.range()];
            for session in y {
                println!("      {}", session);
            }
        }
    }
}

fn extract_player_data<R: BufRead>(players: &mut Players, reader: &mut R, date: &mut NaiveDate) -> io::Result<()> {
    reader.lines()
    .filter_map(|line| line.ok())
    .for_each(|x| 
        match parse_event(x.as_str(), date) {
            Ok((_y,(name, event))) => {
                if players.contains_key(name) {
                    if let Some(player_data) = players.get_mut(name) {
                        player_data.add_event(event);
                    }
                } else {
                    let mut player_data = PlayerData::new();
                    player_data.add_event(event);
                    players.insert(String::from(name), player_data);
                }
            },
            _ => (),
        }
    );
    Ok(())
}
