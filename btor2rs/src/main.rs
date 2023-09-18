use anyhow::anyhow;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
enum Sort {
    Bitvec(usize),
}

struct Btor2 {}

fn parse_btor2(file: File) -> Result<Btor2, anyhow::Error> {
    println!("Hello, world!");

    let mut sorts = HashMap::<usize, Sort>::new();

    let lines = BufReader::new(file).lines().map(|l| l.unwrap());
    for (line_num, line) in lines.enumerate() {
        if line.starts_with(";") {
            // comment
            continue;
        }

        let mut split = line.split_whitespace();
        print!("Line: ");
        for element in split.clone() {
            print!("'{}' ", element);
        }
        println!();
        let Some(id) = split.next() else {
            // empty line
            continue;
        };

        // node
        let Ok(id): Result<usize, _> = id.parse() else {
            return Err(anyhow!("Cannot parse id on line {}", line_num));
        };
        let second = split
            .next()
            .ok_or(anyhow!("Missing second symbol on line {}", line_num))?;
        if second == "sort" {
            // insert to sorts
            let third = split
                .next()
                .ok_or(anyhow!("Missing sort type on line {}", line_num))?;
            if third == "bitvec" {
                let bitvec_length = split
                    .next()
                    .ok_or(anyhow!("Missing bitvec length on line {}", line_num))?;

                let Ok(bitvec_length): Result<usize, _> = bitvec_length.parse() else {
                    return Err(anyhow!("Cannot parse bitvec length on line {}", line_num));
                };
                sorts.insert(id, Sort::Bitvec(bitvec_length));
            }
        }
    }
    println!("Sorts: {:?}", sorts);
    Ok(Btor2 {})
}

fn main() {
    println!("CWD: {}", std::env::current_dir().unwrap().display());
    let file = File::open("examples/count2.btor2").unwrap();
    parse_btor2(file).unwrap();
}
