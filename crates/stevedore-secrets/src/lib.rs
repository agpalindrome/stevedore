//! `stevedore` — move secrets between password managers and vaults.
//!
//! - [`SecretValue`] — a secret that redacts itself in `Debug` and `Display`.
//!   Read it deliberately with [`SecretValue::expose`].
//! - [`dashlane`] — read a Dashlane vault through Dashlane's own `dcli`.
//! - [`proton`] — write into a Proton Pass vault through Proton's own
//!   `pass-cli`.
//! - [`mover`] — plan a move from the one into the other, then carry it out.
//!
//! Dashlane is read, Proton Pass is written; neither does the other. A Proton
//! vault can be listed — item titles, kinds and states, never values — so a
//! write can tell what is already there.
//!
//! # Moving a vault
//!
//! Planning writes nothing, so it is safe against any vault. Both stores must
//! already be logged in — stevedore never authenticates.
//!
//! ```no_run
//! use stevedore_secrets::{mover, proton};
//!
//! let vault = proton::vault("Personal")?;
//!
//! let plan = mover::plan(&vault)?;
//! let (logins, notes) = (plan.logins(), plan.notes());
//!
//! let report = mover::apply(&vault, plan)?;
//! assert_eq!(report.created, logins + notes - report.failures.len());
//! # Ok::<(), stevedore_secrets::Error>(())
//! ```

mod cli;
mod dataroot;

pub mod dashlane;
pub mod error;
pub mod mover;
pub mod proton;
pub mod secret;

pub use error::{CliError, Error, Result};
pub use secret::SecretValue;
