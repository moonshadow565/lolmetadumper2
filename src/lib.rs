#![feature(abi_thiscall)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![macro_use]
extern crate serde;
extern crate serde_json;

mod meta;
mod meta_dump;
mod module_info;
use winapi::um::processthreadsapi::ExitProcess;
use winapi::um::consoleapi::AllocConsole;

unsafe fn main() {
    AllocConsole();
    println!("Started!");
    let module_info = module_info::ModuleInfo::create();
    module_info.print_info();
    module_info.dump_meta_info_file("meta");
    println!("Done!");
    ExitProcess(0);
}

mod bugsplat_dll {
    #[export_name = "??0MiniDmpSender@@QAE@PB_W000K@Z"]
    pub unsafe extern "system" fn _0(_1: usize) {
        super::main();
    }

    #[export_name = "?setCallback@MiniDmpSender@@QAEXP6A_NIPAX0@Z@Z"]
    pub unsafe extern "system" fn _1() {}
    #[export_name = "?setLogFilePath@MiniDmpSender@@QAEXPB_W@Z"]
    pub unsafe extern "system" fn _2() {}
    #[export_name = "?unhandledExceptionHandler@MiniDmpSender@@QAEJPAU_EXCEPTION_POINTERS@@@Z"]
    pub unsafe extern "system" fn _3() {}
    #[export_name = "?resetAppIdentifier@MiniDmpSender@@QAEXPB_W@Z"]
    pub unsafe extern "system" fn _4() {}
    #[export_name = "?setMiniDumpType@MiniDmpSender@@QAEXW4_BS_MINIDUMP_TYPE@1@@Z"]
    pub unsafe extern "system" fn _5() {}
    #[export_name = "??1MiniDmpSender@@UAE@XZ"]
    pub unsafe extern "system" fn _6() {}
    #[export_name = "?sendAdditionalFile@MiniDmpSender@@QAEXPB_W@Z"]
    pub unsafe extern "system" fn _7() {}
}
