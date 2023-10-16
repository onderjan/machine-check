# machine-check - a formal verification tool for digital systems

machine-check is a tool for formal verification of digital system properties, currently in a proof-of-concept stage. Currently, [Computation Tree Logic](https://en.wikipedia.org/wiki/Computation_tree_logic) properties of hardware systems written in the [Btor2 language](https://doi.org/10.1007/978-3-319-96145-3_32) can be verified. In the future, support of more system descriptions is planned, such as machine-code (assembly) and source-code programs.

**The tool is currently in a proof-of-concept stage, with poor performance compared to state-of-the-art tools. Use tools like [ABC](https://github.com/berkeley-abc/abc) or [AVR](https://github.com/aman-goel/avr) if you want to actually verify Btor2 systems.**

# Installation

To try out the tool, run
```console
$ cargo install machine-check
```

The tool translates the system to a finite-state machine written in Rust, abstracts it (i.e. replacing variable types with types that represent sets of variable values), creates another machine for refinement of the abstraction, compiles the resulting machine using Cargo or rustc together with verification logic, and runs the executable. 

To avoid downloading all of the libraries for every machine compilation, first run
```console
$ cargo machine-check prepare
```
This will create a new directory `machine-check-preparation` in the executable installation directory, which contains all of the libraries and allows using only rustc for building, speeding up compilation and avoiding subsequent Internet downloads, allowing for offline verification.

# Reachability verification

To actually verify something, obtain a simple Btor2 system, e.g. [recount4.btor2](https://gitlab.com/sosy-lab/research/data/word-level-hwmc-benchmarks/-/blob/991551e58cfc85358dc820fd98ecbd9a1e7e28f8/bv/btor2/btor2tools-examples/recount4.btor2). By pointing machine-check to a Btor2 file, it will by default verify reachability properties specified in that file:
```
$ cargo machine-check verify ./recount4.btor2
[2023-10-16T19:18:41Z INFO  machine_check::verify::work] Transcribing the system into a machine.
[2023-10-16T19:18:41Z INFO  machine_check::verify::work] Building a machine verifier.
[2023-10-16T19:18:42Z INFO  machine_check::verify::work] Executing the machine verifier.
[2023-10-16T19:18:42Z INFO  machine_check_exec] Starting verification.
[2023-10-16T19:18:42Z INFO  machine_check_exec] Verification ended.
[2023-10-16T19:18:42Z INFO  machine_check::verify] Used 18 states and 44 refinements.
[2023-10-16T19:18:42Z INFO  machine_check::verify] Reached conclusion: false
```

# CTL property verification

Instead of verifying reachability properties in the file, you can verify custom Computation Tree Logic properties. In the property, you can use single-bit states as atomic labellings. For example, the following Btor2 file has three bead positions and a single input. If the input is 0, the beads stay in their positions, if it is 1, they move to the next position:
```
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


