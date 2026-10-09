#!/usr/bin/env python3
"""Copy generated model interfaces and regenerate the native MuJoCo FK oracle."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

import mujoco
import numpy as np

parser = argparse.ArgumentParser()
parser.add_argument('model', type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
model_dir = args.model.resolve()
if mujoco.__version__ != '3.10.0':
    raise SystemExit('FK fixtures require MuJoCo 3.10.0')
contract = json.loads((model_dir / 'contract.json').read_text())
linkage = contract['mouth_linkage']
if contract['cad_revision'] != 'R20' or linkage['type'] != 'direct_drive':
    raise SystemExit('Expected canonical R20 direct-drive model; no legacy crank fallback')
if len(contract['physical_joint_names']) != 15 or len(contract['policy_action_joints']) != 14:
    raise SystemExit('Expected 15 physical joints and 14 policy actions')
if contract['physical_joint_names'][9] != 'mouth' or 'mouth' in contract['policy_action_joints']:
    raise SystemExit('Mouth must remain separate physical joint 9')
for source, destination in [('kinematics.xml', 'kinematics/assets/indieduck/kinematics.xml'), ('contract.json', 'duck-control/assets/indieduck/contract.json'), ('poses.json', 'duck-control/assets/indieduck/poses.json')]:
    out = root / destination
    out.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(model_dir / source, out)
generated = '// Generated geometric mapping from canonical IndieDuck R20 contract. Apache-2.0.\n'
generated += f"pub const INDIEDUCK_MOUTH_OPEN: f64 = {float(linkage['motor_open_rad'])};\n"
generated += f"pub const INDIEDUCK_MOUTH_CLOSED: f64 = {float(linkage['motor_closed_rad'])};\n"
identity = f"pub const MODEL_ID: &str = {json.dumps(contract['model_id'])};\n"
identity += f"pub const CAD_REVISION: &str = {json.dumps(contract['cad_revision'])};\n"
identity += f"pub const CONTRACT_VERSION: u32 = {int(contract['contract_version'])};\n"
(root / 'duck-control/assets/indieduck/identity.rs').write_text(identity)
poses = json.loads((model_dir / 'poses.json').read_text())
stand = next(p for p in poses['poses'] if p['id'] == 'stand')['joint_positions_rad']
home = [float(stand[name]) for name in contract['physical_joint_names']]
generated += f'pub const INDIEDUCK_HOME_POSITION: [f64; 15] = {home!r};\n'
(root / 'duck-control/assets/indieduck/linkage.rs').write_text(generated)
contract_hash = hashlib.sha256((model_dir / 'contract.json').read_bytes()).hexdigest()
(root / 'duck-control/assets/indieduck/contract.sha256').write_text(contract_hash)
model = mujoco.MjModel.from_xml_path(str(model_dir / 'scene.xml'))
data = mujoco.MjData(model)
trunk = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_BODY, 'trunk_base')
rng = np.random.default_rng(7)
samples = []
for _ in range(64):
    mujoco.mj_resetData(model, data)
    joints = {}
    for j in range(model.njnt):
        if model.jnt_type[j] != mujoco.mjtJoint.mjJNT_HINGE:
            continue
        name = mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_JOINT, j)
        value = float(rng.uniform(*model.jnt_range[j]))
        data.qpos[model.jnt_qposadr[j]] = value
        joints[name] = value
    mujoco.mj_kinematics(model, data)
    rotation = data.xmat[trunk].reshape(3,3)
    sites = {}
    for i in range(model.nsite):
        name = mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_SITE, i)
        pos = rotation.T @ (data.site_xpos[i] - data.xpos[trunk])
        local_rotation = rotation.T @ data.site_xmat[i].reshape(3,3)
        quat = np.empty(4)
        mujoco.mju_mat2Quat(quat, local_rotation.ravel())
        sites[name] = {'pos':pos.tolist(),'quat':quat.tolist()}
    samples.append({'joints':joints,'sites':sites})
fixture = {'model_id':contract['model_id'],'mujoco_version':mujoco.__version__, 'contract_sha256':contract_hash, 'seed':7, 'samples':samples}
(root / 'kinematics/tests/fixtures/fk_indieduck.json').write_text(json.dumps(fixture, indent=2)+'\n')
print(f'Synced {contract_hash}; {len(samples)} poses, {model.nsite} sites each')
