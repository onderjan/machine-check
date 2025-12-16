fn main() {
    let (exec_args, system_args) = machine_check::parse_args(std::env::args());
    machine_check_riscv::execute(exec_args, system_args);
}
