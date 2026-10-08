# IndieDuck Runtime

Onboard Rust source for the IndieDuck robotics project, maintained by Armature AI Labs.

**Status: initial source integration base. The complete workspace is not currently buildable or deployable from this snapshot.** Licensed IndieDuck robot assets, deployment configuration and tested hardware integration still need to be supplied. No IndieDuck walking, motor-bus timing or policy compatibility is claimed.

This is actual source from [JoyandAI/microduck](https://github.com/JoyandAI/microduck/tree/b6b658a6e2bb888302236f8a5b35844b53685409), based on Pollen Robotics’ MicroDuck, with an explicit file allowlist and exclusions. It is an attributed source snapshot, not a GitHub fork containing the complete upstream history. Existing crate names and upstream package versions are retained to limit unnecessary interface changes. They do not designate an IndieDuck release version.

## Source map

- `duck-control/`: robot I/O, Feetech bus driver (`ftbus.rs`), IMU paths, observations, policy execution and control logic.
- `duck-ipc-proto/`, `duckctl/`, `robotd/`, `robotd-params/`: client protocol, command tools, daemon and parameters.
- `kinematics/`, `robotctl/`: kinematics and management source, currently referencing excluded robot assets.
- Other workspace crates retain upstream device, media, detection and service support. Their presence does not confirm support on the selected IndieDuck hardware.
- `provenance/`: exact upstream commit, included and excluded files, changes, authors and validation evidence.

## What prevents running it

The filtered snapshot omits robot XML/geometry, trained policies and detector models, MIDI/demo assets, deployment files, and upstream publishing workflows. Several crates use `include_str!` or `include_bytes!` for these omitted files, so compiling the full workspace will fail until those dependencies are replaced or removed. [The validation inventory](provenance/validation.json) lists the detected missing compile-time assets. Other target-specific build and runtime dependencies have not been validated.

The selected hardware target is a Radxa Zero 3W with 4 GB RAM/eMMC, Pollen D1 Robot HAT and its integrated IMU, and 15 Feetech HD-1910-C001 servos on a regulated 5.0 V rail. The imported source contains its own upstream hardware variants. Those variants must be reviewed against the actual wiring, IMU interface and actuator contract before motor operation.

## Contributor starting point

1. Review [upstream provenance](provenance/upstream.json), [recorded checks](provenance/validation.json) and [CONTRIBUTING.md](CONTRIBUTING.md).
2. Restore buildability using independently exportable IndieDuck geometry, original configuration examples and redistributable replacements for missing assets. Do not copy restricted assets back into this repository.
3. Validate joint order, signs, limits, observation/action layouts and IMU orientation against the matching RL revision.
4. Bench one servo, then all 15: verify regulated power, read/write behaviour, control-loop timing, fault handling and shutdown before powered robot trials.

The retained workspace declares Rust 1.89 or newer. This lightweight manifest check runs without building or fetching dependencies:

```sh
cargo metadata --offline --locked --no-deps --format-version 1
```

It passed during preparation. Full compilation, automated test suites, device operation and deployment have not passed or been claimed. Upstream scripts and comments may refer to files deliberately excluded here.

## Project repositories

| Project | Repository | Scope |
| --- | --- | --- |
| IndieDuck | [indieduck](https://github.com/armature-ai-labs/indieduck) | Hardware CAD, fit tests and project documentation |
| IndieDuck Runtime | [indieduck-runtime](https://github.com/armature-ai-labs/indieduck-runtime) | Onboard Rust control and device integration |
| IndieDuck RL | [indieduck-rl](https://github.com/armature-ai-labs/indieduck-rl) | Python environments, actuator models, training and export |

The hardware project is **IndieDuck by Armature AI Labs, adapted from Open Duck Mini and inspired by Pollen Robotics’ MicroDuck**. The software in this repository is credited separately below.

## Licensing and credits

Eligible upstream code retains its [Apache-2.0 licence](LICENSE) and applicable third-party terms. Preserve [third-party notices](THIRD_PARTY_NOTICES.md), [upstream attribution](UPSTREAM.md), source headers and the provenance records. JoyandAI, Pollen Robotics and their contributors authored the imported code; Armature AI Labs prepared this filtered source base and the IndieDuck integration roadmap. This repository does not imply upstream endorsement.
