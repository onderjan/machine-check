// Tests of the wait example.
//
// The wait example is like the basic example, but with additional delay spin loop.
// This means verifying it is much harder as it has many more states.
//
// Only the properties that can be verified in reasonable time for tests
// (in tens of seconds) have been retained.

// --- COMMON TESTS (only some) ---

// Inherent property.
//
// Verification result should be true so that we can reason about further properties.
test_inherent!(wait, true);

// Always globally in main.
//
// Verification result should be false as we do not start in main.
test_property!(
    wait,
    main_always_globally,
    "AG![pc_within_symbol!(\"main\")]",
    false
);

// Stays in the instruction pointed to by the label 'assign1' independently of inputs.
//
// Verification result should be false as the instruction should not loop on itself.
test_property!(
    wait,
    assign1_always_stay,
    "AF![AG![pc_at_symbol!(\"assign1\")]]",
    false
);

// The variable 'button_pressed' has value 0 after init.
//
// Verification result should be true as the value will be set to zero during init.
test_property!(
    wait,
    button_pressed_0_after_init,
    "AG![!(pc_at_symbol!(\"main\")) || typed_symbol!(\"button_pressed\") == 0]",
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
    wait,
    button_pressed_zeroed_before_main,
    "AR![typed_symbol!(\"button_pressed\") == 0, !pc_within_symbol!(\"main\")]",
    true
);
