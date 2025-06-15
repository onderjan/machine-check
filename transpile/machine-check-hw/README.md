# machine-check-hw: formal verification of hardware digital systems

**Machine-check-hw** is a tool for verifying hardware systems in the [Btor2](https://fmv.jku.at/papers/NiemetzPreinerWolfBiere-CAV18.pdf) specification format. It translates the system to the subset of Rust supported by **machine-check**, compiles the resulting Rust package using Cargo or rustc together with verification logic, and runs the executable.

## Quickstart

To install **machine-check-hw**, execute
```console
$ cargo install machine-check-hw
```

The next step is optional, but useful for reasonable performance. To avoid downloading all of the libraries for every machine compilation, first run
```console
$ machine-check-hw prepare
```
This will create a new directory `machine-check-preparation` in the executable installation directory which contains the needed libraries, speeding up compilation and avoiding subsequent downloads, allowing offline verification.

In case something goes wrong with preparation, you can revert back to no-preparation mode by running
```console
$ machine-check-hw prepare --clean
```

It is possible to verify various properties of Btor2 systems using **machine-check-hw**. The inherent property is also verified during property verification: there are no explicit panics created from the Btor2 files, but division and remainder by zero violate the inherent property. For example, consider [`beads.btor2`](https://docs.rs/crate/machine-check-hw/0.5.0/source/examples/beads.btor2) from **machine-check-hw** examples. By pointing machine-check-hw to a Btor2 file, it can verify the safety of the system, as specified in the Btor2 file, with a property `AG![safe == 1]`, which uses a special field `safe`:
```console
$ machine-check-hw verify ./beads.btor2 --property 'AG![safe == 1]'
[2025-06-15T17:12:53Z INFO  machine_check_compile::verify] Transcribing the system into a machine.
[2025-06-15T17:12:53Z INFO  machine_check_compile::verify] Building a machine verifier.
[2025-06-15T17:12:54Z INFO  machine_check_compile::verify] Executing the machine verifier.
[2025-06-15T17:12:54Z INFO  machine_check] Starting verification.
[2025-06-15T17:12:54Z INFO  machine_check::verify] Verifying the inherent property first.
[2025-06-15T17:12:54Z INFO  machine_check::verify] The inherent property holds, proceeding to the given property.
[2025-06-15T17:12:54Z INFO  machine_check::verify] Verifying the given property.
[2025-06-15T17:12:54Z INFO  machine_check] Verification ended.
[2025-06-15T17:12:54Z INFO  machine_check_compile::verify] Stats: Stats { transcription_time: Some(0.000682), build_time: Some(0.9404879), execution_time: Some(0.0755522), prepared: Some(true) }
[2025-06-15T17:12:54Z INFO  machine_check_hw::verify] Used 3 states and 0 refinements.
[2025-06-15T17:12:54Z INFO  machine_check_hw::verify] Reached conclusion: true
```

For more examples, [read the **machine-check** book](https://book.machine-check.org/0.5.0/) and the [chapter on **machine-check-hw**](https://book.machine-check.org/0.5.0/systems/machine-check-hw.html).

