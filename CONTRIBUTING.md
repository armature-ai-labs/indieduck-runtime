# Contributing

Keep changes focused and describe the problem, affected interface and evidence. Start with the open integration work in the README. Preserve upstream module names unless the change includes a migration and compatibility checks.

- Retain upstream copyright and licence notices. Record new sources, revisions and terms in `provenance/`; do not restore excluded models, checkpoints, media or secrets without independently established redistribution rights.
- Identify the hardware CAD, runtime, RL and policy revisions used in any compatibility report. State whether results are syntax checks, compilation, unit tests, simulation or physical experiments.
- For a behaviour change, run the relevant tests and report exact commands and outcomes. When an excluded dependency prevents testing, name it and describe the untested scope rather than claiming success.
- Keep credentials, robot network details, private datasets, generated build output and local deployment configuration out of commits.
- Prefer small pull requests. Include only related changes and explain any upstream-source edits in the provenance change record.

The initial snapshot has known integration blockers. Documentation and source-review contributions are useful before a complete runtime or simulator exists.
