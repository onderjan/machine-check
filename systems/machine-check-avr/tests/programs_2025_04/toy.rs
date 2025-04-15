macro_rules! toy_inherent {
    ($target:ident, $name:ident) => {
        test_inherent!(toy, $target, $name, true);
    };
}

macro_rules! toy_property {
    ($target:ident, $name:ident, $property_ident:ident, $property:literal, $expected:literal) => {
        test_property!(toy, $target, $name, $property_ident, $property, $expected);
    };
}

toy_inherent!(Debug, basic_branch);
toy_property!(
    Debug,
    basic_branch,
    initialisation,
    "AF![((PC == 0x41 && DDRB == 0x02) && PORTB == 0x00)]",
    true
);
toy_property!(
    Debug,
    basic_branch,
    invariant_lock,
    "AG![(!(DDRB == 0x02) || AG![DDRB == 0x02])]",
    true
);
toy_property!(
    Debug,
    basic_branch,
    recovery,
    "AG![EF![(PC == 0x41 && PORTB == 0x00)]]",
    true
);

toy_inherent!(Release, basic_branch);
toy_property!(
    Release,
    basic_branch,
    initialisation,
    "AF![((PC == 0x41 && DDRB == 0x02) && PORTB == 0x00)]",
    true
);
toy_property!(
    Release,
    basic_branch,
    invariant_lock,
    "AG![(!(DDRB == 0x02) || AG![DDRB == 0x02])]",
    true
);
toy_property!(
    Release,
    basic_branch,
    recovery,
    "AG![EF![(PC == 0x41 && PORTB == 0x00)]]",
    true
);

toy_inherent!(Debug, blink);
toy_property!(
    Debug,
    blink,
    initialisation,
    "AF![((PC == 0x05 && DDRB == 0x01) && PORTB == 0x00)]",
    true
);
toy_property!(
    Debug,
    blink,
    invariant_lock,
    "AG![(!(DDRB == 0x02) || AG![DDRB == 0x02])]",
    true
);
toy_property!(
    Debug,
    blink,
    recovery,
    "AG![EF![(PC == 0x05 && PORTB == 0x00)]]",
    true
);

toy_inherent!(Debug, gate_array);
toy_property!(
    Debug,
    gate_array,
    initialisation,
    "AF![((PC == 0x05 && DDRB == 0x1E) && PORTB == 0x00)]",
    true
);
toy_property!(
    Debug,
    gate_array,
    invariant_lock,
    "AG![(!(DDRB == 0x1E) || AG![DDRB == 0x1E])]",
    true
);
toy_property!(
    Debug,
    gate_array,
    recovery,
    "AG![EF![(PC == 0x05 && PORTB == 0x00)]]",
    true
);

toy_inherent!(Debug, independent_nondeterminism);
toy_property!(
    Debug,
    independent_nondeterminism,
    initialisation,
    "AF![((PC == 0x02 && DDRB == 0x00) && R[16] == 0x00)]",
    true
);
toy_property!(
    Debug,
    independent_nondeterminism,
    invariant_lock,
    "AG![(!(DDRB == 0x00) || AG![DDRB == 0x00])]",
    true
);

// independent nondeterminism should not recover
toy_property!(
    Debug,
    independent_nondeterminism,
    recovery,
    "AG![EF![(PC == 0x02 && R[16] == 0x00)]]",
    false
);

toy_inherent!(Debug, momentary_selection);
toy_property!(
    Debug,
    momentary_selection,
    initialisation,
    "AF![((PC == 0x02 && DDRB == 0x04) && PORTB == 0x00)]",
    true
);
toy_property!(
    Debug,
    momentary_selection,
    invariant_lock,
    "AG![(!(DDRB == 0x04) || AG![DDRB == 0x04])]",
    true
);
toy_property!(
    Debug,
    momentary_selection,
    recovery,
    "AG![EF![(PC == 0x02 && PORTB == 0x00)]]",
    true
);
