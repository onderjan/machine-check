macro_rules! normal_inherent {
    ($target:ident, $name:ident) => {
        test_inherent_ignore!(normal, $target, $name, true);
    };
}

macro_rules! normal_property {
    ($target:ident, $name:ident, $property_ident:ident, $property:expr, $expected:literal) => {
        test_property_ignore!(
            normal,
            $target,
            $name,
            $property_ident,
            $property,
            $expected
        );
    };
}

// --- FACTORIAL ---

// Debug

normal_inherent!(Debug, factorial);
normal_property!(
    Debug,
    factorial,
    initialisation,
    "AF![((((PC == 0x5A && DDRB == 0x00) && DDRD == 0xFF) && PORTB == 0x00) && PORTD == 0x00)]",
    true
);
normal_property!(
    Debug,
    factorial,
    invariant_lock,
    "AG![(!((DDRB == 0x00 && DDRD == 0xFF)) || AG![(DDRB == 0x00 && DDRD == 0xFF)])]",
    true
);
normal_property!(
    Debug,
    factorial,
    recovery,
    "AG![EF![(PC == 0x5A && (PORTB == 0x00 && PORTD == 0x00))]]",
    false
);
normal_property!(
    Debug,
    factorial,
    stack_min,
    "AG![(as_unsigned(SPH) > 0x08 || (SPH == 0x08 && as_unsigned(SPL) >= 0xDD))]",
    true
);
normal_property!(
    Debug,
    factorial,
    stack_above_min,
    "AG![(as_unsigned(SPH) > 0x08 || (SPH == 0x08 && as_unsigned(SPL) >= 0xDE))]",
    false
);

// Release

normal_inherent!(Release, factorial);
normal_property!(
    Release,
    factorial,
    initialisation,
    "AF![((((PC == 0x55 && DDRB == 0x00) && DDRD == 0xFF) && PORTB == 0x00) && PORTD == 0x00)]",
    true
);
normal_property!(
    Release,
    factorial,
    invariant_lock,
    "AG![(!((DDRB == 0x00 && DDRD == 0xFF)) || AG![(DDRB == 0x00 && DDRD == 0xFF)])]",
    true
);
normal_property!(
    Release,
    factorial,
    recovery,
    "AG![EF![(PC == 0x55 && (PORTB == 0x00 && PORTD == 0x00))]]",
    false
);
normal_property!(
    Release,
    factorial,
    stack_min,
    "AG![(as_unsigned(SPH) > 0x08 || (SPH == 0x08 && as_unsigned(SPL) >= 0xFB))]",
    true
);
normal_property!(
    Release,
    factorial,
    stack_above_min,
    "AG![(as_unsigned(SPH) > 0x08 || (SPH == 0x08 && as_unsigned(SPL) >= 0xFC))]",
    false
);

// --- CALIBRATION ---

macro_rules! calibration_tests {
    ($target:ident, $name:ident, $fixed:literal, $program_counter:literal) => {
        normal_inherent!($target, $name);
        normal_property!(
            $target,
            $name,
            initialisation,
            ::std::concat!(
                "AF![((((PC == ",
                $program_counter,
                " && DDRC == 0x01) && DDRD == 0xFF) && PORTC == 0x00) && PORTD == 0x00)]"
            ),
            true
        );
        normal_property!(
            $target,
            $name,
            invariant_lock,
            "AG![(!((DDRC == 0x01 && DDRD == 0xFF)) || AG![(DDRC == 0x01 && DDRD == 0xFF)])]",
            true
        );
        normal_property!(
            $target,
            $name,
            recovery,
            ::std::concat!(
                "AG![EF![(PC == ",
                $program_counter,
                " && (PORTC == 0x00 && PORTD == 0x00))]]"
            ),
            $fixed
        );
        normal_property!(
            $target,
            $name,
            stack_min,
            "AG![(as_unsigned(SPH) > 0x08 || (SPH == 0x08 && as_unsigned(SPL) >= 0xFD))]",
            true
        );
        normal_property!(
            $target,
            $name,
            stack_above_min,
            "AG![(as_unsigned(SPH) > 0x08 || (SPH == 0x08 && as_unsigned(SPL) >= 0xFE))]",
            false
        );
    };
}

calibration_tests!(Debug, calibration_original, false, 0x46);
calibration_tests!(Release, calibration_original, false, 0x44);
calibration_tests!(Debug, calibration_fixed, true, 0x46);
calibration_tests!(Release, calibration_fixed, true, 0x44);
calibration_tests!(Debug, calibration_complicated_original, false, 0x59);
calibration_tests!(Release, calibration_complicated_original, false, 0x57);
calibration_tests!(Debug, calibration_complicated_fixed, true, 0x59);
calibration_tests!(Release, calibration_complicated_fixed, true, 0x57);
