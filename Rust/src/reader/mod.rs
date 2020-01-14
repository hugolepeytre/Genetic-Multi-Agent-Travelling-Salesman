use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn read_to_string(path: &str) -> String {
    return fs::read_to_string(path).expect("Something went wrong reading the file")
}

pub fn read_to_string2(path: &str) -> String {
    let path = Path::new(path);
    let display = path.display();
	
	let mut s = String::new();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display,
                                                   why.description()),
        Ok(file) => file,
    };
    
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                                                   why.description()),
        Ok(_) => (),
    }
    
    return s
}