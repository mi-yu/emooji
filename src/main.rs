use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

mod tokenizer;
use tokenizer::Tokenizer;

fn main() {
    println!("Hello, world!");
    let path = Path::new("lorem_ipsum.txt");
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                       	    path.display(),
                            why.description()),
        Ok(file) => file,
    };
    gen_data(&mut file, path);
}

fn gen_data(file: &mut File, path: &Path) {
    let content = ".data\n\
                    \t\targc_: .quad 0\n\
                    \t\tFormat: .byte '%%', 'l', 'u', 10, 0\n\
                    \t\tFuncTable: .quad 0\n\
                    \t\tFuncCall: .quad 0\n";
    match file.write_all(content.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {:?}: {}", path,
                                               why.description())
        },
        Ok(_) => println!("successfully wrote to {:?}", path),
    }
}