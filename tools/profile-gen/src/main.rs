mod parser;
mod writer;

use clap::Parser;
use lazy_static::lazy_static;
use std::fs;
use std::io::{Read, Write};

lazy_static! {
    pub(crate) static ref ROOT_DIR: std::path::PathBuf = std::env::current_dir().unwrap();
}

#[derive(Parser, Debug)]
#[command(version)]
struct CommandArgs {
    /// Garmin Profile file path
    #[arg(short, long)]
    path: String,

    /// Custom Garmin Profile file version, example 'xx.xx.xx'
    #[arg(long)]
    sdk_version: Option<String>,
}

fn main() {
    let args = CommandArgs::parse();
    let (version, bytes) = match read_profile_file(&args.path) {
        Ok((version, bytes)) => (
            {
                args.sdk_version
                    .unwrap_or_else(|| version.unwrap_or_else(|| panic!("Missing Profile version")))
            },
            bytes,
        ),
        Err(err) => panic!("{}", err),
    };
    let profile = parser::process_profile(&bytes);
    writer::process_types(&profile.types, &version);
    writer::process_messages(&profile.messages, &profile.types, &version);
    writer::process_version(&version);
    println!(
        "types size = {} messages size = {}",
        profile.types.len(),
        profile.messages.len()
    );
    // for message in &profile.messages {
    //     for field in &message.fields {
    //         if field.accumulate {
    //             println!("{}/{}", message.name, field.field_name);
    //         }
    //         for (_, component) in &field.components {
    //             if field.accumulate {
    //                 println!(
    //                     "{}/{}/{}",
    //                     message.name, field.field_name, component.field_name
    //                 );
    //             }
    //         }
    //     }
    // }
    // println!("喵喵喵?");
}

fn read_profile_file(path: &str) -> Result<(Option<String>, Vec<u8>), String> {
    let path = {
        let path = std::path::PathBuf::from(path);
        let path = if path.is_absolute() {
            path
        } else {
            ROOT_DIR.join(path)
        };
        if path.is_dir() {
            path.join("Profile.xlsx")
        } else {
            path
        }
    };
    if !path.exists() {
        return Err(format!(
            "Profile file \"{:?}\" not exists",
            path.components()
                .map(|it| it.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>()
                .join("/")
                .replace('\\', "")
        ));
    }
    match path.extension().and_then(|it| it.to_str()).unwrap() {
        "zip" => {
            let mut archive = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
            let name = archive
                .file_names()
                .find(|it| it.contains("Profile.xlsx"))
                .map(|it| it.to_string())
                .unwrap();
            let file = archive.by_name(&name).unwrap();
            let version = extract_sdk_version(name.split('/').collect::<Vec<_>>()[0]);
            Ok((
                version,
                file.bytes().map(|byte| byte.unwrap()).collect::<Vec<_>>(),
            ))
        }
        "xlsx" => {
            let mut bytes = Vec::new();
            fs::File::open(&path)
                .unwrap()
                .write_all(&mut bytes)
                .unwrap();
            Ok((None, bytes))
        }
        other => Err(format!("Not supported '{other}' file type")),
    }
}
fn extract_sdk_version(str: &str) -> Option<String> {
    let parts = str
        .replace("FitSDKRelease_", "")
        .split('.')
        .map(|it| it.to_string())
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        None
    } else {
        Some(parts.join("."))
    }
}
