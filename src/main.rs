use log_parse::*;

use nom::{IResult, error::{VerboseError, context}, branch::alt, bytes::complete::{tag_no_case, tag}, sequence::tuple, character::complete::space1};

use std::{fs::File, collections::HashMap};
use std::io::{prelude::*, BufReader};
use std::path::Path;
use flate2::read::GzDecoder;


fn main() {

    // println!("{:?}", std::env::current_dir());
    let mut fname = "./logs/2022-03-29-1.log.gz";
    let path = Path::new(fname);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let reader = BufReader::new(GzDecoder::new(file));

    let mut players: Players = HashMap::new();
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|x| {
            match msg_the_game(x.as_str()) {
                Ok((_y,pe)) => {
                    if players.contains_key(pe.user) {
                        if let Some(events) = players.get_mut(pe.user) {                         
                            events.push(pe);
                        }
                    } else {
                        let mut events: Events = Vec::new();
                        events.push(pe);
                        players.insert(pe.user.to_string(), events);
                    }
                    // println!("{:?}", y)
                },
                _ => (),
            }
        } 
        )
}

