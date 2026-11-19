// SPDX-License-Identifier: GPL-3.0-only
//! Installing a newer raul over the running one (ADR-0020): the portable core, tested on synthetic folders.
//! Nothing here touches the network or starts a program; the verified, staged files come from the caller.

pub mod install;
pub mod selfcheck;
pub mod swap;
