pub mod constants;
pub mod epoch_manager;
pub mod fee;
pub mod incentive_manager;
pub mod pool_manager;
pub mod coin;
pub mod common;
pub mod lp_common;

pub mod tokenfactory;

#[allow(clippy::all)]
mod uints {
    use uint::construct_uint;
    construct_uint! {
        pub struct U256(4);
    }
}

pub use uints::U256;