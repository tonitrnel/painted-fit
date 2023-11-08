mod parser;
mod writer;

use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref ROOT_DIR: std::path::PathBuf = std::env::current_dir().unwrap();
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    if args.is_empty() {
        panic!("Missing profile file path");
    }
    let profile_path = {
        let path = std::path::PathBuf::from(&args[0]);
        if path.is_absolute() {
            path
        } else {
            ROOT_DIR.join(&args[0])
        }
    };
    if !profile_path.exists() {
        panic!("Profile file \"{}\" not exists", profile_path.display());
    }
    let profile = parser::process_profile(&profile_path);
    writer::process_types(&profile.types);
    writer::process_messages(&profile.messages);
    println!(
        "types size = {} messages size = {}",
        profile.types.len(),
        profile.messages.len()
    );
}
