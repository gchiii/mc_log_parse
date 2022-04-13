use chrono::{NaiveDate, DateTime, Utc};
use log_parse::*;


use std::path::PathBuf;
use std::{fs::File, collections::HashMap};
use std::io::{prelude::*, BufReader, self};

use flate2::read::GzDecoder;
use glob::glob;




fn main() {

    let mut players: Players = HashMap::new();


    for entry in glob("./logs/*.log*").expect("no files") {
        if let Ok(path) = entry {
            let mut date = extract_date_from_path(&path);            
            
            let display = path.display();
            let file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why),
                Ok(file) => file,
            };

            let mut reader = get_reader(path.extension().unwrap() == "gz", file);
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

fn extract_date_from_path(path: &PathBuf) -> NaiveDate{
    let fname = match path.file_name() {
        Some(fname) => fname.to_str().unwrap(),
        None => panic!("no file name"),
    };

    let date = match parse_datelike(fname) {
        Ok((_,date)) => date,
        Err(_e) => match path.metadata() {
            Ok(metadata) => {
                if let Ok(created) = metadata.created() {
                    let date_time = DateTime::<Utc>::from(created);
                    let date: NaiveDate = date_time.date().naive_utc();
                    date
                } else {
                    panic!("something bad happenned")
                }
            },
            Err(x) => panic!("couldn't get metadata: {}", x),
        },
    };
    date
}

fn get_reader(gz: bool, file: File) -> Box<dyn BufRead> {
    if gz {
        Box::new(BufReader::new(GzDecoder::new(file)))
    } else {
        Box::new(BufReader::new(file))
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
