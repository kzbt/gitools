use crate::git::RepoHeader;
use druid::{Data, Lens, Size, WidgetId};
use im::{vector, Vector};
use serde::{Deserialize, Deserializer};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub win_size: Size,
    pub repo_header: RepoHeader,
    pub cheatsheet: CheatSheetState,
    pub fuzzybar: FuzzybarState,
    pub git: GitState,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct CheatSheetState {
    pub is_hidden: bool,
    pub keymap: KeyMap,
    pub current_node: u8,
    pub current_level: KeyMapLevel,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct FuzzybarState {
    pub is_hidden: bool,
    pub cmd: Command,
    pub query: String,
    pub source: Vector<String>,
    pub filtered: Vector<ListItem>,
}

#[derive(Clone, Data, Lens, Debug, PartialEq)]
pub struct ListItem {
    pub name: String,
    pub selected: bool,
}

impl FuzzybarState {
    pub fn filter(&mut self) {
        let mut new = vector![];

        let mut selected = false;
        let mut found = true;
        for name in self.source.iter() {
            if name.contains(&self.query) {
                if found && !selected {
                    selected = true;
                    found = false;
                }

                if new.len() >= 20 {
                    break;
                }

                new.push_back(ListItem {
                    name: name.to_owned(),
                    selected,
                });

                selected = false;
            }
        }

        self.filtered = new;
    }
}

#[derive(Clone, Copy, PartialEq, Data, Debug, Deserialize)]
pub enum Command {
    ShowMenu,
    BranchCheckout,
    Commit,
}

#[derive(Clone, Data, Lens, Debug)]
pub struct GitState {
    pub local_branches: Vector<String>,
    pub remote_branches: Vector<String>,
    pub all_branches: Vector<String>,
}

pub type KeyMap = Rc<BTreeMap<u8, L1Node>>;
pub type KeyMapL2 = Rc<BTreeMap<u8, L2Node>>;

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

fn de_keymap<'de, D>(deserializer: D) -> Result<Rc<BTreeMap<u8, L1Node>>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_map = BTreeMap::<String, L1Node>::deserialize(deserializer)?;
    let result = str_map
        .into_iter()
        .map(|(k, v)| {
            let ku8 = *k.as_bytes().get(0).unwrap();
            (ku8, v)
        })
        .collect();

    Ok(Rc::new(result))
}

fn de_keymap_l2<'de, D>(deserializer: D) -> Result<Rc<BTreeMap<u8, L2Node>>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_map = BTreeMap::<String, L2Node>::deserialize(deserializer)?;
    let result = str_map
        .into_iter()
        .map(|(k, v)| {
            let ku8 = *k.as_bytes().get(0).unwrap();
            (ku8, v)
        })
        .collect();

    Ok(Rc::new(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzybarstate_should_filter() {
        let mut s = FuzzybarState {
            is_hidden: true,
            cmd: Command::ShowMenu,
            query: "b".to_owned(),
            source: vector![
                "aa".to_owned(),
                "ab".to_owned(),
                "bc".to_owned(),
                "bca".to_owned()
            ],
            filtered: vector![],
        };

        s.filter();
        let expected = vector![
            ListItem {
                name: "ab".to_owned(),
                selected: true
            },
            ListItem {
                name: "bc".to_owned(),
                selected: false
            },
            ListItem {
                name: "bca".to_owned(),
                selected: false
            },
        ];
        assert_eq!(expected, s.filtered);
    }

    #[test]
    fn filtered_should_have_limited_items() {
        let source = (1..200)
            .map(|s| format!("Item {}", s))
            .collect::<Vector<String>>();
        let mut s = FuzzybarState {
            is_hidden: true,
            cmd: Command::ShowMenu,
            query: "2".to_owned(),
            source,
            filtered: vector![],
        };

        s.filter();
        assert_eq!(20, s.filtered.len());
    }
}
