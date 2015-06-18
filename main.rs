use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::io::BufReader;
use std::io::BufWriter;

fn main() -> () {
    println!("Hello, world!");

    let arguments: Vec<_> = env::args().collect();
    let (source_path, destination_path) = match &arguments[..] {
        [_, ref source_path, ref destination_path, ..] => (source_path, destination_path),
        _ => panic!("Not enough arguments")
    };

    match compile(source_path, destination_path) {
        Ok(_) => println!("Finished"),
        Err(e) => println!("{}", e)
    }
    //println!(source_path)
}

fn compile(source_path: &String, destination_path: &String) -> io::Result<()> {
    let lines = try!(read_lines(source_path));
    let bytecodes = lines.into_iter().map(parse_line);
    write_lines(destination_path, bytecodes)
}

// String -> Line
fn parse_line(line: String) -> i32 {
    let mut parts = line.split(" ");
    let opcode = opcode(parts.next().unwrap());
    // TODO: reject args for noarg opcodes
    // let arg = parts.next().map(std::str::FromStr<i16>::from_str).unwrap_or(0);
    let arg = parts.next()
        // TODO: don't drop parse errors on the floor
        .and_then(|s| s.parse::<i16>().ok())
        .unwrap_or(0);
    opcode + ((arg as i32) << 16)
}

fn opcode(name: &str) -> i32 {
    match name {
        "const" => 0, // value << 16
        "pop" => 1,
        "dup" => 2,
        "swap" => 3, // + (depth << 16)
        "cmp" => 4,
        "add" => 5,
        "mul" => 6,
        "jmp" => 7,
        "jle" => 8,
        _ => panic!("Unrecognised opcode")
    }
}

fn read_lines(path: &String) -> io::Result<Vec<String>> {
    let file = try!(File::open(&path));
    let lines = BufReader::new(file).lines();
    lines.collect()
}

fn write_lines<I: Iterator<Item=i32>>(path: &String, bytecodes: I) -> io::Result<()> {
    let file = try!(File::create(path));
    let mut writer = BufWriter::new(file);
    let bytecodes_vec: Vec<i32> = bytecodes.collect();
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            bytecodes_vec.as_ptr() as *const u8,
            bytecodes_vec.len() * std::mem::size_of::<i32>())
    };
    writer.write_all(bytes)
}

fn with_lines<F: Fn(String) -> String>(f: F) -> io::Result<()> {
    let arguments: Vec<_> = env::args().collect();
    let (source_file, destination_file) = match &arguments[..] {
        [_, ref source_file, ref destination_file, ..] => (source_file, destination_file),
        _ => panic!("Not enough arguments")
    };

    let in_file = try!(File::open(source_file));

    let out_file = try!(File::create(destination_file));
    let mut writer = BufWriter::new(out_file);

    let lines = BufReader::new(in_file).lines();
    for line in lines {
        let line2 = f(try!(line));
        try!(writer.write_fmt(format_args!("{}\n", line2)));
    }
    Ok(())
}
