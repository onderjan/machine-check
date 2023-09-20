#![feature(const_format_args)]
#![feature(trace_macros)]

use std::num::Wrapping;

use machine_check_macros::Abstraction;
use machine_check_traits::Abstraction;

mod interval_domain;

use interval_domain::w;

use crate::interval_domain::IntervalDomain;

#[derive(Abstraction)]
struct Demo {
    b: Wrapping<i8>,
}

fn main() {
    println!("Hello, world!");
    let a = Demo { b: w(5) };
    println!("{}", a.b);

    type AbstractDemo = <Demo as Abstraction>::AbstractType;
    let alpha: <Demo as Abstraction>::AbstractType = AbstractDemo {
        b: IntervalDomain::from_interval(42, 56),
    };
    println!("alpha: {:?}", alpha.b);

    //let x = IntervalAbstraction::from_concrete(32 as u8);
    let x = IntervalDomain::from_interval(32i8, 64);
    let y = IntervalDomain::from_interval(41, 42);
    let z = x + y;
    println!("z: {:?}", z);
    let w = x - y;
    println!("w: {:?}", w);

    //trace_macros!(true);
    const_format_args!("Args");
    //panic!("Test panic");
    //trace_macros!(false);
}
