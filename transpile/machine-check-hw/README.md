# machine-check-hw: formal verification of hardware digital systems

[machine-check](https://docs.rs/machine-check) is a tool for formal verification of digital system properties, currently in a proof-of-concept stage. 

This crate provides support for verification of hardware systems written in the [Btor2 language](https://doi.org/10.1007/978-3-319-96145-3_32) using the formal verification tool [machine-check](https://docs.rs/machine-check). The systems can be verified against [Computation Tree Logic](https://en.wikipedia.org/wiki/Computation_tree_logic) properties.

**Hardware verification via machine-check is in a proof-of-concept stage, with poor performance compared to state-of-the-art tools for hardware system verification. Use tools such as [ABC](https://github.com/berkeley-abc/abc) or [AVR](https://github.com/aman-goel/avr) if you want to actually verify Btor2 systems.**

# Installation

To install the tool, execute
```console
$ cargo install machine-check-hw
```

If it was correctly installed and added to your system path, execute
```console
$ machine-check-hw --version
```
The next step is optional, but necessary for reasonable performance. The tool translates the system to a finite-state machine written in Rust, abstracts it (replacing variable types with types that represent sets of variable values), creates another machine for refinement of the abstraction, compiles the resulting machine using Cargo or rustc together with verification logic, and runs the executable. 

To avoid downloading all of the libraries for every machine compilation, first run
```console
$ machine-check-hw prepare
```
This will create a new directory `machine-check-preparation` in the executable installation directory, which contains all of the libraries and allows using only rustc for building, speeding up compilation and avoiding subsequent Internet downloads, allowing for offline verification.

In case something goes wrong with preparation, you can revert back to no-preparation mode by running
```console
$ machine-check-hw prepare --clean
```

# Reachability verification

To actually verify something, obtain a simple Btor2 system, e.g. [recount4.btor2](https://gitlab.com/sosy-lab/research/data/word-level-hwmc-benchmarks/-/blob/991551e58cfc85358dc820fd98ecbd9a1e7e28f8/bv/btor2/btor2tools-examples/recount4.btor2). By pointing machine-check-hw to a Btor2 file, it will by default verify reachability properties specified in that file:
```console
$ machine-check-hw verify ./recount4.btor2
[2023-10-16T19:18:41Z INFO  machine_check::verify::work] Transcribing the system into a machine.
[2023-10-16T19:18:41Z INFO  machine_check::verify::work] Building a machine verifier.
[2023-10-16T19:18:42Z INFO  machine_check::verify::work] Executing the machine verifier.
[2023-10-16T19:18:42Z INFO  machine_check_exec] Starting verification.
[2023-10-16T19:18:42Z INFO  machine_check_exec] Verification ended.
[2023-10-16T19:18:42Z INFO  machine_check::verify] Used 18 states and 44 refinements.
[2023-10-16T19:18:42Z INFO  machine_check::verify] Reached conclusion: false
```

# CTL property verification

Instead of verifying reachability properties in the file, we can verify custom Computation Tree Logic properties. In the property, we can use single-bit states as atomic labellings. For example, the following Btor2 file has three bead positions and a single input. If the input is 0, the beads stay in their positions, if it is 1, they move to the next position:
```console
1 sort bitvec 1 ; bit type

10 zero 1 ; bit zero
11 one 1 ; bit one

20 input 1 ; only input

100 state 1 ; first bead position
101 init 1 100 11 ; present at first

200 state 1 ; second bead position
201 init 1 200 10 ; absent at first

300 state 1 ; third bead position
301 init 1 300 10 ; absent at first

; move beads to the next position if input is 1
1100 ite 1 20 300 100
1101 next 1 100 1100
2100 ite 1 20 100 200
2101 next 1 200 2100
3100 ite 1 20 200 300
3101 next 1 300 3100
```

Saving the code snippet to file `beads.btor2`, we can now verify various properties. 

*Note that the written meanings are not formally precise, they are written for a layperson to understand.*

```console
$ machine-check-hw verify ./beads.btor2 --property EG[node_200]
(...)
[2023-10-16T20:33:40Z INFO  machine_check::verify] Used 3 states and 0 refinements.
[2023-10-16T20:33:40Z INFO  machine_check::verify] Reached conclusion: false
```
Meaning: there does not exist a path where the bead is always in the second position, as it is in the first position in the first state.

```console
$ machine-check-hw verify ./beads.btor2 --property EF[EG[node_200]]
(...)
[2023-10-16T20:36:53Z INFO  machine_check::verify] Used 7 states and 2 refinements.
[2023-10-16T20:36:53Z INFO  machine_check::verify] Reached conclusion: true
```
Meaning: there exists a future where the bead moves to second position and then stays there. 

```console
$ machine-check-hw verify ./beads.btor2 --property AF[node_200]
(...)
[2023-10-16T20:34:13Z INFO  machine_check::verify] Used 5 states and 1 refinements.
[2023-10-16T20:34:13Z INFO  machine_check::verify] Reached conclusion: false
```
Meaning: we cannot be sure that the bead will ever move to the second position.

```console
$ machine-check-hw verify ./beads.btor2 --property EU[node_100,node_200]
(...)
[2023-10-16T20:42:21Z INFO  machine_check::verify] Used 5 states and 1 refinements.
[2023-10-16T20:42:21Z INFO  machine_check::verify] Reached conclusion: true
```
Meaning: there exists a path where the the bead is in the first position until it is in the second position, and the second position is reached.

```console
$ machine-check-hw verify ./beads.btor2 --property AU[node_100,node_200]
(...)
[2023-10-16T20:47:33Z INFO  machine_check::verify] Used 5 states and 1 refinements.
[2023-10-16T20:47:33Z INFO  machine_check::verify] Reached conclusion: false
```
Meaning: the bead stops being in the first position before being in the second position (which we know is not possible), or the second position is never reached (which is possible if it stays in the first position forever).


Logical connectives `not`, `or`, `and` are supported as well. Due to the currently simple parsing, they are written as if they were functions. You can use brackets, parentheses, or curly braces to visually differentiate the various nesting levels (subject to escaping their behaviour in your console). Spaces are not permitted. The CTL property format may (and probably will) change in the future.

```console
$ machine-check-hw verify ./beads.btor2 --property "AG[or(not(node_200),EX[node_300])]"
(...)
[2023-10-16T20:54:17Z INFO  machine_check::verify] Used 7 states and 4 refinements.
[2023-10-16T20:54:17Z INFO  machine_check::verify] Reached conclusion: true

$ machine-check-hw verify ./beads.btor2 --property "AG[or(not(node_200),EX[node_100])]"
(...)
[2023-10-16T20:55:32Z INFO  machine_check::verify] Used 5 states and 1 refinements.
[2023-10-16T20:55:32Z INFO  machine_check::verify] Reached conclusion: false
```
Meaning: the bead being in the second position implies that the bead can move to the third position in the next state. However, it cannot move to the first position so quickly.

# Verification strength and power

The tool is intended to be sound and complete, i.e. give you the correct results in finite time. However, it may fail to do so in reasonable time or memory usage for larger systems. Absence of bugs is also not guaranteed. Proceed with caution.

# Compatibility

machine-check-hw still in an early stage of development and thus subject to drastic changes. SemVer compatibility will be on best-effort basis.

# License

This tool and its constituent crates [machine-check](https://docs.rs/machine-check), [mck](https://docs.rs/mck), [machine-check-common](https://docs.rs/machine-check-common), and [machine-check-exec](https://docs.rs/machine-check-exec) are licensed under Apache 2.0 License or MIT License at your discretion.

# See also

[btor2rs](https://docs.rs/btor2rs): Btor2 parsing library written for use in machine-check