use duck_control::sim::RemoteIo;
use duck_control::{JointTargets, RobotIo};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::args()
        .nth(1)
        .ok_or("usage: sim_probe HOST:PORT")?;
    let (mut io, _) = RemoteIo::connect(&address)?;
    io.set_torque(false)?;
    io.set_gain(32)?;
    let mut elapsed = Vec::with_capacity(500);
    for index in 0..550 {
        let started = Instant::now();
        let sample = io.read()?;
        io.write(&JointTargets::new(sample.positions))?;
        if index >= 50 {
            elapsed.push(started.elapsed().as_secs_f64() * 1000.0);
        }
    }
    elapsed.sort_by(f64::total_cmp);
    let slow = io.slow_sensors()?;
    println!(
        "{}",
        serde_json::json!({"samples":500,"warmup":50,"read_write_ms":{
        "median":elapsed[250],"p95":elapsed[475],"p99":elapsed[495],"max":elapsed[499]},
        "servo_volts":slow.volts,"auxiliary_head_imu":io.head_imu().is_some(),"torque_enabled":false})
    );
    Ok(())
}
