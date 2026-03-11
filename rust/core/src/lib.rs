pub mod config;
pub mod logger;
pub mod paths;
pub mod runner;
pub mod router;
pub mod session;
pub mod ikuai;
pub mod update;

pub fn hello_core() -> &'static str {
    "ikb-core"
}
