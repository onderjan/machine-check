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
                riscv_test(::std::stringify!($name), None, $expected);
            }
        );
    };
}

macro_rules! test_property {
    ($name:ident, $property_ident:ident, $property:expr, $expected:literal) => {
        ::pastey::paste!(
            #[test]
            fn [<riscv _ $name _ $property_ident>]() {
                riscv_test(::std::stringify!($name), Some(String::from($property)), $expected);
            }
        );
    };
}

// Inherent property.
//
// Verification result should be true so that we can reason about further properties.
test_inherent!(basic, true);

// Always globally in main.
//
// Verification result should be false as we do not start in main.
test_property!(
    basic,
    main_always_globally,
    "AG![pc_within_symbol!(\"main\")]",
    false
);

// Can always reach main.
//
// Verification result should be true as main should be reached after
// initialisation independently of inputs.
test_property!(
    basic,
    main_always_reach,
    "AF![pc_within_symbol!(\"main\")]",
    true
);

// Reaches main and stays in it.
//
// Verification result should be true. Main should never be left.
test_property!(
    basic,
    main_always_stay,
    "AF![AG![pc_within_symbol!(\"main\")]]",
    true
);

// The label 'assign1' can be reached independently of inputs.
//
// Verification result should be true.
test_property!(
    basic,
    assign1_always_reach,
    "AF![pc_at_symbol!(\"assign1\")]",
    true
);

// Stays in the instruction pointed to by the label 'assign1' independently of inputs.
//
// Verification result should be false as the instruction should not loop on itself.
test_property!(
    basic,
    assign1_always_stay,
    "AF![AG![pc_at_symbol!(\"assign1\")]]",
    false
);

// The variable 'button_pressed' value 0 can be reached independently of inputs.
//
// Verification result should be true as the value will be set to zero during init.
test_property!(
    basic,
    button_pressed_always_reach_0,
    "AF![typed_symbol!(\"button_pressed\") == 0]",
    true
);

// The variable 'button_pressed' value is zeroed before reaching main.
//
// Verification result should be true.
//
// We express the property by button_pressed being zero releasing the condition
// that the program counter is not within main.
// For simplicity, it is not checked that main is actually reached in this property.
test_property!(
    basic,
    button_pressed_zeroed_before_main,
    "AR![typed_symbol!(\"button_pressed\") == 0, !pc_within_symbol!(\"main\")]",
    true
);

// The variable 'button_pressed' value 1 can be reached independently of inputs.
//
// Verification result should be false as it might not ever be set to 1.
test_property!(
    basic,
    button_pressed_always_reach_1,
    "AF![typed_symbol!(\"button_pressed\") == 1]",
    false
);

// The variable 'button_pressed' value 1 can be reached with some input.
//
// Verification result should be true as it is possible for it to be set to 1
// as a consequence of an input.
test_property!(
    basic,
    button_pressed_exists_reach_1,
    "EF![typed_symbol!(\"button_pressed\") == 1]",
    true
);

// The variable 'button_pressed' should be always able to recover to value 0
// with some sequence of inputs.
//
// Verification result should be true.
test_property!(
    basic,
    button_pressed_recovery_0,
    "AG![EF![typed_symbol!(\"button_pressed\") == 0]]",
    true
);

// The variable 'button_pressed' should be always able to recover to value 1
// with some sequence of inputs.
//
// Verification result should be true.
test_property!(
    basic,
    button_pressed_recovery_1,
    "AG![EF![typed_symbol!(\"button_pressed\") == 1]]",
    true
);
