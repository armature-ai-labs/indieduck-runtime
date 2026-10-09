//! IndieDuck protocol 1 TCP adapter. A fault latches until the daemon reconnects.

use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::rc::Rc;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::io::{ImuIo, ImuStale, IoError, JointTargets, Result, RobotIo, Sensors, SlowSensors};
use crate::{ImuData, JOINT_NAMES, NUM_JOINTS};

const MAX_LINE: u64 = 64 * 1024;
const TIMEOUT: Duration = Duration::from_millis(100);
include!("../assets/indieduck/identity.rs");
pub const CONTRACT_SHA256: &str = include_str!("../assets/indieduck/contract.sha256");

#[derive(Deserialize)]
struct Hello {
    protocol: u32,
    mujoco_version: String,
    model_id: String,
    contract_version: u32,
    contract_sha256: String,
    cad_revision: String,
    joint_names: Vec<String>,
    policy_action_joints: Vec<String>,
    session_id: String,
    epoch: u64,
    servo_volts: f64,
}

#[derive(Clone, Copy, Deserialize)]
struct ImuSample {
    gyro: [f64; 3],
    gravity: [f64; 3],
    quat: [f64; 4],
}

impl ImuSample {
    fn checked(self) -> Result<ImuData> {
        if !self
            .gyro
            .iter()
            .chain(&self.gravity)
            .chain(&self.quat)
            .all(|x| x.is_finite())
        {
            return Err(fault("non-finite IMU sample"));
        }
        let norm = self.quat.iter().map(|x| x * x).sum::<f64>();
        if (norm - 1.0).abs() > 0.01 {
            return Err(fault("IMU quaternion is not normalized"));
        }
        Ok(ImuData {
            gyro: self.gyro,
            gravity: self.gravity,
            quat: self.quat,
        })
    }
}

#[derive(Deserialize)]
struct Sample {
    positions: [f64; NUM_JOINTS],
    velocities: [f64; NUM_JOINTS],
    currents_ma: [f64; NUM_JOINTS],
    imu: ImuSample,
    head_imu: ImuSample,
    session_id: String,
    epoch: u64,
    sim_time: f64,
}

struct Connection {
    stream: BufReader<TcpStream>,
    session_id: String,
    epoch: u64,
    imu: Option<ImuData>,
    head_imu: Option<ImuData>,
    last_time: Option<f64>,
    stale: ImuStale,
    error: Option<String>,
}

fn fault(message: impl std::fmt::Display) -> IoError {
    IoError::Bus(format!("IndieDuck simulator: {message}"))
}

impl Connection {
    fn exchange(&mut self, mut request: Value) -> Result<Value> {
        if let Some(error) = &self.error {
            return Err(fault(error));
        }
        if request["op"] != "hello" {
            request["session_id"] = json!(self.session_id);
            request["epoch"] = json!(self.epoch);
        }
        let result = (|| {
            serde_json::to_writer(self.stream.get_mut(), &request).map_err(fault)?;
            self.stream.get_mut().write_all(b"\n").map_err(fault)?;
            let mut line = Vec::with_capacity(2048);
            self.stream
                .by_ref()
                .take(MAX_LINE + 1)
                .read_until(b'\n', &mut line)
                .map_err(fault)?;
            if line.len() as u64 > MAX_LINE || line.last() != Some(&b'\n') {
                return Err(fault("disconnected or oversized/unterminated response"));
            }
            let answer: Value = serde_json::from_slice(&line).map_err(fault)?;
            if !answer.is_object() {
                return Err(fault("response must be a JSON object"));
            }
            if let Some(error) = answer.get("error") {
                return Err(fault(error));
            }
            Ok(answer)
        })();
        if let Err(error) = &result {
            self.latch(error.to_string());
        }
        result
    }

    fn latch(&mut self, message: String) {
        self.error = Some(message);
        self.imu = None;
        self.head_imu = None;
        let _ = self.stream.get_ref().shutdown(Shutdown::Both);
    }

    fn read_sample(&mut self) -> Result<Sensors> {
        let answer = self.exchange(json!({"op": "read"}))?;
        let result = (|| {
            let sample: Sample = serde_json::from_value(answer).map_err(fault)?;
            if sample.session_id != self.session_id || sample.epoch != self.epoch {
                return Err(fault(
                    "simulation reset; restart robotd to adopt the new pose",
                ));
            }
            if !sample
                .positions
                .iter()
                .chain(&sample.velocities)
                .chain(&sample.currents_ma)
                .all(|x| x.is_finite())
                || !sample.sim_time.is_finite()
                || sample.sim_time < 0.0
            {
                return Err(fault("non-finite or invalid sensor data"));
            }
            if self.last_time.is_some_and(|time| sample.sim_time < time) {
                return Err(fault("simulation clock moved backwards"));
            }
            let imu = sample.imu.checked()?;
            let head_imu = sample.head_imu.checked()?;
            if self.last_time == Some(sample.sim_time) {
                self.stale.total += 1;
                self.stale.run += 1;
            } else {
                self.stale.run = 0;
            }
            self.last_time = Some(sample.sim_time);
            self.imu = Some(imu);
            self.head_imu = Some(head_imu);
            Ok(Sensors {
                positions: sample.positions,
                velocities: sample.velocities,
                currents_ma: sample.currents_ma,
                imu,
            })
        })();
        if let Err(error) = &result {
            self.latch(error.to_string());
        }
        result
    }
}

