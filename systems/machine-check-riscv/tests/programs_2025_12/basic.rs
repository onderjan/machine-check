// Tests of the basic example.
//
// Only the common tests.

// --- COMMON TESTS ---

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
