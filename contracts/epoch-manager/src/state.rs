use cw_storage_plus::Item;
use mantra_dex_std::epoch_manager::Config;

pub const CONFIG: Item<Config> = Item::new("config");
