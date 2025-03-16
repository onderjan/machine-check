use std::time::Duration;

use chrono::{DateTime, Local, Timelike};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlTableRowElement};

use crate::{
    frontend::{
        util::web_idl::{document, get_element_by_id},
        view::View,
    },
    shared::snapshot::log::MessageType,
};

pub fn display(view: &View) {
    LogDisplayer::new(view).display();
}

struct LogDisplayer<'a> {
    view: &'a View,
    log_wrapper_element: Element,
    log_body_element: Element,
}

impl LogDisplayer<'_> {
    fn new(view: &View) -> LogDisplayer {
        let log_wrapper_element = get_element_by_id("log_wrapper");
        let log_body_element = get_element_by_id("log").first_element_child().unwrap();
        let log_body_element = log_body_element.dyn_into().unwrap();
        LogDisplayer {
            view,
            log_wrapper_element,
            log_body_element,
        }
    }

    fn display(&self) {
        let previous_num_children = self.log_body_element.child_element_count();

        // remove all children except the first (table header)
        let first_child = self.log_body_element.first_element_child().unwrap();
        self.log_body_element.set_inner_html("");
        self.log_body_element.append_child(&first_child).unwrap();

        for message in &self.view.snapshot().log.messages {
            let row_element: HtmlTableRowElement =
                document().create_element("tr").unwrap().dyn_into().unwrap();

            let time_element = row_element.insert_cell().unwrap();
            let duration_element = row_element.insert_cell().unwrap();
            duration_element.set_text_content(Some("-"));
            let type_element = row_element.insert_cell().unwrap();
            let message_element = row_element.insert_cell().unwrap();

            let local_time: DateTime<Local> = message.time.into();

            time_element.set_text_content(Some(&pretty_time(&local_time)));

            match &message.ty {
                MessageType::Error(msg) => {
                    type_element.set_text_content(Some("Error"));
                    message_element.set_text_content(Some(&format!("Error: {}", msg)));
                }
                MessageType::Step(step_message) => {
                    duration_element
                        .set_text_content(Some(&pretty_duration(&step_message.duration)));
                    type_element.set_text_content(Some("Step"));
                    message_element.set_text_content(Some(&format!(
                        "{}: {} refinements.",
                        match step_message.status {
                            crate::shared::snapshot::log::StepStatus::Completed => "Completed",
                            crate::shared::snapshot::log::StepStatus::Cancelled => "Cancelled",
                        },
                        step_message.num_refinements
                    )));
                }
            }

            self.log_body_element.append_child(&row_element).unwrap();
        }

        let num_children = self.log_body_element.child_element_count();

        if num_children > previous_num_children {
            // new messages, scroll down
            self.log_wrapper_element
                .set_scroll_top(self.log_wrapper_element.scroll_height());
        }
    }
}

fn pretty_time(time: &DateTime<Local>) -> String {
    // just write the time in hours, minutes, and seconds
    // it just serves as a way to recall things
    let time = time.time();
    format!(
        "{:0>2}:{:0>2}:{:0>2}",
        time.hour(),
        time.minute(),
        time.second()
    )
}

fn pretty_duration(duration: &Duration) -> String {
    const SECONDS_IN_MINUTE: u64 = 60;
    const SECONDS_IN_HOUR: u64 = SECONDS_IN_MINUTE * 60;
    let seconds = duration.as_secs();
    let hours = seconds / SECONDS_IN_HOUR;
    let seconds = seconds % SECONDS_IN_HOUR;
    let minutes = seconds / SECONDS_IN_MINUTE;
    let seconds = seconds % SECONDS_IN_MINUTE;

    let millis = duration.subsec_millis();

    if hours == 0 {
        if minutes == 0 {
            format!("{}.{:0>3} s", seconds, millis)
        } else {
            format!("{}:{:0>2}.{:0>3} s", minutes, seconds, millis)
        }
    } else {
        format!("{}:{:0>2}:{:0>2}.{:0>3} s", hours, minutes, seconds, millis)
    }
}
