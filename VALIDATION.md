# Validation and current limitations

R20 simulation contributor study, verified locally on 9 October 2026 with Rust 1.91.1 and native MuJoCo 3.10.0 on macOS. No physical hardware was operated.

## Executed checks

- `cargo test --locked --offline`: 345 tests pass, zero failures. Two manual kinematics benchmark tests remain intentionally ignored; they are not correctness checks.
- Affected daemon, adapter, parameters, odometry and FK suite: 236 passing tests, included in the default total. Protocol library: 49; sound core: 60.
- Optimized `robotd` and `sim_probe` build with bundled score assets disabled.
- Regenerated FK oracle: 64 seeded poses, all 70 sites, native MuJoCo position/orientation tolerance 1e-6.
- Tests reject wrong model, old R07 revision, contract version/hash, joint ordering, invalid telemetry, malformed replies, reset, timeout and disconnect. Faults latch and cached IMUs become unavailable.
- The real release daemon connected to the native R20 server for 100 healthy ticks with zero missed ticks and zero bus errors. Torque stayed off until explicit `robot.init`; reset disabled torque, invalidated the IMU and required daemon restart.
- The release executable rejected default startup, a serial-port override and serial `init` before loading hardware configuration or opening IO. Simulation cannot fall back to serial.
- Native TCP probe: 50 warmup and 500 measured 15-joint read/write pairs, median 0.407 ms, p95 0.536 ms, p99 0.802 ms, maximum 0.971 ms. This is one local Mac run using native XML PD, not BAM timing, a Radxa benchmark, serial-bus timing or a guarantee for other hosts.

The contract SHA-256 is `fdb633cacc80985bd7cb5e2dfc759e7a60eb7ba140a5b663a87c7b7259606345`. Generated constants, XML and fixtures share that contract. No R07 crank equation remains in the current simulation mouth mapping.

Evidence: `validation/r20-runtime.json`, `validation/r20-default-tests.txt`, `validation/r20-release-build.txt` and `validation/native-runtime-r20.json`. Full reproduction commands are in [R20-STUDY.md](R20-STUDY.md). Other R07 logs and source-import inventories are retained as historical evidence, not current verification.

## Supported scope

Default Cargo members are the simulation daemon and checked libraries. `--sim HOST:PORT --no-policy` is the supported native simulation connection; `--fake` supports software tests. Physical startup is blocked without an override.

The retained full workspace still has auxiliary gaps: `robotctl` references omitted renderer/configuration assets, updater/packaging tests need deployment fixtures, and policies/detector weights are absent. `cargo test --workspace` is not a verified release workflow. Optional `bundled-scores` requires separately licensed files and stays disabled.

The inherited physical profile conflates regulated 5 V servo telemetry with 2S pack voltage, and its home pose, limits and calibration have not been validated on R20 hardware. A reviewed pack-voltage input and physical profile must precede a hardware release. Geometry-based direct-drive mapping is not encoder calibration. Printed fit, cable motion, strength and learned behavior remain unverified.
