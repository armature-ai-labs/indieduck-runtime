# Validation

IndieDuck Cyber Edition (design R25) simulation contributor study, verified locally on 14 October 2026 with Rust 1.91.1 and native MuJoCo 3.10.0 on macOS (arm64). No physical hardware was opened.

- Model: `indieduck-cyber` synced from indieduck-rl (contract SHA-256 `7e81eb31f3bb57ff67419a1b9ee3908a8d79ce0eef9fdc2dc4c80016da604027`), with 64 seeded native forward-kinematics cases regenerated against MuJoCo 3.10.0.
- Tests: the documented default suites pass, 345 tests in total (`cargo test --locked`), including kinematics against native MuJoCo on the R25 model and contract identity checks.
- Release daemon: physical startup is rejected before any I/O. Against the native R25 server, 500 read/write round trips have a median of 0.35 ms (p99 0.87 ms). Steady control held 49.9 Hz with zero missed ticks and zero bus errors. Explicit initialisation reached native torque. The deliberate reset test removes the simulator mid-run: the daemon reports itself unhealthy after consecutive bus failures and requires a restart, as designed.
- Evidence: `validation/native-runtime-r25.json`. R20 records remain in `validation/` and at the tag for that release.

Not verified: any physical servo, IMU, power or calibration behaviour; trained policies.

The inherited physical profile conflates regulated 5 V servo telemetry with 2S pack voltage, and its home pose, limits and calibration are not IndieDuck measurements; physical profiles stay blocked.
