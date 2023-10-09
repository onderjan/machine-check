use std::{fs::File, io::BufRead, io::BufReader};

use proc_macro2::TokenStream;

mod btor2;
mod generator;

pub fn translate_iter<'a>(
    lines: impl Iterator<Item = &'a str>,
) -> Result<TokenStream, anyhow::Error> {
    let btor2 = btor2::parse(lines)?;
    generator::generate(btor2)
}

pub fn translate_file(file: File) -> Result<TokenStream, anyhow::Error> {
    let lines_result: Result<Vec<_>, _> = BufReader::new(file).lines().collect();
    let lines: Vec<String> = lines_result?;
    translate_iter(lines.iter().map(|l| l.as_ref()))
}
