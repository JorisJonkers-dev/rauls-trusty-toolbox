---
status: accepted
date: 2026-11-16
---
# The Rolling engine channel

Amended by [ADR-0018](0018-the-submodule-pins-are-the-set.md): there is no chain of Known-good sets or `known-good.toml` any more. "The newest Known-good set" and "the current set" below mean the Component versions the submodule pins and the lock files fix; the Rolling channel itself stands.

nv-rs builds may be offered ahead of the newest Known-good set, because the fork publishes several builds a day and
the owner runs the newest. Before a newer build is offered, the Toolbox checks compatibility: the `capabilities.json`
difference, that the Load order is still covered, and that every Contract version the build emits is one the current
set's consumers accept. A build is installed side by side and becomes active by a pointer; rollback switches the
pointer back in one action. Nothing updates without the person asking, and update checks carry no identifier.

This moves the decision recorded in nvmod-rs `docs/DECISIONS.md` (2026-11-16) into the Toolbox, which now implements
it; nvmod-rs keeps a pointer here. The chain-of-sets rule stays for every other Component.
