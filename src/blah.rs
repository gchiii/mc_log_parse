// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

// Import the macro. Don't forget to add `error-chain` in your
// `Cargo.toml`!
#[macro_use]
extern crate error_chain;

use error_chain::error_chain;

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use flate2::read::GzDecoder;
use regex::Regex;
use regex::RegexSetBuilder;


// // We'll put our errors in an `errors` module, and other modules in
// // this crate will `use errors::*;` to get access to everything
// // `error_chain!` creates.
mod errors {
//     // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}
error_chain! {
    foreign_links {
        Io(std::io::Error);
        Regex(regex::Error);
    }    
}

// This only gives access within this module. Make this `pub use errors::*;`
// instead if the types must be accessible from other modules (e.g., within
// a `links` section).
use errors::*;


fn main() -> Result<()> {

    println!("{:?}", std::env::current_dir());
    let mut fname = "./logs/2022-03-29-1.log.gz";
    let path = Path::new(fname);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let set = RegexSetBuilder::new(&[
            r#"joined the game"#,
            r#"left the game"#,
            // r#"\d{2}:\d{2}:\d{2}"#,
        ]).build()?;


    // let timestamp_re = Regex::new(r"[(\d{2}:\d{2}:\d{2})]").unwrap();

    let reader = BufReader::new(GzDecoder::new(file));

    reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| set.is_match(line.as_str()))
        .for_each(|x| println!("{}", x));

    // for line in reader.lines() {
    //     println!("{}", line?);
    //     let cap = timestamp_re.captures(line.map_or("", |m| m.as_str())).unwrap();
    //     println!("timestamp: {:?}", cap.get(0));
    //     // for cap in timestamp_re.captures_iter(line. ) {
    //     // }
    // }

    Ok(())
    // println!("Hello, world!");
}
