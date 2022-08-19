mod client;
mod client_ui;
mod server;

mod consts;
mod message;
mod utils;

mod paragraph_chamber;
mod reflow_chamber;

use std::env;

fn main() {
    // println!("{}, {}", "一".len(), "一二".len());
    // let s: String = "hello".to_string();
    // let mut arr: Vec<char> = s.chars().collect();
    // println!("{:?}", arr);
    // println!("{:?}", arr.remove(1));
    // println!("{:?}", arr);
    // let str1 = String::from("asd啊");
    // println!("{}", str1.len());
    // let mut temp = String::from("asd啊asd");
    // assert_eq!(temp.remove(3), '啊');

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
