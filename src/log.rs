use crate::utils::Utils;
use colored::Colorize;

pub struct Log {}
impl Log {
    pub fn info(output: &str) {
        println!("{} {}", Utils::get_readable_timestamp(), output.blue());
    }

    pub fn success(output: &str) {
        println!("{} {}", Utils::get_readable_timestamp(), output.green());
    }

    pub fn warn(output: &str) {
        println!("{} {}", Utils::get_readable_timestamp(), output.yellow());
    }

    pub fn error(output: &str) {
        println!("{} {}", Utils::get_readable_timestamp(), output.red());
    }
}
