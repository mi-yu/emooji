use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::env;
use std::process::Command;

mod compiler;
use compiler::Compiler;

fn main() {
    // parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Incorrect number of arguments.");
    }

    // read from .moo file
    let path = format!("{}{}", &args[1], ".moo");
    let path = Path::new(&path);

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open .moo file: {}",
                            why.description()),
        Ok(file) => file,
    };

    let mut program_contents = String::with_capacity(500);
    match file.read_to_string(&mut program_contents) {
        Err(why) => panic!("Couldn't read .moo file: {}",
                            why.description()),
        Ok(_) => {},
    };


    // create assembly file
    let path = format!("{}{}", &args[1], ".s");
    let path = Path::new(&path);
    let file = match File::create(&path) {
        Err(why) => panic!("Couldn't create .s file: {}",
                            why.description()),
        Ok(file) => file,
    };

    // create Tokenizer
    let mut compiler = Compiler::new(program_contents, file);

    // start compilation
    compiler.gen_data();
    compiler.check_syntax();
    compiler.gen_code();

    // compile binary
    Command::new("gcc")
    		.arg("-no-pie")
    		.arg("-fno-pie")
    		.arg("-g")
    		.arg("-o")
    		.arg(&args[1])
    		.arg(format!("{}{}", &args[1], ".s"))
    		.spawn()
    		.expect("could not assemble .s file");

    // Command::new(format!("{}{}", "./", &args[1]))
    // 		.spawn()
    // 		.expect("could not run binary");
}