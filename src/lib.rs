#![feature(abi_thiscall)]
#![allow(dead_code)]

mod meta;
mod meta_dump;
mod native;

use std::fs::{self, File};
use std::io::Write;

use serde_json::json;
use winapi::um::winnt::DLL_PROCESS_ATTACH;

const PATTERN: &str = "83 3D ? ? ? ? FF 75 DF 33 C0 48 8D 0D ? ? ? ? 48 89 05 $ { ' } 48 89 05";
type MetaVector = meta::RiotVector<&'static meta::Class>;

fn main() {
    let folder = "meta";
    native::alloc_console();

    println!("Fetching module info...");
    let info = native::ModuleInfo::create();

    println!("Base: {:#X}", info.base);
    println!("ImageSize: {:#X}", info.image_size);
    println!("Version: {}", &info.version);

    println!("Stopping other threads!");
    native::pause_threads();

    println!("Finding metaclasses...");
    let classes = info
        .scan_memory(PATTERN)
        .map(|x| x as *const MetaVector)
        .and_then(|x| unsafe { x.as_ref() })
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
