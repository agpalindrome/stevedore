//! Write secrets into a Proton Pass vault through Proton's `pass-cli`.
//!
//! Secret values reach `pass-cli` on **stdin**, never as command-line
//! arguments: a process's arguments are readable by any other process on the
//! machine while it runs.
//!
//! stevedore **never authenticates**. Logging `pass-cli` in is a one-time setup
//! performed with `pass-cli` directly (see `docs/pass-cli/`). Every call here
//! needs a working session and fails with a clear error otherwise.
//!
//! # What can be written
//!
//! Logins ([`NewLogin`]) and secure notes ([`NewNote`]), each **created** in a
//! named [`Vault`]. Existing items are never updated: `pass-cli item update`
//! takes field values as command-line arguments, which is not a safe channel
//! for a secret.
//!
//! A login carries a title, username, email, password, TOTP URI and URLs —
//! Proton's item template offers no field for a note attached to a login, so
//! that text has nowhere to go.
//!
//! # What can be read
//!
//! [`items`] describes what a vault already holds — each [`Item`]'s title,
//! [`Kind`] and [`State`]. Descriptions only: `pass-cli` yields item contents
//! just for `--show-secrets`, which stevedore never passes. Proton Pass is a
//! sink, so no secret is read out of it.

pub(crate) mod client;
mod item;
mod listing;
mod vault;

pub use client::{Session, create_login, create_note, items, session, vault, vaults};
pub use item::{NewLogin, NewNote};
pub use listing::{Item, Kind, State};
pub use vault::Vault;

/// Store name, used in errors and the CLI's `stores` listing.
pub const NAME: &str = "proton-pass";
