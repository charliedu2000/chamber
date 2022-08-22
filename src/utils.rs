use std::thread;

pub fn sleep_100_millis() {
    thread::sleep(std::time::Duration::from_millis(100));
}

/// Convert a `&String` to a `Vec<char>`
pub fn string_to_char_vec(origin_string: &String) -> Vec<char> {
    origin_string.chars().collect()
}

/// Convert a `&Vec<char>` to a `String`
pub fn char_vec_to_string(char_vec: &Vec<char>) -> String {
    char_vec.iter().collect::<String>()
}

/// Convert a char array (`&[char]`) to a `String`
pub fn char_arr_to_string(char_vec: &[char]) -> String {
    char_vec.iter().collect::<String>()
}
