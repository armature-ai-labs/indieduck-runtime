# Upstream attribution and snapshot provenance

This repository contains a filtered source snapshot from [JoyandAI/microduck](https://github.com/JoyandAI/microduck/tree/b6b658a6e2bb888302236f8a5b35844b53685409), branch `main`, commit `b6b658a6e2bb888302236f8a5b35844b53685409`. It is an integration base for IndieDuck. It is not a complete robot release or a claim that upstream code runs on IndieDuck hardware.

JoyandAI's repository is a GitHub fork of [pollen-robotics/microduck](https://github.com/pollen-robotics/microduck). Credit belongs to Pollen Robotics, JoyandAI and the upstream contributors. The source authors found in the history of the included paths are retained in [provenance/UPSTREAM_AUTHORS.txt](provenance/UPSTREAM_AUTHORS.txt). This list is supplemental; source copyright notices and third-party credits remain authoritative.

This is a clean code import. Original Git history is not included and this repository must not be described as a GitHub-native fork. Every imported source file is byte-identical to the selected upstream Git blob. IndieDuck has added attribution, selection manifests and validation notes. No robot-model adaptation or interface integration has been performed in this snapshot.

## Verified ancestry

The selected JoyandAI commit and the Pollen comparison commit `c2b0a213abee69e038414fd3cc16b7c685ab4c0d` share common ancestor `2c61dcc1f03440541cdc0729f7a375b2a9ea3005`. The selected JoyandAI commit is 12 commits ahead of that common ancestor; the Pollen comparison commit is 559 commits ahead. The Pollen comparison commit is **not** an ancestor of the selected JoyandAI commit. These pins describe divergent histories, not a containment or compatibility guarantee.

## Included and excluded source

- [Included source manifest](provenance/included-upstream-files.csv): 183 imported paths, upstream Git blob IDs and SHA-256 hashes.
- [Excluded source manifest](provenance/excluded-upstream-files.csv): 135 omitted upstream paths and reasons.
- [Machine-readable source record](provenance/upstream.json).
- [Static validation record](provenance/validation.json).
- [License boundaries](THIRD_PARTY_NOTICES.md).
- [Current build and execution limitations](VALIDATION.md).

Neither selected upstream tree contains Git submodule entries or a `.gitmodules` file. Third-party code is vendored or obtained through package dependencies. Lockfiles retain the upstream dependency choices; those choices have not been tested on IndieDuck.

Upstream developer paths in test fixtures and examples remain unchanged for source fidelity. They are public upstream examples, not paths from an IndieDuck maintainer's machine. Some manual scripts retain upstream-specific defaults that need review before use. No GitHub Actions workflows or deployment keys are included.
