use super::meta::{Class, RiotVector};
use super::meta_dump::dump_class_list;
use pelite::pe::{Pe, PeView};
use regex::bytes::Regex;
use serde_json::json;
use serde_json::Value;
use std::fs::{self, File};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::winbase::IsBadReadPtr;

const PATTERNS: &[&str] = &[
    r"(?s-u)\x83\x3D....\xFF\x75\xE4\x68....\xC7\x05(....)\x00\x00\x00\x00\xC7\x05....\x00\x00\x00\x00\xC7\x05....\x00\x00\x00\x00\xC6\x05....\x00\xE8",
];

pub struct ModuleInfo {
    pub base: usize,
    pub version: String,
    pub image_size: usize,
}

impl ModuleInfo {
    pub fn create() -> Self {
        unsafe {
            let base = GetModuleHandleA(core::ptr::null()) as *const _ as usize;
            let module = PeView::module(base as *const _);
            let code_base = module.optional_header().BaseOfCode as usize;
            let code_size = module.optional_header().SizeOfCode as usize;
            let image_size = code_base + code_size;
            let resources = module.resources().expect("Failed to open resources");
            let version_info = resources
                .version_info()
                .expect("Failed to find version info!");
            let lang = version_info
                .translation()
                .get(0)
                .expect("Failed to find resource language!");
            let version = version_info
                .value(*lang, "ProductVersion")
                .expect("Failed to find version string")
                .replace("\0", "")
                .to_string();
            Self {
                base,
                version,
                image_size,
            }
        }
    }

    pub fn print_info(&self) {
        println!("Base: 0x{:X}", self.base);
        println!("ImageSize: 0x{:X}", self.image_size);
        println!("Version: {}", &self.version);
    }

    pub fn dump_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.image_size);
        data.resize(self.image_size, 0);
        let mut remain = self.image_size as usize;
        let process = unsafe { GetCurrentProcess() };
        loop {
            if remain == 0 {
                break;
            }
            let page_size = if remain % 0x1000 != 0 {
                remain % 0x1000
            } else {
                0x1000
            };
            remain -= page_size;
            let offset = (self.base + remain) as *const _;
            unsafe {
                if IsBadReadPtr(offset, page_size) == 0 {
                    let dest = &mut data[remain] as *mut _ as *mut _;
                    ReadProcessMemory(process, offset, dest, page_size, core::ptr::null_mut());
                }
            }
        }
        data
    }

    pub fn find_meta(&self) -> Option<&RiotVector<&Class>> {
        let data = self.dump_data();
        for &pattern in PATTERNS {
            let regex = Regex::new(pattern).expect("Bad pattern!");
            if let Some(captures) = regex.captures(&data) {
                let result = captures.get(1).unwrap().as_bytes().as_ptr();
                if result != core::ptr::null() {
                    unsafe {
                        let offset = *(result as *const *const RiotVector<&Class>);
                        if offset != core::ptr::null() {
                            return Some(&*offset);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn dump_meta_info(&self) -> Value {
        let classes = self.find_meta().expect("Failed to find metaclasses");
        json!({
            "version": self.version,
            "classes": dump_class_list(self.base, classes.slice()),
        })
    }

    pub fn dump_meta_info_file(&self, folder: &str) {
        let meta_info = self.dump_meta_info();
        fs::create_dir_all(folder).expect("Failed to create folder!");
        let path = format!("{}/meta_{}.json", folder, self.version);
        let file = File::create(path).expect("Failed to create meta file!");
        serde_json::to_writer_pretty(file, &meta_info).expect("Failed to serialize json!");
    }
}
