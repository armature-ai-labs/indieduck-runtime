# Validation and current limitations

This is a source integration base. The complete workspace does **not** currently build from this filtered snapshot. No robot connection, motor command, deployment, firmware installation, GPU job, training, policy evaluation or physical test was performed.

## Checks performed

- All 183 imported upstream files match their pinned source hashes; source files were not modified.
- `cargo metadata --offline --no-deps --format-version 1` successfully reads the 19-crate workspace. This checks manifest structure, not compilation.
- The one imported Python training file parses with Python 3.14.6. The runtime's actual Rust/MSRV and target toolchains were not tested.
- A bounded scan found no GitHub/Hugging Face token patterns or private-key blocks. Public upstream example and fixture paths are retained and recorded in `provenance/validation.json`.
- No `.github` workflows, model weights, mesh/model files, music scores, `.pyc` caches or Git history are present.

## Known compile and test blockers

| Source reference | Missing input and consequence |
| --- | --- |
| `kinematics/src/lib.rs:36` | `kinematics/assets/alpha/robot_walk.xml` is embedded at compile time. The kinematics crate and its dependants cannot compile without an appropriately licensed model. |
| `robotctl/src/duck.rs:24` | `robotctl/assets/duck.bin` is an embedded baked robot mesh. The renderer cannot compile as supplied. |
| `sounds/src/chorale/mod.rs:481,491,507` | The three score assets are embedded at compile time, including the excluded copyrighted Outer Wilds arrangement. The sound crate cannot compile as supplied. |
| `kinematics/tests/fk_against_mujoco.rs:37` | The original model-derived FK fixture is omitted. It must be regenerated for the selected licensed robot model. |
| `robotd-params/src/lib.rs:1372`, `robotctl/src/configure.rs:889` | Tests embed the omitted upstream deployment configuration. |
| `updater/src/config.rs:446,495` | Tests embed the omitted upstream updater example configuration. |

Detector and motion-policy implementations load omitted weights by path at runtime. Their source being present does not supply a usable policy or prove model transfer. The packaging/updater tooling also expects omitted policies, deployment files and scripts. Upstream release targets, platform assumptions and source APIs remain unadapted.

Full tests were not run: the known missing assets already prevent the complete workspace from compiling. Before integration, supply a canonical, appropriately licensed IndieDuck simulator model, adapt affected code and fixtures, decide which runtime features to retain, configure deployment deliberately, then build and test on the selected target. Do not copy restricted geometry or music merely to make the build pass.
