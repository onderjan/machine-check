mod model_check;
mod precision;
mod space;

use std::collections::VecDeque;

use log::{error, info, log_enabled};
use machine_check_common::{Culprit, Error, ExecResult, Info};
use mck::{
    mark::{Join, MarkMachine, MarkState},
    AbstractMachine, MarkBitvector,
};
use model_check::{safety_proposition, Proposition};

use self::{precision::Precision, space::Space};

use clap::Parser;
use mck::FieldManipulate;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    batch: bool,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long)]
    property: Option<String>,
}

pub fn run<M: MarkMachine>() {
    if let Err(err) = run_inner::<M>() {
        // log root error
        error!("{:#?}", err);
        // terminate with non-success code
        std::process::exit(-1);
    }
    // terminate successfully, the information is in stdout
}

fn run_inner<M: MarkMachine>() -> Result<ExecResult, anyhow::Error> {
    // if not run in batch mode, log to stderr with env_logger
    let args = Args::parse();
    if !args.batch {
        let filter_level = match args.verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };

        env_logger::builder().filter_level(filter_level).init();
    }
    info!("Starting verification.");

    let verification_result = verify::<M>(args.property.as_ref());

    if log_enabled!(log::Level::Info) {
        // the result will be propagated, just inform that we ended somehow
        match verification_result.conclusion {
            Ok(_) => info!("Verification ended."),
            Err(_) => error!("Verification failed."),
        }
    }

    // serialize the verification result to stdout
    serde_json::to_writer(std::io::stdout(), &verification_result)?;
    Ok(verification_result)
}

fn verify<M: MarkMachine>(property: Option<&String>) -> ExecResult {
    let mut refinery = Refinery::<M>::new();
    let proposition = if let Some(property_str) = property {
        match Proposition::parse(property_str) {
            Ok(prop) => prop,
            Err(err) => {
                return ExecResult {
                    conclusion: Err(err),
                    info: refinery.info(),
                }
            }
        }
    } else {
        safety_proposition()
    };
    loop {
        let result = model_check::check_prop(&refinery.space, &proposition);
        // if verification was incomplete, try to refine the culprit
        let culprit = match result {
            Ok(conclusion) => {
                return ExecResult {
                    conclusion: Ok(conclusion),
                    info: refinery.info(),
                }
            }
            Err(err) => match err {
                Error::Incomplete(culprit) => culprit,
                _ => {
                    return ExecResult {
                        conclusion: Err(err),
                        info: refinery.info(),
                    }
                }
            },
        };
        if !refinery.refine(&culprit) {
            // it really is incomplete
            return ExecResult {
                conclusion: Err(Error::Incomplete(culprit)),
                info: refinery.info(),
            };
        }
    }
}

struct Refinery<M: MarkMachine> {
    precision: Precision<M>,
    space: Space<M::Abstract>,
    num_refinements: usize,
}

impl<M: MarkMachine> Refinery<M> {
    fn new() -> Self {
        let mut refinery = Refinery {
            precision: Precision::new(),
            space: Space::new(),
            num_refinements: 0,
        };
        // generate first space
        refinery.regenerate_init();
        refinery
    }

    fn refine(&mut self, culprit: &Culprit) -> bool {
        self.num_refinements += 1;
        // compute marking
        let mut current_state_mark = M::State::new_unmarked();
        let mark_bit = current_state_mark.get_mut(&culprit.name).unwrap();
        *mark_bit = MarkBitvector::new_marked();

        // try increasing precision of the state preceding current mark
        let previous_state_iter = culprit.path.iter().cloned().rev().skip(1);
        let current_state_iter = culprit.path.iter().cloned().rev();
        let iter = previous_state_iter.zip(current_state_iter);

        for (previous_state_index, current_state_index) in iter {
            let previous_state = self.space.get_state_by_index(previous_state_index);

            let input = &self
                .space
                .get_representative_step_input(previous_state_index, current_state_index);

            // step using the previous state as input
            let (new_state_mark, input_mark) =
                <M as MarkMachine>::next((previous_state, input), current_state_mark);
            let previous_state_precision = self.precision.get_for_state_mut(previous_state_index);

            let mut joined_precision: M::Input = previous_state_precision.clone();
            Join::apply_join(&mut joined_precision, input_mark);
            if previous_state_precision != &joined_precision {
                *previous_state_precision = joined_precision;
                // regenerate step from the state
                let mut queue = VecDeque::new();
                queue.push_back(previous_state_index);
                self.regenerate_step(queue);
                return true;
            }

            current_state_mark = new_state_mark;
        }

        let initial_state = culprit
            .path
            .front()
            .expect("culprit should have an initial state");

        let init_input = self.space.get_representative_init_input(*initial_state);

        // increasing state precision failed, try increasing init precision
        let (input_mark,) = <M as MarkMachine>::init((init_input,), current_state_mark);

        let init_precision = self.precision.get_init_mut();
        let mut joined_precision: M::Input = init_precision.clone();
        Join::apply_join(&mut joined_precision, input_mark);
        if *init_precision != joined_precision {
            *init_precision = joined_precision;
            // regenerate init
            self.regenerate_init();
            return true;
        }

        // incomplete
        false
    }

    pub fn regenerate_init(&mut self) {
        // remove initial states
        self.space.remove_initial_states();

        // regenerate them using init function with init precision
        // remember the states that were actually added
        let mut added_states_queue = VecDeque::new();

        let initial_precision = self.precision.get_init();
        for input in M::input_precision_iter(initial_precision) {
            let (initial_state_id, added) = self
                .space
                .add_initial_state(M::Abstract::init(&input), &input);
            if added {
                added_states_queue.push_back(initial_state_id);
            }
        }

        // generate every state that was added
        self.regenerate_step(added_states_queue);
    }

    pub fn regenerate_step(&mut self, mut queue: VecDeque<usize>) {
        // construct state space by breadth-first search
        while let Some(current_state_index) = queue.pop_front() {
            // remove outgoing edges
            self.space.remove_outgoing_edges(current_state_index);

            // prepare state and precision
            let state = self.space.get_state_by_index(current_state_index).clone();
            let step_precision = self.precision.get_for_state(current_state_index);

            // generate direct successors
            for input in M::input_precision_iter(&step_precision) {
                let next_state = M::Abstract::next(&state, &input);

                let (next_state_index, added) =
                    self.space.add_step(current_state_index, next_state, &input);

                if added {
                    // add to queue
                    queue.push_back(next_state_index);
                }
            }
        }
    }

    pub fn info(&self) -> Info {
        Info {
            num_states: self.space.num_states(),
            num_refinements: self.num_refinements,
        }
    }
}
