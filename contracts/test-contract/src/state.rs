use cw_storage_plus::{Item};

pub const RANDOMNESS: Item<String> = Item::new("randomness");