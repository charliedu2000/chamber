mod client;
mod server;

mod consts;
mod message;
mod utils;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from(consts::ARG_CLIENT)) {
        println!("Start client!");
        let _res = client::start();
    } else if args.contains(&String::from(consts::ARG_SERVER)) {
        println!("Start server!");
        let _res = server::start();
    }
}
