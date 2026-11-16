# Agent contract

The estate-wide conventions live in one place and are **not duplicated here**:

**https://github.com/JorisJonkers-dev/workspace/blob/main/CLAUDE.md**

Read it before doing anything non-trivial in this repository. It covers pull request labels
(run `gh label list --repo JorisJonkers-dev/rauls-trusty-toolbox` before passing `--label`),
verifying values rather than commands, the testing standard, and traps around workflow runs and
detached submodule HEADs.

---

## This repository

- The plan is [`docs/PLAN.md`](docs/PLAN.md). Read it before changing structure.
- Components under `components/` are git submodules. A change to a component is a pull request
  in that component's repository, then a pointer bump here. Never commit on a detached
  submodule HEAD.
- No game data, mod archives or Windows binaries enter this repository. Fixtures are synthetic.
- Terms follow `GLOSSARY.md`; decisions are in `docs/adr`. New source files carry
  `SPDX-License-Identifier: GPL-3.0-only`.
