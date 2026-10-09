use super::*;
use std::net::TcpListener;
use std::thread;
use std::time::Instant;

fn hello() -> Value {
    json!({"protocol":1,"mujoco_version":"3.10.0","model_id":MODEL_ID,"contract_version":CONTRACT_VERSION,"contract_sha256":CONTRACT_SHA256,"cad_revision":CAD_REVISION,
        "joint_names":JOINT_NAMES,"policy_action_joints":JOINT_NAMES.iter().filter(|name| **name != "mouth").collect::<Vec<_>>(),
        "servo_volts":5.0,"session_id":"test-session","epoch":0})
}

fn sample() -> Value {
    let imu = json!({"gyro":[0.0,0.0,0.0],"gravity":[0.0,0.0,-1.0],"quat":[1.0,0.0,0.0,0.0]});
    json!({"positions":vec![0.0;15],"velocities":vec![0.0;15],"currents_ma":vec![0.0;15],
        "imu":imu,"head_imu":imu,"session_id":"test-session","epoch":0,"sim_time":1.0})
}

fn server(
    mut answer: impl FnMut(Value) -> Option<Value> + Send + 'static,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let join = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        socket.set_nodelay(true).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut socket = BufReader::new(socket);
        loop {
            let mut line = String::new();
            if socket.read_line(&mut line).unwrap_or(0) == 0 {
                break;
            }
            let Some(response) = answer(serde_json::from_str(&line).unwrap()) else {
                break;
            };
            if serde_json::to_writer(socket.get_mut(), &response).is_err() {
                break;
            }
            if socket.get_mut().write_all(b"\n").is_err() {
                break;
            }
        }
    });
    (address, join)
}

#[test]
fn handshake_and_every_control_operation_round_trip() {
    let (address, join) = server(|request| {
        if request["op"] != "hello" {
            assert_eq!(request["session_id"], "test-session");
            assert_eq!(request["epoch"], 0);
        }
        Some(match request["op"].as_str().unwrap() {
            "hello" => {
                assert_eq!(request["joint_names"][9], "mouth");
                hello()
            }
            "read" => sample(),
            "write" => {
                assert_eq!(request["targets"].as_array().unwrap().len(), 15);
                json!({})
            }
            "gain" => {
                assert_eq!(request["kp"], 32);
                json!({})
            }
            "torque" => {
                assert_eq!(request["on"], true);
                json!({})
            }
            "slow" => json!({"volts":5.0,"temps_c":vec![32.0;15]}),
            other => panic!("unexpected {other}"),
        })
    });
    let (mut io, mut imu) = RemoteIo::connect(&address).unwrap();
    assert!(!imu.ready());
    let sensed = io.read().unwrap();
    assert_eq!(imu.read().unwrap(), sensed.imu);
    assert!(io.head_imu().is_some());
    io.write(&JointTargets::new([0.0; 15])).unwrap();
    io.set_gain(32).unwrap();
    io.set_torque(true).unwrap();
    assert_eq!(io.slow_sensors().unwrap().volts, 5.0);
    drop(io);
    drop(imu);
    join.join().unwrap();
}

#[test]
fn rejects_wrong_model_contract_and_order() {
    for field in [
        "model_id",
        "contract_version",
        "joint_names",
        "policy_action_joints",
        "protocol",
        "cad_revision",
        "contract_sha256",
        "mujoco_version",
    ] {
        let (address, join) = server(move |_| {
            let mut response = hello();
            response[field] = match field {
                "contract_version" | "protocol" => json!(99),
                "joint_names" | "policy_action_joints" => json!(["wrong"]),
                _ => json!("wrong"),
            };
            Some(response)
        });
        assert!(RemoteIo::connect(&address).is_err(), "{field}");
        join.join().unwrap();
    }
}

#[test]
fn reset_disconnection_and_bad_sample_latch_until_restart() {
    for failure in [
        "reset",
        "disconnect",
        "short",
        "clock",
        "server_error",
        "bad_quaternion",
    ] {
        let mut reads = 0;
        let (address, join) = server(move |request| {
            if request["op"] == "hello" {
                return Some(hello());
            }
            reads += 1;
            if reads == 1 {
                return Some(sample());
            }
            let mut response = sample();
            match failure {
                "reset" => response["epoch"] = json!(1),
                "disconnect" => return None,
                "short" => response["positions"] = json!([0.0]),
                "clock" => response["sim_time"] = json!(0.0),
                "server_error" => return Some(json!({"error":"solver fault"})),
                "bad_quaternion" => response["imu"]["quat"] = json!([0.0, 0.0, 0.0, 0.0]),
                _ => unreachable!(),
            }
            Some(response)
        });
        let (mut io, mut imu) = RemoteIo::connect(&address).unwrap();
        io.read().unwrap();
        assert!(io.read().is_err(), "{failure}");
        assert!(io.write(&JointTargets::new([0.0; 15])).is_err());
        assert!(!imu.ready());
        assert!(imu.read().is_err());
        drop(io);
        drop(imu);
        join.join().unwrap();
    }
}

#[test]
fn timeout_is_bounded_and_nonfinite_target_is_rejected() {
    let (address, join) = server(|request| {
        if request["op"] == "hello" {
            Some(hello())
        } else {
            thread::sleep(Duration::from_millis(250));
            None
        }
    });
    let (mut io, imu) = RemoteIo::connect(&address).unwrap();
    assert!(io.write(&JointTargets::new([f64::NAN; 15])).is_err());
    let start = Instant::now();
    assert!(io.read().is_err());
    assert!(start.elapsed() < Duration::from_secs(1));
    drop(io);
    drop(imu);
    join.join().unwrap();
}

#[test]
fn localhost_protocol_latency_report() {
    let (address, join) = server(|request| {
        Some(if request["op"] == "hello" {
            hello()
        } else if request["op"] == "read" {
            sample()
        } else {
            json!({})
        })
    });
    let (mut io, imu) = RemoteIo::connect(&address).unwrap();
    let mut samples = Vec::new();
    for tick in 0..550 {
        let start = Instant::now();
        io.read().unwrap();
        io.write(&JointTargets::new([0.0; 15])).unwrap();
        if tick >= 50 {
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "localhost 15-joint read+write, 50 warmup/500 samples: median={:.3} ms p95={:.3} ms p99={:.3} ms max={:.3} ms",
        samples[250], samples[475], samples[495], samples[499]
    );
    drop(io);
    drop(imu);
    join.join().unwrap();
}

#[test]
fn oversized_response_and_invalid_servo_rail_latch() {
    for failure in ["oversize", "rail", "short_slow"] {
        let (address, join) = server(move |request| {
            Some(if request["op"] == "hello" {
                hello()
            } else if failure == "oversize" {
                json!({"padding":"x".repeat(70_000)})
            } else if failure == "short_slow" {
                json!({"volts":5.0})
            } else {
                json!({"volts":7.4,"temps_c":vec![32.0;15]})
            })
        });
        let (mut io, imu) = RemoteIo::connect(&address).unwrap();
        assert!(io.slow_sensors().is_err());
        assert!(io.set_torque(true).is_err());
        drop(io);
        drop(imu);
        join.join().unwrap();
    }
}

#[test]
fn r07_server_is_rejected_before_any_command() {
    let (address, join) = server(|request| {
        assert_eq!(request["op"], "hello");
        let mut response = hello();
        response["model_id"] = json!("indieduck-r07");
        response["cad_revision"] = json!("R07");
        Some(response)
    });
    assert!(RemoteIo::connect(&address).is_err());
    join.join().unwrap();
}
