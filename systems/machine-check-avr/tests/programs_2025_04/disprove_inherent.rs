#![cfg(test)]

macro_rules! disprove_inherent {
    ($name:ident) => {
        test_inherent!(disprove_inherent, Debug, $name, false);
        test_inherent!(disprove_inherent, Release, $name, false);
    };
}

disprove_inherent!(get_from_undescribed);
disprove_inherent!(jump_outside);
disprove_inherent!(set_global_interrupt_flag);
disprove_inherent!(set_to_restricted);
disprove_inherent!(set_to_undescribed);
disprove_inherent!(undescribed_instruction);
