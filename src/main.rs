use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

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
    gen_data(&mut file);
    gen_code(&mut file);
}

fn gen_data(file: &mut File){
    let content = ".data\n\
                    \t\targc_: .quad 0\n\
                    \t\tFormat: .byte '%', 'l', 'u', 10, 0\n\
                    \t\tFuncTable: .quad 0\n\
                    \t\tFuncCall: .quad 0\n";
    match file.write_all(content.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to file: {}", why.description())
        },
        Ok(_) => {},
    }
}

fn gen_code(file: &mut File){
    let content = ".text\n\
                    .global main\n\
                    .extern printf\n\
                    .extern malloc\n\
                    main:\n\
                    \t\tmovq %rdi, argc_\n\
                    \t\tmovq $16000, %rdi\n\
                    \t\tcall malloc\n\
                    \t\tmovq %rax, FuncTable\n";
    match file.write_all(content.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to file: {}", why.description())
        },
        Ok(_) => {},
    }
    // load_token(code_text);
    // init funcs;
    // seq();
    match file.write_all("\t\tretq\n".as_bytes()) {
        Err(why) => {
            panic!("couldn't write to file: {}", why.description())
        },
        Ok(_) => {},
    }
}