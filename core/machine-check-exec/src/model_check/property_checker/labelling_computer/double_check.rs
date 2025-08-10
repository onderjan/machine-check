use std::collections::BTreeSet;
use std::sync::LazyLock;

use log::{trace, warn};
use machine_check_common::ExecError;

use crate::model_check::property_checker::labelling_computer::LabellingComputer;
use crate::FullMachine;

const DOUBLE_CHECK_ENV_VAR: &str = "MACHINE_CHECK_DOUBLE_CHECK";

static DOUBLE_CHECK: LazyLock<bool> = LazyLock::new(|| {
    let Some(should_double_check) = std::env::var_os(DOUBLE_CHECK_ENV_VAR) else {
        return false;
    };

    let result = should_double_check == "1" || should_double_check.eq_ignore_ascii_case("true");
    if result {
        warn!("Double-checking incremental computation due to environment variable '{}'. This will be slow.", DOUBLE_CHECK_ENV_VAR);
    }
    result
});

impl<M: FullMachine> LabellingComputer<'_, M> {
    pub(super) fn perform_double_check(&mut self) -> Result<(), ExecError> {
        if !*DOUBLE_CHECK {
            return Ok(());
        }

        // TODO: double-checking does not take moving of time-points into account
        trace!("Double-checking");

        let mut fresh_property_checker = self.property_checker.clone();
        fresh_property_checker.invalidate();
        fresh_property_checker
            .focus
            .extend_dirty(self.space, self.space.states());
        LabellingComputer::new(&mut fresh_property_checker, self.space)?.compute_inner()?;

        // retain only states in state space for comparison
        // as it is allowed for incremental model checking to retain states
        // that are no longer in the state space

        let states = BTreeSet::from_iter(self.space.states());

        for history in self.property_checker.histories.values_mut() {
            history
                .states
                .retain(|state_id, _| states.contains(state_id));
            for time_map in history.times.values_mut() {
                time_map.retain(|state_id, _| states.contains(state_id));
            }
        }

        let computation_inconsistencies =
            fresh_property_checker.computations != self.property_checker.computations;
        let history_inconsistencies =
            fresh_property_checker.histories != self.property_checker.histories;

        if !computation_inconsistencies && !history_inconsistencies {
            return Ok(());
        }
        eprintln!("Double-checking found inconsistencies in incremental model-checking, current focus: {:?}", self.property_checker.focus);
        if computation_inconsistencies {
            eprintln!(
                "Double-checking found inconsistencies in computations\n\
                    Fresh computations: {:#?}\n\
                    Incremental computations: {:#?}",
                fresh_property_checker.computations, self.property_checker.computations,
            );
        } else {
            eprintln!(
                "No inconsistencies in computations:\n{:#?}",
                self.property_checker.computations,
            );
        }
        if history_inconsistencies {
            eprintln!(
                "Double-checking found inconsistencies in histories\n\
                    Fresh histories: {:#?}\n\
                    Incremental histories: {:#?}",
                fresh_property_checker.histories, self.property_checker.histories,
            );
        } else {
            eprintln!(
                "No inconsistencies in histories:\n{:#?}",
                self.property_checker.histories,
            );
        }
        panic!("Found inconsistencies when double-checking incremental model checking");
    }
}
