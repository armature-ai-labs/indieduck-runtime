# IndieDuck R07 runtime study

Local review copy, 8 October 2026. Public repositories and R06 remain unchanged.

IndieDuck by Armature AI Labs, adapted from Open Duck Mini and inspired by Pollen Robotics’ MicroDuck. Runtime source retains JoyandAI and Pollen attribution in `UPSTREAM.md` and `THIRD_PARTY_NOTICES.md`. New R07 integration code and generated IndieDuck interfaces are Apache-2.0.

## Run the local study

Start the native MuJoCo 3.10.0 body server from the sibling RL repository:

```sh
cd ../indieduck-rl
PYTHONPATH=src .venv-study/bin/python -m mjlab_microduck.sim.body_server --headless
```

Then, from this runtime directory:

```sh
cargo run -p robotd -- --sim 127.0.0.1:7801 --no-policy --socket /tmp/indieduck-r07.sock
```

The daemon adopts the simulator-reported startup pose with torque off. Its existing local JSON-RPC API accepts `robot.init` to enable simulation actuation and ramp to the canonical bent standing pose, and `robot.relax` to disable it. This is a control integration study with no trained policy. Gravity does not prove balance or walking. The body server permits one controlling connection.

`--sim`, `--fake` and a serial `--port` are mutually exclusive. `--sim` requires `--no-policy`. The standalone serial `init` subcommand refuses simulation and fake modes; use the running daemon's JSON-RPC `robot.init` method. Simulation ignores the default machine configuration unless `--params` is explicitly supplied. Battery shutdown and audio demonstrations are disabled, and simulation cannot invoke host poweroff. The reported 5 V servo rail is not presented as a 2S battery percentage.

## Shared interfaces

`duck-control/src/sim.rs` implements existing newline JSON TCP protocol 1: hello, read, write, gain, torque and slow telemetry. Connection requires the exact MuJoCo 3.10.0 version, IndieDuck model identity, R07 revision, contract version and SHA-256, 15 physical joint names, 14 action joint names and nominal 5 V rail. Mouth stays physical index 9 and is omitted from policy actions. Existing 61-observation code is unchanged. Native current values are zero placeholders, temperature is nominal and voltage is the regulated servo rail. They are not measured battery or thermal data. The head IMU is available through `RemoteIo::head_imu()` as a separate auxiliary sample.

A malformed sample, server error, connection loss, timeout, simulation clock reversal or reset epoch change latches a fault, clears cached IMU samples and closes the socket. There is no automatic reconnect and no serial fallback. Every command includes the connected session and reset epoch, so the server can reject stale commands atomically. Reset the native study as desired, then restart the daemon to adopt its new pose. The server also turns torque off when a controlling connection ends. I/O uses a 100 ms timeout and 64 KiB maximum reply size.

The canonical model lives only in `indieduck-rl/src/mjlab_microduck/robot/indieduck`. Runtime XML, contract, poses, linkage dimensions and FK oracle are generated copies. Resynchronise with:

```sh
../indieduck-rl/.venv-study/bin/python tools/sync_indieduck_model.py ../indieduck-rl/src/mjlab_microduck/robot/indieduck
```

The generator preserves metres/radians in simulation. It derives the simulator home pose from the `stand` record. The inherited `DEFAULT_POSITION` remains the legacy policy observation baseline and hardware homing baseline. R07 trained policies must explicitly reconcile that baseline before removing `--no-policy`.

R07 has a crank-driven jaw. Its analytic conversion is selected only for `--sim`; the inherited physical mouth conversion is preserved. A semantic 0 to 1 mouth request represents 0 to 30 degrees of jaw opening, converted analytically to motor crank travel of 0 to approximately -0.96419 radians. Sending +30 degrees directly to physical joint 9 would be wrong. The linkage equation closes the nominal 12 mm rod across the requested jaw range. Its physical calibration remains untested.

## Build and verification

```sh
cargo test -p duck-control -p kinematics -p odometry -p robotd-params -p robotd --lib --bins --tests --offline
cargo test -p sounds --lib --offline
cargo build --release -p robotd -p duck-control --example sim_probe --bin robotd --offline
../indieduck-rl/.venv-study/bin/python tools/check_simulation.py --rl ../indieduck-rl --binary target/release/robotd --probe target/release/examples/sim_probe
```

The affected runtime suite passes 234 tests and the sounds suite passes 60. Two manual kinematics timing probes are separately run in release mode. The generated FK oracle tests 64 seeded poses and all 70 sites against native MuJoCo, with position and orientation tolerance 1e-6. Protocol tests exercise wrong identity, joint order and contract hash, reset, disconnect, oversized or malformed replies, timeout, non-finite targets, invalid rail telemetry and latched faults.

Evidence is in `validation/`: build/test logs, native runtime JSON and release timing results. The native harness starts the real daemon, checks at least 100 healthy ticks without unsolicited torque, issues an explicit init, then resets the server and verifies unhealthy status, unavailable IMU and torque off. Its release probe measures 500 native TCP read/write pairs after 50 warmup rounds with torque off. The release run recorded median 0.473 ms, p95 0.816 ms and p99 1.158 ms per read/write pair, with 1.969 ms maximum. The daemon sustained its 50 Hz target across 100 checked ticks with no missed ticks or bus errors. These local Mac measurements use the default XML PD body server. They do not include the separate native m1/D8 BAM diagnostic path, and are not Radxa, serial-bus or trained-behaviour benchmarks. The frozen contract hash is `bca2a9edb18882bee2073046f1e4515350b7affc64ca34d57664fec176bf71eb`.

## Build scope and remaining gaps

The default build excludes bundled score assets. The excluded copyrighted music arrangement has no runtime registry entry. Optional `bundled-scores` requires separately supplied licensed score files; that feature is not verified. The original generated default TOML is a schema/test example, not a deployment configuration.

`robotctl` still needs a licensed baked renderer mesh and configuration fixture. Updater/packaging tests still need deliberate deployment fixtures. The entire workspace therefore remains unfinished even though the simulation daemon and its tested dependencies build.

The inherited physical `DEFAULT_POSITION` also sets neck pitch to approximately 0.3491 rad. The R07 CAD clearance study finds a Trunk collision at that angle. It must not be treated as an approved R07 physical home pose. Physical homing, joint limits and calibration require review against the actual R07 assembly before powered operation. The hardware settings are intentionally preserved in this local study.

The inherited physical battery path still conflates servo supply with pack voltage: Feetech reports the regulated 5 V rail, while legacy shutdown uses a 6.6 V threshold and optional voltage adaptation defaults to 7.4 V. Simulation handles the rail separately. Physical deployment needs a real pack-voltage input and a reviewed power profile; the schema example must not be deployed as-is.

No physical serial port, actuator, camera, NFC reader, speaker or battery was operated. The servo driver, wiring, gains, offsets, current estimates, friction and mass assumptions need hardware measurements. No policy weights, excluded Pollen meshes, MicroDuck simulator code or copyrighted music were restored. No trained walking, get-up, skating or picking capability is claimed by this study.
