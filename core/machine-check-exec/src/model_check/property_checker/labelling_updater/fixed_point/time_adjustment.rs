use std::ops::ControlFlow;

use log::{debug, trace};
use machine_check_common::ExecError;

use crate::{
    model_check::property_checker::{labelling_updater::LabellingUpdater, FixedPointComputation},
    FullMachine,
};

impl<M: FullMachine> LabellingUpdater<'_, M> {
    pub(super) fn process_old_end_time(
        &mut self,
        fixed_point_index: usize,
        current_computation_index: usize,
        start_time: u64,
    ) -> Result<ControlFlow<(), Option<u64>>, ExecError> {
        Ok(ControlFlow::Continue(
            match current_computation_index.cmp(&self.property_checker.computations.len()) {
                std::cmp::Ordering::Less => {
                    // we have an old computation
                    let old_computation =
                        &self.property_checker.computations[current_computation_index];
                    if old_computation.fixed_point_index != fixed_point_index
                        || old_computation.start_time != start_time
                    {
                        // invalidate and return
                        trace!("Invalidating as computation does not match");
                        self.invalidate = true;
                        return Ok(ControlFlow::Break(()));
                    }
                    trace!(
                        "Old computation present, only considering focus {:?}",
                        self.property_checker.focus
                    );
                    Some(old_computation.end_time)
                }
                std::cmp::Ordering::Equal => {
                    // we do not have an old computation
                    // all states must be dirty
                    debug!(
                        "Reached the end of history with {}/{} states dirty",
                        self.property_checker.focus.dirty().len(),
                        self.space.num_states()
                    );
                    self.property_checker.focus.make_whole_dirty(self.space);

                    // add the computation
                    self.property_checker
                        .computations
                        .push(FixedPointComputation {
                            fixed_point_index,
                            start_time,
                            end_time: start_time,
                        });
                    assert_eq!(
                        self.next_computation_index,
                        self.property_checker.computations.len()
                    );
                    None
                }
                std::cmp::Ordering::Greater => {
                    panic!(
                        "Computation index {} > {} is not permitted",
                        current_computation_index,
                        self.property_checker.computations.len()
                    );
                }
            },
        ))
    }

    pub(super) fn adjust_end_time_using_old(
        &mut self,
        start_time: u64,
        computation_clone: FixedPointComputation,
    ) -> Result<(), ExecError> {
        trace!(
            "Current computation [{}, {}], old computation [{},{}]",
            start_time,
            self.current_time,
            computation_clone.start_time,
            computation_clone.end_time
        );

        assert!(computation_clone.start_time < computation_clone.end_time);

        match self.current_time.cmp(&computation_clone.end_time) {
            std::cmp::Ordering::Less => {
                // remove any computations remaining within
                while let Some(next_computation) = self
                    .property_checker
                    .computations
                    .get(self.next_computation_index)
                {
                    if next_computation.start_time < computation_clone.end_time
                        && next_computation.end_time <= computation_clone.end_time
                    {
                        self.property_checker
                            .computations
                            .remove(self.next_computation_index);
                    } else {
                        break;
                    }
                }

                self.current_time = computation_clone.end_time;
            }
            std::cmp::Ordering::Equal => {
                // nothing to do
            }
            std::cmp::Ordering::Greater => {
                // invalidate and return
                trace!(
                    "Invalidating as new computation end time is greater: ours {}, old {}",
                    self.current_time,
                    computation_clone.end_time
                );
                self.invalidate = true;
            }
        };
        Ok(())
    }

    pub(super) fn adjust_end_time_padding(
        &mut self,
        current_computation_index: usize,
        computation_clone: FixedPointComputation,
    ) {
        // TODO: decide the padding time
        const PADDING_TIME: u64 = 100;
        self.current_time += PADDING_TIME;

        assert_eq!(computation_clone.start_time, computation_clone.end_time);
        assert!(computation_clone.end_time < self.current_time);

        self.property_checker.computations[current_computation_index].end_time = self.current_time;
    }

    pub(super) fn process_calm(
        &mut self,
        fixed_point_index: usize,
        current_computation_index: usize,
        start_time: u64,
    ) -> Result<bool, ExecError> {
        if !self
            .property_checker
            .closed_form_subproperties
            .contains(&fixed_point_index)
            || !self.calmable_fixed_points.contains(&fixed_point_index)
        {
            return Ok(false);
        }
        // this fixed point is calm, it should have the same start and end time
        let current_computation = &self.property_checker.computations[current_computation_index];
        if current_computation.end_time != start_time {
            // invalidate and return
            trace!("Invalidating as calm fixed-point computation does not match");
            self.invalidate = true;
        } else {
            trace!(
                "Not computing fixed point {} as it is calm",
                fixed_point_index
            );
        }
        Ok(true)
    }
}
