use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::io::BufReader;
use std::io::BufWriter;
use std::collections::HashMap;
use std::ops::RangeFrom;

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
}

enum Line<'a> {
    Instruction { opcode: &'a str, arg: Argument<'a> },
    Label { name: &'a str }
}

enum Argument<'a> {
    Integer(i16),
    Label(&'a str),
    None
}

struct Instruction<'a> {
    opcode: &'a str,
    arg: i16
}

fn compile(source_path: &String, destination_path: &String) -> io::Result<()> {
    let raw_lines = try!(read_lines(source_path));
    let lines = raw_lines.iter().map(parse_line);
    let instructions = resolve(lines.collect());
    let bytecodes = instructions.into_iter().map(encode_instruction);
    write_lines(destination_path, bytecodes)
}

fn parse_line<'a>(line: &'a String) -> Line<'a> {
    if line.starts_with(":") {
        Line::Label { name: line }
    } else {
        let mut parts = line.split(" ");
        let opcode = parts.next().unwrap();
        // TODO: reject args for noarg opcodes
        let arg = parse_arg(parts.next());
        Line::Instruction { opcode: opcode, arg: arg }
    }
}

fn parse_arg(part: Option<&str>) -> Argument {
    // TODO: don't drop parse errors on the floor
    part.and_then(
        |s|
        if s.starts_with(":") {
            Some(Argument::Label(s))
        } else {
            s.parse::<i16>().ok().map(Argument::Integer)
        })
        .unwrap_or(Argument::None)
}

fn resolve<'a>(lines: Vec<Line<'a>>) -> Vec<Instruction<'a>> {
    let lines_with_addresses = lines_with_addresses(lines);
    let label_addresses = find_labels(&lines_with_addresses);
    lines_with_addresses.into_iter().filter_map(|line| resolve_line(&label_addresses, line)).collect()
}

fn resolve_line<'a>(label_addresses: &HashMap<String, i16>, (line, address) : (Line<'a>, i16)) -> Option<Instruction<'a>> {
    match line {
        Line::Instruction { opcode: opcode, arg: arg } =>
            Option::Some(Instruction { opcode: opcode, arg: resolve_arg(label_addresses, address, &arg) }),
        _ =>
            Option::None
    }
}

fn resolve_arg<'a>(label_addresses: &HashMap<String, i16>, address: i16, argument: &Argument<'a>) -> i16 {
    match argument {
        &Argument::Integer(value) => value,
        &Argument::Label(name) => label_addresses[name.to_string()] - (address + 1),
        &Argument::None => 0
    }
}

fn encode_instruction<'a>(instruction: Instruction<'a>) -> i32 {
    let bytecode = encode_opcode(&instruction.opcode);
    let arg = instruction.arg as i32;
    bytecode + (arg << 16)
}

fn encode_opcode(name: &str) -> i32 {
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

fn lines_with_addresses<'a>(lines: Vec<Line<'a>>) -> Vec<(Line<'a>, i16)> {
    let mut address = 0;
    let mut result = Vec::new();
    for line in lines.into_iter() {
        let is_instruction = match line {
            Line::Instruction {..} => true,
            _ => false
        };
        result.push((line, address));
        if is_instruction {
            address += 1
        }
    }
    result
}

fn find_labels<'a>(lines: &Vec<(Line<'a>, i16)>) -> HashMap<String, i16> {
    let mut labels = HashMap::new();
    for &(ref line, address) in lines {
        match line {
            &Line::Label { name: name } => {
                labels.insert(name.to_string(), address);
            },
            _ => ()
        }
    }
    labels
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
