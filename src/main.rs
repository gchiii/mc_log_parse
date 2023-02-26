use chrono::{NaiveDate, DateTime, Utc};
use log_parse::*;


use std::path::{PathBuf, Path};
use std::{fs::File};
use std::io::{prelude::*, BufReader};

use flate2::read::GzDecoder;
use glob::glob;

use clap::Parser;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(parse(from_os_str), default_value = "./logs")]
    log_path: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    // let mut players: Players = HashMap::new();
    let mut game_info = GameInfo::new();

    let pattern : PathBuf = [args.log_path.to_str().unwrap(), "*.log*"].iter().collect();
    for entry in glob(pattern.to_str().unwrap()).expect("no files") {
        if let Ok(path) = entry {
            let mut date = extract_date_from_path(&path);

            let display = path.display();
            let file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why),
                Ok(file) => file,
            };

            let mut reader = get_reader(path.extension().unwrap() == "gz", file);
            let pdata = extract_player_data(&mut reader, &mut date);
            for (_name, event) in pdata {
                game_info.add_event(event);
            }
        } else {
            todo!()
        };
    }
    game_info.print_players();
}

fn extract_date_from_path(path: &Path) -> NaiveDate{
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


fn extract_player_data<R: BufRead>(reader: &mut R, date: &mut NaiveDate) -> Vec<(String, PlayerEvent)> {
    let mut pdata = Vec::<(String, PlayerEvent)>::new();
    reader.lines()
    .filter_map(|line| line.ok())
    .for_each(|x|
        if let Ok((_y,(name, event))) = parse_event(x.as_str(), date) {
            pdata.push((name.to_string(), event));
        }
    );
    pdata
}
