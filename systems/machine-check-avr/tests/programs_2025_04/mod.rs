//! Tests for AVR properties added in April 2025.
//!
//! The original artefact from which the source and hex files are taken is
//! [available on Zenodo](https://zenodo.org/records/15109092).
#![cfg(test)]

use machine_check::{ExecArgs, ExecStrategy};
use machine_check_avr::SystemArgs;

/// Test a single AVR property.
///
/// Panics if it does not match the expected result.
fn avr_test(group: &str, release: bool, name: &str, property: Option<String>, expected: bool) {
    let target = if release { "Release" } else { "Debug" };
    let hex_file = format!(
        "{}/tests/programs_2025_04/{group}/{name}/{target}/{name}.hex",
        env!("CARGO_MANIFEST_DIR")
    );

    let check_inherent = property.is_none();

    let exec_result = machine_check_avr::execute_with_args(
        ExecArgs {
            silent: true,
            verbose: 0,
            batch: true,
            gui: false,
            inherent: check_inherent,
            assume_inherent: !check_inherent,
            property,
            strategy: ExecStrategy::Default,
        },
        SystemArgs { hex_file },
    );

    let Ok(result) = exec_result.result else {
        panic!("Expected to verify, but got {:?}", exec_result);
    };

    if result
        .clone()
        .try_into_bool()
        .is_none_or(|result| result != expected)
    {
        panic!("Expected verification result {expected}, but got {result}");
    }
}

macro_rules! test_inherent {
    ($group:ident, $target:ident, $name:ident, $expected:literal) => {
        ::pastey::paste!(
            #[test]
            fn [<avr_ $group _ $name _ $target:lower>]() {
                let release = match ::std::stringify!($target) {
                    "Debug" => false,
                    "Release" => true,
                    _ => panic!("Unknown target"),
                };
                $crate::programs_2025_04::avr_test(::std::stringify!($group), release, ::std::stringify!($name), None, $expected);
            }
        );
    };
}

macro_rules! test_inherent_ignore {
    ($group:ident, $target:ident, $name:ident, $expected:literal) => {
        ::pastey::paste!(
            #[test]
            #[ignore]
            fn [<avr_ $group _ $name _ $target:lower>]() {
                let release = match ::std::stringify!($target) {
                    "Debug" => false,
                    "Release" => true,
                    _ => panic!("Unknown target"),
                };
                $crate::programs_2025_04::avr_test(::std::stringify!($group), release, ::std::stringify!($name), None, $expected);
            }
        );
    };
}

macro_rules! test_property {
    ($group:ident, $target:ident, $name:ident, $property_ident:ident, $property:expr, $expected:literal) => {
        ::pastey::paste!(
            #[test]
            fn [<avr_ $group _ $name _ $target:lower _ $property_ident>]() {
                let release = match ::std::stringify!($target) {
                    "Debug" => false,
                    "Release" => true,
                    _ => panic!("Unknown target"),
                };
                $crate::programs_2025_04::avr_test(::std::stringify!($group), release, ::std::stringify!($name), Some(String::from($property)), $expected);
            }
        );
    };
}

macro_rules! test_property_ignore {
    ($group:ident, $target:ident, $name:ident, $property_ident:ident, $property:expr, $expected:literal) => {
        ::pastey::paste!(
            #[test]
            #[ignore]
            fn [<avr_ $group _ $name _ $target:lower _ $property_ident>]() {
                let release = match ::std::stringify!($target) {
                    "Debug" => false,
                    "Release" => true,
                    _ => panic!("Unknown target"),
                };
                $crate::programs_2025_04::avr_test(::std::stringify!($group), release, ::std::stringify!($name), Some(String::from($property)), $expected);
            }
        );
    };
}

mod disprove_inherent;
mod normal;
mod toy;
