use crate::git::RepoHeader;
use druid::{Data, Lens};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub repo_header: RepoHeader,
    pub cheatsheet: CheatSheetState,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct CheatSheetState {
    pub is_hidden: bool,
    pub keymap: KeyMap,
    pub current_node: u8,
    pub current_level: KeyMapLevel,
}

#[derive(Clone, PartialEq, Data, Debug, Deserialize)]
pub enum Command {
    ShowMenu,
    BranchCheckout,
    Commit,
}

pub type KeyMap = Rc<HashMap<u8, L1Node>>;
pub type KeyMapL2 = Rc<HashMap<u8, L2Node>>;

#[derive(Clone, Data, Debug, Deserialize)]
pub struct L1Node {
    #[serde(deserialize_with = "de_u8_from_string")]
    pub key: u8,
    pub name: String,
    #[serde(deserialize_with = "de_keymap_l2")]
    pub next: KeyMapL2,
}

#[derive(Clone, Data, Debug, Deserialize)]
pub struct L2Node {
    #[serde(deserialize_with = "de_u8_from_string")]
    pub key: u8,
    pub name: String,
    pub command: Command,
}

#[derive(Clone, PartialEq, Data, Debug)]
pub enum KeyMapLevel {
    L1,
    L2(u8),
}

#[derive(Debug, Deserialize)]
pub struct KeyMapConfig {
    #[serde(deserialize_with = "de_keymap")]
    pub map: KeyMap,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub keymap: KeyMapConfig,
}

fn de_u8_from_string<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    struct DeStrAsU8;

    impl<'de> serde::de::Visitor<'de> for DeStrAsU8 {
        type Value = u8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("single ascii char")
        }

        fn visit_string<E>(self, value: String) -> Result<u8, E>
        where
            E: serde::de::Error,
        {
            Ok(*value.as_bytes().get(0).unwrap())
        }

        fn visit_str<E>(self, value: &str) -> Result<u8, E>
        where
            E: serde::de::Error,
        {
            Ok(*value.as_bytes().get(0).unwrap())
        }
    }

    deserializer.deserialize_str(DeStrAsU8)
}

fn de_keymap<'de, D>(deserializer: D) -> Result<Rc<HashMap<u8, L1Node>>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_map = HashMap::<String, L1Node>::deserialize(deserializer)?;
    let result = str_map
        .into_iter()
        .map(|(k, v)| {
            let ku8 = *k.as_bytes().get(0).unwrap();
            (ku8, v)
        })
        .collect();

    Ok(Rc::new(result))
}

fn de_keymap_l2<'de, D>(deserializer: D) -> Result<Rc<HashMap<u8, L2Node>>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_map = HashMap::<String, L2Node>::deserialize(deserializer)?;
    let result = str_map
        .into_iter()
        .map(|(k, v)| {
            let ku8 = *k.as_bytes().get(0).unwrap();
            (ku8, v)
        })
        .collect();

    Ok(Rc::new(result))
}
