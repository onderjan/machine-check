// Tests of the choice example.

// --- CHOICE-SPECIFIC TESTS ---

// The output is set to light up exactly one LED when in main.
//
// Verification result should be false as main starts with no LED lit.
test_property!(
    choice,
    always_one_led_in_main,
    "AG![(!pc_within_symbol!(\"main\")) || (typed_symbol!(\"output\") == 0x1 || typed_symbol!(\"output\") == 0x80)]",
    false
);

// The output is set to light up exactly one LED when at assign2.
//
// Verification result should be true as the 'output' variable is set directly beforehand to either of the values
// depending on the choice made through the input.
test_property!(
    choice,
    always_one_led_at_assign2,
    "AG![(!pc_at_symbol!(\"assign2\")) || (typed_symbol!(\"output\") == 0x1 || typed_symbol!(\"output\") == 0x80)]",
    true
);

// --- COMMON TESTS ---

// Inherent property.
//
// Verification result should be true so that we can reason about further properties.
test_inherent!(choice, true);

// Always globally in main.
//
// Verification result should be false as we do not start in main.
test_property!(
    choice,
    main_always_globally,
    "AG![pc_within_symbol!(\"main\")]",
    false
);

// Can always reach main.
//
// Verification result should be true as main should be reached after
// initialisation independently of inputs.
test_property!(
    choice,
    main_always_reach,
    "AF![pc_within_symbol!(\"main\")]",
    true
);

// Reaches main and stays in it.
//
// Verification result should be true. Main should never be left.
test_property!(
    choice,
    main_always_stay,
    "AF![AG![pc_within_symbol!(\"main\")]]",
    true
);

// The label 'assign1' can be reached independently of inputs.
//
// Verification result should be true.
test_property!(
    choice,
    assign1_always_reach,
    "AF![pc_at_symbol!(\"assign1\")]",
    true
);

// Stays in the instruction pointed to by the label 'assign1' independently of inputs.
//
// Verification result should be false as the instruction should not loop on itself.
test_property!(
    choice,
    assign1_always_stay,
    "AF![AG![pc_at_symbol!(\"assign1\")]]",
    false
);

// The variable 'button_pressed' has value 0 after init.
//
// Verification result should be true as the value will be set to zero during init.
test_property!(
    choice,
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
    choice,
    button_pressed_zeroed_before_main,
    "AR![typed_symbol!(\"button_pressed\") == 0, !pc_within_symbol!(\"main\")]",
    true
);

// The variable 'button_pressed' value 1 can be reached independently of inputs within the main function.
//
// Verification result should be false as it might not ever be set to 1.
test_property!(
    choice,
    button_pressed_always_reach_1_after_init,
    "AG![!(pc_at_symbol!(\"main\")) || AF![typed_symbol!(\"button_pressed\") == 1]]",
    false
);

// The variable 'button_pressed' value 1 can be reached with some input.
//
// Verification result should be true as it is possible for it to be set to 1
// as a consequence of an input.
test_property!(
    choice,
    button_pressed_exists_reach_1_after_init,
    "AG![!(pc_at_symbol!(\"main\")) || EF![typed_symbol!(\"button_pressed\") == 1]]",
    true
);

// The variable 'button_pressed' should be always able to recover to value 0
// with some sequence of inputs while in main.
//
// Verification result should be true.
test_property!(
    choice,
    button_pressed_recovery_0_within_main,
    "AG![!(pc_within_symbol!(\"main\")) || EF![typed_symbol!(\"button_pressed\") == 0]]",
    true
);

// The variable 'button_pressed' should be always able to recover to value 1
// with some sequence of inputs while in main.
//
// Verification result should be true.
test_property!(
    choice,
    button_pressed_recovery_1_within_main,
    "AG![!(pc_within_symbol!(\"main\")) || EF![typed_symbol!(\"button_pressed\") == 1]]",
    true
);
