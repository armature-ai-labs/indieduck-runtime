# IndieDuck Runtime

Onboard Rust source for the IndieDuck robotics project, maintained by Armature AI Labs.

**Status: R20 simulation contributor study. The `robotd` daemon and its tested dependencies build and connect to the matching native IndieDuck simulator.** See [R20 setup and interfaces](R20-STUDY.md). The entire workspace and physical deployment remain unfinished. No trained IndieDuck walking, motor-bus timing or physical fit is claimed.

This is actual source from [JoyandAI/microduck](https://github.com/JoyandAI/microduck/tree/b6b658a6e2bb888302236f8a5b35844b53685409), based on Pollen Robotics’ MicroDuck, with an explicit file allowlist and exclusions. It is an attributed source snapshot, not a GitHub fork containing the complete upstream history. Existing crate names and upstream package versions are retained to limit unnecessary interface changes. They do not designate an IndieDuck release version.

## Source map

- `duck-control/`: robot I/O, Feetech bus driver (`ftbus.rs`), IMU paths, observations, policy execution and control logic.
- `duck-ipc-proto/`, `duckctl/`, `robotd/`, `robotd-params/`: client protocol, command tools, daemon and parameters.
- `kinematics/`: generated R20 kinematics and native MuJoCo parity tests. `robotctl/` still references excluded renderer assets.
- Other workspace crates retain upstream device, media, detection and service support. Their presence does not confirm support on the selected IndieDuck hardware.
- `provenance/`: exact upstream commit, included and excluded files, changes, authors and validation evidence.

## Scope and remaining build gaps

R20 supplies generated IndieDuck XML and FK fixtures, a protocol-1 simulation adapter and an original default configuration example. The daemon explicitly blocks physical startup; use `--sim HOST:PORT --no-policy` or `--fake`. The default Cargo build targets the study daemon and checked libraries, with optional bundled scores disabled. Trained policies, detector models, deployment files and some auxiliary renderer/updater fixtures remain excluded. Those auxiliary crates still prevent a complete workspace build. [Current validation](VALIDATION.md) distinguishes resolved blockers from the historical [source inventory](provenance/validation.json).

The selected hardware target is a Radxa Zero 3W with 4 GB RAM/eMMC, Pollen D1 Robot HAT and its integrated IMU, and 15 Feetech HD-1910-C001 servos on a regulated 5.0 V rail. The imported source contains its own upstream hardware variants. Those variants must be reviewed against the actual wiring, IMU interface and actuator contract before motor operation.

## Contributor starting point

1. Review [upstream provenance](provenance/upstream.json), [recorded checks](provenance/validation.json) and [CONTRIBUTING.md](CONTRIBUTING.md).
2. Run the checked local study in [R20-STUDY.md](R20-STUDY.md). Finish auxiliary build gaps using appropriately licensed replacements. Do not restore restricted assets.
3. Validate joint order, signs, limits, observation/action layouts and IMU orientation against the matching RL revision.
4. Before enabling a future physical release, bench one servo, then all 15: verify regulated power, read/write behaviour, control-loop timing, fault handling and shutdown before powered robot trials.

The retained workspace declares Rust 1.89 or newer. This lightweight manifest check runs without building or fetching dependencies:

```sh
cargo metadata --offline --locked --no-deps --format-version 1
```

Manifest validation, the R20 daemon build and the documented affected test suites pass locally. Full workspace compilation, device operation and deployment remain unverified. Upstream scripts and comments may refer to files deliberately excluded here.

## Project repositories

| Project | Repository | Scope |
| --- | --- | --- |
| IndieDuck | [indieduck](https://github.com/armature-ai-labs/indieduck) | Project, BOM, assembly guide and build documentation |
| IndieDuck CAD | [indieduck-cad](https://github.com/armature-ai-labs/indieduck-cad) | Authoritative R20 FreeCAD sources, drawings and exports |
| IndieDuck Runtime | [indieduck-runtime](https://github.com/armature-ai-labs/indieduck-runtime) | Onboard Rust control and device integration |
| IndieDuck RL | [indieduck-rl](https://github.com/armature-ai-labs/indieduck-rl) | Python environments, actuator models, training and export |

The hardware project is **IndieDuck by Armature AI Labs, adapted from Open Duck Mini and inspired by Pollen Robotics’ MicroDuck**. The software in this repository is credited separately below.

## Licensing and credits

Eligible upstream code retains its [Apache-2.0 licence](LICENSE) and applicable third-party terms. Preserve [third-party notices](THIRD_PARTY_NOTICES.md), [upstream attribution](UPSTREAM.md), source headers and the provenance records. JoyandAI, Pollen Robotics and their contributors authored the imported code; Armature AI Labs prepared the filtered source base, R20 simulator adapter, generated model interfaces and IndieDuck integration changes. This repository does not imply upstream endorsement.
