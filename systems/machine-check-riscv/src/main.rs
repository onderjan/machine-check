fn main() {
    // just parse the arguments using the machine-check function
    // and execute the library function in this crate
    let (exec_args, system_args) = machine_check::parse_args(std::env::args());
    machine_check_riscv::execute(exec_args, system_args);
}
