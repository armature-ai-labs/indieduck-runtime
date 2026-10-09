# IndieDuck R20 runtime study

This release connects the Rust daemon to the canonical IndieDuck R20 native MuJoCo 3.10.0 server. It is a simulation contributor release. No trained policy or physical robot deployment is included.

## Run

Install Rust 1.89 or newer. Follow the sibling [RL repository](https://github.com/armature-ai-labs/indieduck-rl) native-study setup, then start its server:

```sh
cd ../indieduck-rl
uv sync --frozen
uv run python -m mjlab_microduck.sim.body_server --headless
```

From this repository:

```sh
cargo run --locked -p robotd -- --sim 127.0.0.1:7801 --no-policy --socket /tmp/indieduck-r20.sock
```

The daemon adopts the simulator startup pose with torque off. The local JSON-RPC `robot.init` method explicitly enables simulation actuation and ramps toward the canonical standing pose; `robot.relax` disables it. Gravity-enabled execution does not demonstrate balance or walking. The server accepts one controlling connection.

`--sim`, `--fake` and `--port` are mutually exclusive. `--sim` requires `--no-policy`. The daemon rejects physical startup before reading machine configuration or opening hardware, including the serial `init` subcommand. No override enables hardware in this study. `--fake` remains available for software tests.

Simulation ignores default machine configuration unless `--params` is explicit. Battery shutdown, audio and chorale acceptance are disabled. Host poweroff is disabled for virtual modes. The reported regulated 5 V servo rail is not a 2S battery percentage.

## Contract and faults

The newline JSON TCP protocol remains version 1: hello, read, write, gain, torque and slow telemetry. The handshake requires matching model identity, R20 CAD revision, contract version/hash, MuJoCo version, 15 physical joint names, 14 action names and nominal 5 V rail. Mouth remains physical index 9 and outside policy actions. The existing 61-observation boundary is unchanged; the second IMU is auxiliary.

R20 uses a direct-drive jaw. Normalised mouth opening maps linearly between the canonical contract's closed and open motor angles. This is geometric simulation mapping, not servo calibration. R07's crank and rod equation is removed from the current adapter. An R07 server is rejected.

Malformed data, server errors, timeout, disconnection, clock reversal or reset epoch changes latch a fault, clear cached IMUs and close the socket. There is no serial fallback or automatic reconnect. Commands include session identity and reset epoch. After resetting the server, restart the daemon. Server reset/disconnection disables simulated torque. Replies have a 100 ms timeout and 64 KiB size limit.

Current and temperature telemetry are placeholders. Friction, mass, inertia and actuator parameters require physical measurements. The browser's actuator approximation is distinct from the native BAM diagnostic model.

## Reproduce generated interfaces

The authoritative model is `indieduck-rl/src/mjlab_microduck/robot/indieduck`. Runtime XML, contract, poses, jaw mapping and FK fixtures are generated copies:

```sh
../indieduck-rl/.venv/bin/python tools/sync_indieduck_model.py ../indieduck-rl/src/mjlab_microduck/robot/indieduck
cargo test --locked -p duck-control -p kinematics -p odometry -p robotd-params -p robotd --lib --bins --tests
cargo test --locked -p sounds --lib
cargo build --locked --release -p robotd -p duck-control --example sim_probe --bin robotd
../indieduck-rl/.venv/bin/python tools/check_simulation.py --rl ../indieduck-rl --binary target/release/robotd --probe target/release/examples/sim_probe --output validation/native-runtime-r20.json
```

The sync script requires the R20 direct-drive contract and regenerates 64 seeded native FK cases. It never recalibrates hardware. Commit changes in the canonical model first, then regenerate this repository and cross-reference the source revision in the release.

## Hardware work remaining

Inherited physical code uses servo-supply telemetry in legacy battery logic (6.6 V shutdown and optional 7.4 V voltage adaptation). IndieDuck needs actual pack-voltage sensing separate from the regulated 5 V rail. The inherited physical home pose, offsets, joint limits and gains are not approved for the assembled R20 robot. These gaps are why physical startup is blocked.

A reviewed hardware profile, bench checks and physical clearance measurements must precede enabling a hardware release. The source-only schema example is not deployable configuration. No serial actuator, camera, NFC, speaker or battery has been exercised in this study.

Default Cargo members cover the simulation daemon and checked libraries. The retained `robotctl`, updater/packaging and target services are upstream development material requiring omitted renderer/deployment/model assets. Full `--workspace` tests are not the supported contributor command. Optional `bundled-scores` needs separately licensed files and stays disabled. R07 records are historical evidence, not R20 verification.