pub struct RemoteIo(Rc<RefCell<Connection>>);
pub struct RemoteImu(Rc<RefCell<Connection>>);

impl RemoteIo {
    /// Connect once. There is no fallback to a real serial port or automatic reconnect.
    pub fn connect(address: &str) -> Result<(Self, RemoteImu)> {
        let addresses = address.to_socket_addrs().map_err(fault)?;
        let mut stream = None;
        let mut last_error = None;
        for address in addresses {
            match TcpStream::connect_timeout(&address, Duration::from_secs(2)) {
                Ok(value) => {
                    stream = Some(value);
                    break;
                }
                Err(error) => last_error = Some(error),
            }
        }
        let stream =
            stream.ok_or_else(|| fault(format!("cannot connect to {address}: {last_error:?}")))?;
        stream.set_nodelay(true).map_err(fault)?;
        stream.set_read_timeout(Some(TIMEOUT)).map_err(fault)?;
        stream.set_write_timeout(Some(TIMEOUT)).map_err(fault)?;
        let action_joints: Vec<&str> = JOINT_NAMES
            .iter()
            .copied()
            .filter(|name| *name != "mouth")
            .collect();
        let mut connection = Connection {
            stream: BufReader::new(stream),
            session_id: String::new(),
            epoch: 0,
            imu: None,
            head_imu: None,
            last_time: None,
            stale: ImuStale::default(),
            error: None,
        };
        let answer = connection.exchange(json!({
            "op": "hello", "protocol": 1, "model_id": MODEL_ID,
            "contract_version": CONTRACT_VERSION, "contract_sha256": CONTRACT_SHA256, "cad_revision": CAD_REVISION,
            "joint_names": JOINT_NAMES, "policy_action_joints": action_joints,
        }))?;
        let hello: Hello = serde_json::from_value(answer).map_err(fault)?;
        if hello.protocol != 1
            || hello.mujoco_version != "3.10.0"
            || hello.model_id != MODEL_ID
            || hello.contract_version != CONTRACT_VERSION
            || hello.contract_sha256 != CONTRACT_SHA256
            || hello.cad_revision != CAD_REVISION
            || hello.joint_names != JOINT_NAMES
            || hello.policy_action_joints != action_joints
            || hello.session_id.is_empty()
            || !hello.servo_volts.is_finite()
            || (hello.servo_volts - 5.0).abs() > 1e-9
        {
            return Err(fault(
                "protocol, model, revision or joint contract mismatch",
            ));
        }
        connection.session_id = hello.session_id;
        connection.epoch = hello.epoch;
        let connection = Rc::new(RefCell::new(connection));
        Ok((Self(Rc::clone(&connection)), RemoteImu(connection)))
    }

    /// Auxiliary sensor; never inserted into the existing 61-element locomotion observation.
    pub fn head_imu(&self) -> Option<ImuData> {
        self.0.borrow().head_imu
    }
}

impl RobotIo for RemoteIo {
    fn read(&mut self) -> Result<Sensors> {
        self.0.borrow_mut().read_sample()
    }
    fn write(&mut self, targets: &JointTargets) -> Result<()> {
        if !targets.positions.iter().all(|value| value.is_finite()) {
            return Err(fault("non-finite target"));
        }
        self.0
            .borrow_mut()
            .exchange(json!({"op": "write", "targets": targets.positions}))
            .map(|_| ())
    }
    fn set_gain(&mut self, kp: u16) -> Result<()> {
        self.0
            .borrow_mut()
            .exchange(json!({"op": "gain", "kp": kp}))
            .map(|_| ())
    }
    fn set_torque(&mut self, on: bool) -> Result<()> {
        self.0
            .borrow_mut()
            .exchange(json!({"op": "torque", "on": on}))
            .map(|_| ())
    }
    fn slow_sensors(&mut self) -> Result<SlowSensors> {
        #[derive(Deserialize)]
        struct Slow {
            volts: f64,
            temps_c: [f64; NUM_JOINTS],
        }
        let answer = self.0.borrow_mut().exchange(json!({"op": "slow"}))?;
        let slow: Slow = match serde_json::from_value(answer) {
            Ok(slow) => slow,
            Err(error) => {
                self.0.borrow_mut().latch(error.to_string());
                return Err(fault(error));
            }
        };
        if !slow.volts.is_finite()
            || !(4.75..=5.25).contains(&slow.volts)
            || !slow.temps_c.iter().all(|value| value.is_finite())
        {
            self.0
                .borrow_mut()
                .latch("invalid 5 V servo rail or temperature data".into());
            return Err(fault("invalid 5 V servo rail or temperature data"));
        }
        Ok(SlowSensors {
            volts: slow.volts,
            temps_c: slow.temps_c,
        })
    }
}

impl ImuIo for RemoteImu {
    fn read(&mut self) -> Result<ImuData> {
        self.0
            .borrow()
            .imu
            .ok_or_else(|| fault("no current IMU sample"))
    }
    fn ready(&self) -> bool {
        self.0.borrow().imu.is_some()
    }
    fn stale(&self) -> ImuStale {
        self.0.borrow().stale
    }
}

#[cfg(test)]
mod tests;
