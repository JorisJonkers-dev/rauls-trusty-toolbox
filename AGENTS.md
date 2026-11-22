# Agents

Guidance for any coding agent working in this repository.

- Agent contract: [CLAUDE.md](CLAUDE.md)
- Estate conventions: https://github.com/JorisJonkers-dev/workspace/blob/main/CLAUDE.md
- Plan: [docs/PLAN.md](docs/PLAN.md)
- Pictures, soaks, sweeps and benches of the game: [docs/SCENES.md](docs/SCENES.md) (`raul scene`, `raul bench game`; no scripts that start the game)

## Build and test

- Apply the branch rulesets: `scripts/apply-ruleset.sh`
- Rust 1.100.0 (`rust-toolchain.toml`) with a 7-day registry cooldown (`.cargo/config.toml`).
- Rust build and test commands arrive with the umbrella workspace (Phase 1 of the plan).
