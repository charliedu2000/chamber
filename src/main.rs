mod client;
mod server;

mod consts;
mod message;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from(consts::ARG_CLIENT)) {
        println!("Start client!");
        client::init();
    } else if args.contains(&String::from(consts::ARG_SERVER)) {
        println!("Start server!");
        server::init();
    }
}
