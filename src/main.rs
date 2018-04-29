use std::env;
use std::fs::File;
use std::io::prelude::*;

mod tokenizer;
use tokenizer::Tokenizer;

fn main() {
	let args : Vec<String> = env::args().collect();
	let (program, output_file) = read_file(&args);
	let mut tokenizer = Tokenizer::new(program);
	tokenizer.start();
	// tokenizer.test('\u{fe0f}');
}

fn read_file(args: &[String]) -> (String, String) {
	let input_file = &args[1];
	let output_file = &args[2];
    
    let mut f = File::open(input_file).expect("File not found.");
    let mut program = String::new();
    f.read_to_string(&mut program).expect("Reading file failed.");

    (program, output_file.to_string())
}