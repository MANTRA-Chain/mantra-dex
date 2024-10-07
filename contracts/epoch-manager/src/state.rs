use amm::epoch_manager::Config;
use cw_controllers::Admin;
use cw_storage_plus::Item;

pub const CONFIG: Item<Config> = Item::new("config");
pub const ADMIN: Admin = Admin::new("admin");
