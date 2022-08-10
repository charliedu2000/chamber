mod client;
mod client_ui;
mod server;

mod consts;
mod message;
mod utils;

mod paragraph_chamber;
mod reflow_chamber;

use std::{any::type_name, env};

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

fn main() {
    println!("{}", type_of('😀'));
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from(consts::ARG_CLIENT)) {
        println!("Start client!");
        let _res = client::start();
    } else if args.contains(&String::from(consts::ARG_SERVER)) {
        println!("Start server!");
        let _res = server::start();
    } else if args.contains(&"ui".to_string()) {
        let _res = client_ui::ui_init();
    }
}
