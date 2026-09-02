//! Read a Dashlane vault through Dashlane's `dcli` command-line tool.
//!
//! Secret values stay in-process, vault access is read-only, and nothing is
//! written to disk.
//!
//! stevedore **never authenticates**. Registering a device, entering the Master
//! Password and passing 2FA are a one-time setup performed with `dcli` directly
//! (see `docs/dcli/`). Every call here needs an authenticated, unlocked `dcli`
//! and fails with a clear error otherwise.
//!
//! # What can be read
//!
//! Logins ([`Login`]), secure notes ([`Note`]) and secrets ([`Secret`]) —
//! nothing else. Passkeys, payments, IDs and personal info cannot be reached.
//! Secrets are a Business feature, so a Personal vault yields none. Grouping is
//! unavailable: a login carries no category, and a note's [`Note::category`]
//! does not reflect Dashlane's Collections.
//!
//! Note attachments can be listed but not fetched — see [`Attachment`].
//!
//! Reads are served from `dcli`'s local copy of the vault, which it refreshes
//! from Dashlane when that copy is over an hour old — so a read can reach the
//! network without [`sync`] being called.

pub(crate) mod client;
mod login;
mod note;
mod secret;

pub use client::{Status, logins, notes, secrets, status, sync};
pub use login::Login;
pub use note::{Attachment, NO_CATEGORY, Note};
pub use secret::Secret;

/// Store name, used in errors and the CLI's `stores` listing.
pub const NAME: &str = "dashlane";
