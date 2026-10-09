#!/usr/bin/env python3
"""Exercise the real robotd executable against the native MuJoCo body server."""
import argparse
from datetime import datetime, timezone
import hashlib
import platform
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import threading
import time

parser = argparse.ArgumentParser()
parser.add_argument('--rl', type=Path, required=True)
parser.add_argument('--binary', type=Path, default=Path('target/debug/robotd'))
parser.add_argument('--probe', type=Path)
parser.add_argument('--output', type=Path, default=Path('validation/native-runtime.json'))
args = parser.parse_args()
sys.path.insert(0, str(args.rl.resolve() / 'src'))
from mjlab_microduck.sim import body_server as sim

for arguments in [[], ['--port', '/dev/indieduck-must-not-open'], ['init']]:
    result = subprocess.run([str(args.binary.resolve()), *arguments], capture_output=True, text=True, timeout=3)
    assert result.returncode != 0 and 'Physical profiles remain blocked' in result.stderr, result

world = sim.World(sim.DEFAULT_SCENE)
body = sim.Body(world, 0)
world.bodies.append(body)
body.reset('stand')
server = sim.Server(('127.0.0.1', 0), sim.Handler)
server.body = body
threading.Thread(target=server.serve_forever, daemon=True).start()
stop = threading.Event()
def step():
    while not stop.is_set():
        start = time.perf_counter()
        world.step(4)
        stop.wait(max(0, .02 - (time.perf_counter() - start)))
thread = threading.Thread(target=step, daemon=True)
thread.start()

def rpc(path, method):
    with socket.socket(socket.AF_UNIX) as client:
        client.settimeout(1)
        client.connect(str(path))
        client.sendall((json.dumps({'jsonrpc':'2.0','id':1,'method':method,'params':{}})+'\n').encode())
        reply = json.loads(client.makefile('rb').readline())
        assert 'error' not in reply or reply['error'] is None, reply
        return reply['result']

def wait_for(path, predicate, seconds=5):
    deadline = time.monotonic() + seconds
    answer = None
    while time.monotonic() < deadline:
        try:
            answer = rpc(path, 'robot.health')
            if predicate(answer): return answer
        except (OSError, ValueError): pass
        time.sleep(.025)
    raise AssertionError(f'health condition not reached: {answer}')

report = {'model_id':sim.CONTRACT['model_id'], 'contract_sha256':sim.CONTRACT_SHA256,
          'mujoco_version':sim.mujoco.__version__, 'physical_hardware':False,
          'recorded_utc':datetime.now(timezone.utc).isoformat(), 'host':platform.platform(),
          'runtime_binary':str(args.binary.resolve()),
          'runtime_binary_sha256':hashlib.sha256(args.binary.read_bytes()).hexdigest(),
          'physical_startup_rejected_before_io': True}
if args.probe:
    measured = subprocess.check_output([str(args.probe.resolve()), f'127.0.0.1:{server.server_address[1]}'], text=True)
    report['native_protocol_latency'] = json.loads(measured)

with tempfile.TemporaryDirectory(prefix='indieduck-r20-') as directory:
    path = Path(directory) / 'robotd.sock'
    log_path = Path(directory) / 'daemon.log'
    with log_path.open('w') as log:
        child = subprocess.Popen([str(args.binary.resolve()), '--sim', f'127.0.0.1:{server.server_address[1]}', '--no-policy', '--socket', str(path)], stdout=log, stderr=log, env={**os.environ, 'RUST_LOG':'info'})
        try:
            healthy = wait_for(path, lambda h: h['healthy'] and h['control_loop']['ticks'] >= 100)
            assert healthy.get('battery') is None, healthy
            assert not body.torque_on, 'daemon armed torque without init'
            report['steady_health'] = healthy
            report['init_reply'] = rpc(path, 'robot.init')
            deadline = time.monotonic() + 2
            while not body.torque_on and time.monotonic() < deadline: time.sleep(.01)
            assert body.torque_on, 'explicit init did not reach native actuator I/O'
            report['explicit_init_reached_native_torque'] = True
            body.reset('stand')
            fault = wait_for(path, lambda h: not h['healthy'] and h['bus']['consecutive_errors'] >= 10 and not h['imu']['ready'])
            assert not body.torque_on, 'reset must disable torque'
            report['reset_health'] = fault
            report['reset_requires_restart'] = True
        finally:
            child.terminate()
            child.wait(timeout=3)
            stop.set()
            thread.join(timeout=2)
            server.shutdown()
            server.server_close()
    report['daemon_log'] = log_path.read_text()
args.output.parent.mkdir(parents=True, exist_ok=True)
args.output.write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({k:v for k,v in report.items() if k != 'daemon_log'}, indent=2))
