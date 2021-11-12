// mod uci;

// use std::error::Error;
// use std::fs;
use std::io;

pub struct UCIInterface {
    pub query: String,
    pub filename: String,
    // pub test: io::
}

impl UCIInterface {
    pub fn new( args: &[String]) -> Result<Config, &str> {
        // --snip--
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // --snip--
}