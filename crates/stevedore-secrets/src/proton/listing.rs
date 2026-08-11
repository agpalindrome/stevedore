//! What a vault already holds, as `pass-cli item list --output json` reports
//! it.

use serde::Deserialize;

/// One item already in a vault.
///
/// This is the item's description, not its contents: no field here holds a
/// secret value. stevedore lists a vault to see what is already there, so it
/// never asks `pass-cli` for the contents.
#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    /// Identifies the item.
    pub id: String,

    /// Identifies the caller's share of the vault holding it.
    pub share_id: String,

    /// Identifies the vault holding it.
    pub vault_id: String,

    /// The item's name, as shown in Proton Pass.
    pub title: String,

    /// Which kind of item it is.
    pub item_type: Kind,

    /// Whether the item is live or in the trash.
    pub state: State,

    /// Markers `pass-cli` attaches to the item.
    #[serde(default)]
    pub flags: Vec<String>,

    /// When the item was created, as `pass-cli` reports it.
    pub create_time: Option<String>,

    /// When it last changed.
    pub modify_time: Option<String>,
}

impl Item {
    /// Whether the item is live, rather than trashed.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state == State::Active
    }
}

/// The kind of item, as `pass-cli` names it.
///
/// stevedore writes only [`Login`] and [`Note`]; the rest are kinds Proton Pass
/// supports that may already be in a vault.
///
/// [`Login`]: Kind::Login
/// [`Note`]: Kind::Note
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// A username and password.
    Login,
    /// A secure note.
    Note,
    /// A hide-my-email alias.
    Alias,
    /// A payment card.
    CreditCard,
    /// A set of personal details.
    Identity,
    /// An SSH key pair.
    SshKey,
    /// Wi-Fi network credentials.
    Wifi,
    /// An item with user-defined fields.
    Custom,

    /// A kind this version of stevedore does not know.
    #[serde(other)]
    Unknown,
}

/// Whether an item is live or trashed.
///
/// `pass-cli` reports these capitalized (`"Active"`) but takes them lowercase
/// as `--filter-state active`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum State {
    /// Live in the vault.
    Active,
    /// In the trash, and so not counted as present.
    Trashed,

    /// A state this version of stevedore does not know.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub(super) struct ItemList {
    pub items: Vec<Item>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shapes observed from `pass-cli item list --output json`, 2026-07-29.
    const LIST: &str = r#"{
        "items": [
            {
                "id": "I1==",
                "share_id": "S1==",
                "vault_id": "V1==",
                "state": "Active",
                "flags": [],
                "create_time": "2026-07-29T03:03:59",
                "modify_time": "2026-07-29T03:04:02",
                "title": "a login",
                "item_type": "login"
            },
            {
                "id": "I2==",
                "share_id": "S1==",
                "vault_id": "V1==",
                "state": "Trashed",
                "flags": [],
                "create_time": "2026-07-29T03:04:33",
                "modify_time": "2026-07-29T03:04:34",
                "title": "a note",
                "item_type": "note"
            }
        ]
    }"#;

    #[test]
    fn parses_the_item_list() {
        let list: ItemList = serde_json::from_str(LIST).unwrap();
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].title, "a login");
        assert_eq!(list.items[0].item_type, Kind::Login);
        assert!(list.items[0].is_active());
        assert_eq!(list.items[1].item_type, Kind::Note);
        assert!(!list.items[1].is_active());
    }

    #[test]
    fn an_empty_vault_parses() {
        let list: ItemList = serde_json::from_str(r#"{"items":[]}"#).unwrap();
        assert!(list.items.is_empty());
    }

    #[test]
    fn the_kebab_case_kinds_parse() {
        let list: ItemList = serde_json::from_str(
            r#"{"items":[
                {"id":"1","share_id":"S","vault_id":"V","title":"t",
                 "item_type":"credit-card","state":"Active"},
                {"id":"2","share_id":"S","vault_id":"V","title":"t",
                 "item_type":"ssh-key","state":"Active"}
            ]}"#,
        )
        .unwrap();
        assert_eq!(list.items[0].item_type, Kind::CreditCard);
        assert_eq!(list.items[1].item_type, Kind::SshKey);
    }

    /// An unfamiliar kind or state must not fail the whole listing: a vault we
    /// cannot fully describe is still a vault we must not create duplicates in.
    #[test]
    fn an_unknown_kind_or_state_parses() {
        let list: ItemList = serde_json::from_str(
            r#"{"items":[
                {"id":"1","share_id":"S","vault_id":"V","title":"t",
                 "item_type":"something-new","state":"Archived"}
            ]}"#,
        )
        .unwrap();
        assert_eq!(list.items[0].item_type, Kind::Unknown);
        assert_eq!(list.items[0].state, State::Unknown);
        assert!(!list.items[0].is_active());
    }

    #[test]
    fn an_item_without_times_parses() {
        let list: ItemList = serde_json::from_str(
            r#"{"items":[
                {"id":"1","share_id":"S","vault_id":"V","title":"t",
                 "item_type":"login","state":"Active"}
            ]}"#,
        )
        .unwrap();
        assert!(list.items[0].create_time.is_none());
        assert!(list.items[0].flags.is_empty());
    }
}
