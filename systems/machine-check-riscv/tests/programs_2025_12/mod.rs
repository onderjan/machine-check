//! Tests for RISC-V properties added in December 2025.

use machine_check::{ExecArgs, ExecStrategy};
use machine_check_riscv::SystemArgs;

fn riscv_test(name: &str, property: Option<String>, expected: bool) {
    let elf_file = format!("{}/examples/{name}.elf", env!("CARGO_MANIFEST_DIR"));

    let check_inherent = property.is_none();

    let exec_args = ExecArgs {
        silent: true,
        verbose: 0,
        batch: true,
        gui: false,
        inherent: check_inherent,
        assume_inherent: !check_inherent,
        property,
        strategy: ExecStrategy::Default,
    };

    let exec_result = machine_check_riscv::execute(exec_args, SystemArgs { elf_file });

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
    ($name:ident, $expected:literal) => {
        ::pastey::paste!(
            #[test]
            fn [<riscv_ $name _inherent>]() {
                $crate::programs_2025_12::riscv_test(::std::stringify!($name), None, $expected);
            }
        );
    };
}

macro_rules! test_property {
    ($name:ident, $property_ident:ident, $property:expr, $expected:literal) => {
        ::pastey::paste!(
            #[test]
            fn [<riscv _ $name _ $property_ident>]() {
                $crate::programs_2025_12::riscv_test(::std::stringify!($name), Some(String::from($property)), $expected);
            }
        );
    };
}

mod basic;
mod choice;
mod wait;
