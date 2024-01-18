use simple_xml_builder::XMLElement;

use crate::{RunStats, TaskStats, TestConfig};

pub(super) fn generate(test_config: &TestConfig, tests: &Vec<(TaskStats, RunStats)>) -> XMLElement {
    let mut xml_result = XMLElement::new("result");
    // add required attributes
    let tool_name = match test_config.tool {
        crate::Tool::MachineCheck => "machine-check",
        crate::Tool::Abc => "ABC",
        crate::Tool::Avr => "AVR",
    };
    xml_result.add_attribute("tool", tool_name);
    xml_result.add_attribute("toolmodule", tool_name);
    xml_result.add_attribute("version", test_config.version.to_string());
    xml_result.add_attribute(
        "benchmarkname",
        test_config
            .spec
            .rsplit_once('/')
            .map(|(_, file)| file)
            .unwrap_or(test_config.spec.as_str()),
    );
    xml_result.add_attribute("starttime", "");
    xml_result.add_attribute("date", "");
    xml_result.add_attribute("generator", "machine-check-table");
    // not required by DTD, but table-generator fails without options
    xml_result.add_attribute(
        "options",
        test_config
            .options
            .as_ref()
            .map(|a| a.to_string())
            .unwrap_or_default(),
    );

    xml_result.add_child(generate_xml_columns());
    xml_result.add_child(generate_xml_systeminfo(test_config));

    for test in tests {
        generate_run(&test.0, &test.1);
    }

    xml_result
}

fn generate_run(task_stats: &TaskStats, run_stats: &RunStats) -> XMLElement {
    let status = run_stats.result.map(|result| match result {
        crate::RunResult::Correct(verdict) => verdict.to_string(),
        crate::RunResult::Wrong(verdict) => verdict.to_string(),
        crate::RunResult::Timeout => String::from("TIMEOUT"),
        crate::RunResult::OutOfMemory => String::from("OUT OF MEMORY"),
        crate::RunResult::OtherError => String::from("ERROR"),
    });

    let category = run_stats.result.map(|result| match result {
        crate::RunResult::Correct(_) => "correct",
        crate::RunResult::Wrong(_) => "wrong",
        crate::RunResult::Timeout
        | crate::RunResult::OutOfMemory
        | crate::RunResult::OtherError => "error",
    });

    let mut xml_run = XMLElement::new("run");
    xml_run.add_attribute("name", run_stats.name.clone());
    xml_run.add_attribute("expectedVerdict", task_stats.expected_verdict);
    if let Some(status) = status {
        xml_run.add_child(generate_xml_column("status", Some(&status)));
    }
    if let Some(category) = category {
        xml_run.add_child(generate_xml_column("category", Some(category)));
    }
    if let Some(cpu_time) = run_stats.cpu_time {
        xml_run.add_child(generate_xml_column(
            "cputime",
            Some(&format!("{:.6}s", cpu_time)),
        ));
    }
    if let Some(wall_time) = run_stats.wall_time {
        xml_run.add_child(generate_xml_column(
            "walltime",
            Some(&format!("{:.6}s", wall_time)),
        ));
    }
    if let Some(memory) = run_stats.memory {
        xml_run.add_child(generate_xml_column("memory", Some(&format!("{}B", memory))));
    }
    xml_run
}

fn generate_xml_systeminfo(test_config: &TestConfig) -> XMLElement {
    let mut xml_systeminfo = XMLElement::new("systeminfo");

    let mut xml_os = XMLElement::new("os");
    xml_os.add_attribute(
        "name",
        test_config.system_info.os_name.clone().unwrap_or_default(),
    );
    xml_systeminfo.add_child(xml_os);

    let mut xml_cpu = XMLElement::new("cpu");
    xml_cpu.add_attribute(
        "cores",
        test_config
            .limits
            .cpu_cores
            .map(|u| u.to_string())
            .unwrap_or_else(|| {
                test_config
                    .system_info
                    .cpu_cores
                    .map(|u| u.to_string())
                    .unwrap_or_default()
            }),
    );
    xml_cpu.add_attribute(
        "frequency",
        test_config
            .system_info
            .cpu_frequency
            .map(|u| u.to_string())
            .unwrap_or_default(),
    );
    xml_cpu.add_attribute(
        "model",
        test_config
            .system_info
            .cpu_model
            .clone()
            .unwrap_or_default(),
    );
    xml_systeminfo.add_child(xml_cpu);

    let mut xml_ram = XMLElement::new("ram");
    xml_ram.add_attribute(
        "size",
        test_config
            .limits
            .ram_size
            .map(|u| u.to_string())
            .unwrap_or_else(|| {
                test_config
                    .system_info
                    .ram_size
                    .map(|u| u.to_string())
                    .unwrap_or_default()
            }),
    );
    xml_systeminfo.add_child(xml_ram);

    xml_systeminfo.add_child(XMLElement::new("environment"));

    xml_systeminfo
}

fn generate_xml_column(title: &str, value: Option<&str>) -> XMLElement {
    let mut xml_column = XMLElement::new("column");
    xml_column.add_attribute("title", title);
    if let Some(value) = value {
        xml_column.add_attribute("value", value);
    }
    xml_column
}

fn generate_xml_columns() -> XMLElement {
    let mut xml_columns = XMLElement::new("columns");
    xml_columns.add_child(generate_xml_column("status", None));
    xml_columns.add_child(generate_xml_column("cputime", None));
    xml_columns.add_child(generate_xml_column("walltime", None));
    xml_columns
}
