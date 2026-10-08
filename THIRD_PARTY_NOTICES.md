# Third-party notices

The imported runtime software is distributed under the upstream Apache License 2.0 in [LICENSE](LICENSE), subject to the separately licensed components below. Upstream copyright and attribution comments are retained. Adding an IndieDuck repository name does not change their ownership or licenses.

| Component | Source and license boundary |
| --- | --- |
| Pollen Robotics and JoyandAI runtime | [Pinned source](https://github.com/JoyandAI/microduck/tree/b6b658a6e2bb888302236f8a5b35844b53685409). Apache-2.0 root license. Contributor names are in `provenance/UPSTREAM_AUTHORS.txt`. |
| `vendor/bisync` 0.3.1 | Manifest declares `MIT OR Apache-2.0`. Source origin is [JM4ier/bisync](https://github.com/JM4ier/bisync/tree/be3a0a6255a0af592e66be8614a149151f67fdf5), identified by the retained `.cargo_vcs_info.json`. The upstream snapshot omitted standalone license texts; this import adds the exact `LICENSE-APACHE` and `LICENSE-MIT` from that pinned origin. See `provenance/supplementary-license-sources.json`. |
| `tof/vendor` STMicroelectronics drivers | Original headers and [LICENSE.txt](tof/vendor/LICENSE.txt) are retained. VL53L5CX headers offer a BSD-3-Clause option alongside the proprietary option. The component license file supplies BSD-3-Clause terms when received without other package terms. These files are not relicensed Apache-2.0. |
| External Cargo, platform and inference dependencies | `Cargo.lock` and manifests retain upstream versions. Dependencies are not vendored by this snapshot except the directories above. Their separate licenses still apply when installed or distributed. |

## Excluded material

The filtered import excludes trained ONNX/RKNN weights, robot MJCF and the baked `duck.bin` mesh, all musical score assets, deployment keys and configuration, installation scripts and GitHub workflows. These exclusions do not determine a license for the omitted files or authorize later redistribution.

The upstream `sounds/src/chorale/mod.rs` identifies its Outer Wilds arrangement as copyrighted and instructs removal before shipping. All score files remain excluded, including the two that upstream describes as original. Source references are retained for future adaptation; this leaves compile blockers documented in `VALIDATION.md`.

Upstream deployment audio driver `deploy/audio/aic3x-dkms/tlv320aic3x.c` declares `GPL-2.0-only`; it is excluded. Upstream `source-lessons/assets/marked.min.js` declares Marked 15.0.7 under MIT; it is also excluded. These files must not be swept into an Apache-only licensing claim during a future import.

The upstream robot geometry derives from the Microduck simulation model. That model's repository distinguishes 3D model files from Apache software and states a Creative Commons BY-SA-NC license without identifying a version. The runtime XML and baked mesh are therefore omitted pending a per-path rights review or replacement with independently authored, appropriately licensed geometry.
