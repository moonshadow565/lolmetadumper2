#![feature(abi_thiscall)]
#![allow(dead_code)]

mod meta;
mod meta_dump;
mod native;

use std::fs::{self, File};
use std::io::Write;

use regex::bytes::Regex;
use serde_json::json;
use winapi::um::winnt::DLL_PROCESS_ATTACH;

const PATTERN: &str = r"(?s-u)\x83\x3D....\xFF\x75\xE4\x68....\xC7\x05(....)\x00\x00\x00\x00\xC7\x05....\x00\x00\x00\x00\xC7\x05....\x00\x00\x00\x00\xC6\x05....\x00\xE8";
type MetaVector = *const meta::RiotVector<&'static meta::Class>;

fn main() {
    let folder = "meta";
    native::alloc_console();
    let regex = Regex::new(PATTERN).expect("Bad pattern!");

    println!("Fetching module info...");
    let info = native::ModuleInfo::create();

    println!("Base: {:#X}", info.base);
    println!("ImageSize: {:#X}", info.image_size);
    println!("Version: {}", &info.version);

    println!("Stopping other threads!");
    native::pause_threads();

    println!("Finding metaclasses...");
    let classes = info
        .scan_memory(|data| {
            regex
                .captures(data)
                .and_then(|captures| captures.get(1))
                .map(|x| x.as_bytes().as_ptr().cast::<MetaVector>())
                .and_then(|x| unsafe { x.as_ref() })
                .and_then(|x| unsafe { x.as_ref() })
        })
        .expect("Failed to find metaclasses");

    println!("Processing classes...");
    let meta_info = json!({
        "version": info.version,
        "classes": meta_dump::dump_class_list(info.base, classes.slice()),
    });

    println!("Serializing classes...");
    let json_data = serde_json::to_vec_pretty(&meta_info).expect("Failed to serialize json!");

    println!("Writing to file...");
    fs::create_dir_all(folder).expect("Failed to create folder!");
    File::create(format!("{}/meta_{}.json", folder, info.version))
        .expect("Failed to create meta file!")
        .write_all(&json_data)
        .expect("Failed to write to file!");

    println!("Done!");
    native::exit_process(0);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(_: usize, reason: u32, _: usize) -> u32 {
    if reason == DLL_PROCESS_ATTACH {
        main();
    }

    1
}
