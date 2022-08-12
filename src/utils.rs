use std::thread;

pub fn sleep_100_millis() {
    thread::sleep(std::time::Duration::from_millis(100));
}

pub fn string_to_char_vec(origin_string: &String) -> Vec<char> {
    origin_string.chars().collect()
}

pub fn char_vec_to_string(char_vec: &Vec<char>) -> String {
    char_vec.iter().collect::<String>()
}

pub fn char_arr_to_string(char_vec: &[char]) -> String {
    char_vec.iter().collect::<String>()
}
