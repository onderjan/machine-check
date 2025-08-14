use std::collections::BTreeSet;
use std::sync::LazyLock;

use log::{trace, warn};
use machine_check_common::ExecError;

use crate::model_check::property_checker::labelling_updater::LabellingUpdater;
use crate::model_check::property_checker::PropertyChecker;
use crate::space::StateSpace;
use crate::FullMachine;

const INCREMENTAL_DOUBLE_CHECK_ENV_VAR: &str = "MACHINE_CHECK_INCREMENTAL_DOUBLE_CHECK";

static INCREMENTAL_DOUBLE_CHECK: LazyLock<bool> = LazyLock::new(|| {
    let Some(should_double_check) = std::env::var_os(INCREMENTAL_DOUBLE_CHECK_ENV_VAR) else {
        return false;
    };

    let result = should_double_check == "1" || should_double_check.eq_ignore_ascii_case("true");
    if result {
        warn!("Double-checking incremental computation due to environment variable '{}'. This will be slow.", INCREMENTAL_DOUBLE_CHECK_ENV_VAR);
    }
    result
});

impl PropertyChecker {
    pub(super) fn incremental_double_check<M: FullMachine>(
        &mut self,
        space: &StateSpace<M>,
    ) -> Result<(), ExecError> {
        if !*INCREMENTAL_DOUBLE_CHECK {
            return Ok(());
        }
        self.double_check(space)
    }

    pub fn double_check<M: FullMachine>(&self, space: &StateSpace<M>) -> Result<(), ExecError> {
        trace!(
            "Double-checking whether the incremental computation corresponds to non-incremental"
        );

        let mut fresh_property_checker = self.clone();
        fresh_property_checker.invalidate();
        fresh_property_checker.focus.make_whole_dirty(space);
        LabellingUpdater::new(&mut fresh_property_checker, space)?.compute_inner()?;

        // retain only states in state space for comparison
        // as it is allowed for incremental model checking to retain states
        // that are no longer in the state space

        let mut incremental_property_checker = self.clone();

        let states = BTreeSet::from_iter(space.states());

        for history in incremental_property_checker.histories.values_mut() {
            history.retain_states(&states);
        }

        // squash both as the slack might be different
        incremental_property_checker.squash()?;
        fresh_property_checker.squash()?;

        let computation_inconsistencies =
            fresh_property_checker.computations != incremental_property_checker.computations;
        let history_inconsistencies =
            fresh_property_checker.histories != incremental_property_checker.histories;

        if !computation_inconsistencies && !history_inconsistencies {
            return Ok(());
        }
        eprintln!("Double-checking found inconsistencies in incremental model-checking, current focus: {:?}", incremental_property_checker.focus);
        if computation_inconsistencies {
            eprintln!(
                "Double-checking found inconsistencies in computations\n\
                    Fresh computations: {:#?}\n\
                    Incremental computations: {:#?}",
                fresh_property_checker.computations, incremental_property_checker.computations,
            );
        } else {
            eprintln!(
                "No inconsistencies in computations:\n{:#?}",
                incremental_property_checker.computations,
            );
        }
        if history_inconsistencies {
            eprintln!(
                "Double-checking found inconsistencies in histories\n\
                    Fresh histories: {:#?}\n\
                    Incremental histories: {:#?}",
                fresh_property_checker.histories, incremental_property_checker.histories,
            );
        } else {
            eprintln!(
                "No inconsistencies in histories:\n{:#?}",
                incremental_property_checker.histories,
            );
        }
        panic!("Found inconsistencies when double-checking incremental model checking");
    }
}
