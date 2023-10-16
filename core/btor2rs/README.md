# Safe-Rust Btor2 parser

This crate is a lighweight alternative to [Btor2Tools](https://github.com/Boolector/btor2tools)
and their Rust wrapper [btor2tools](https://docs.rs/btor2tools/latest/btor2tools/).
Only safe Rust is used in this parser.

# Usage
```no_run
use btor2rs::Btor2;
let path = std::path::Path::new("example.btor2");
let content = std::fs::read_to_string(path).unwrap();
let btor2 = Btor2::parse(content.lines());
println!("Parsed: {:?}", btor2);
````

# Notes on Btor2

The Btor2 format is (incompletely) documented in
[Niemetz, A., Preiner, M., Wolf, C., Biere, A. (2018). BTOR2, BtorMC and Boolector 3.0. CAV 2018.
](https://doi.org/10.1007/978-3-319-96145-3_32)
This crate aims for compatibility with the format as parsed by Btor2Tools
and used in [Hardware model-checking benchmarks](
https://gitlab.com/sosy-lab/research/data/word-level-hwmc-benchmarks).
Specifically, [right-side nodes can be immediately bit-inverted by using a minus sign](
https://github.com/Boolector/btor2tools/issues/15), which is not present in the original paper.

# License

This crate is licensed under Apache 2.0 License or MIT License at your discretion.
