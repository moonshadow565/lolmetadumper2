use pelite::pattern;
use pelite::pe::{Pe, PeView, Rva};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::handleapi::CloseHandle;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::processthreadsapi::OpenThread;
use winapi::um::processthreadsapi::SuspendThread;
use winapi::um::winbase::IsBadReadPtr;
use winapi::um::winnt::THREAD_ALL_ACCESS;
use winapi::um::{libloaderapi::GetModuleHandleA, processthreadsapi::GetCurrentProcessId};

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
            let mut version = String::new();
            if let Ok(version_info) = resources.version_info() {
                if let Some(lang) = version_info.translation().get(0) {
                    if let Some(product_version) = version_info.value(*lang, "ProductVersion") {
                        version = product_version.replace('\0', "");
                    }
                }
            }
            Self {
                base,
                version,
                image_size,
            }
        }
    }

    pub fn scan_memory(&self, pat: &str) -> Option<usize>
    {
        // TODO: remove this, leftover from using regex for scanning
        let mut remain = self.image_size as usize;
        while remain != 0 {
            let page_size = Some(remain % 0x1000).filter(|&x| x != 0).unwrap_or(0x1000);
            remain -= page_size;
            let offset = (self.base + remain) as *const _;
            unsafe {
                if IsBadReadPtr(offset, page_size) != 0 {
                    continue;
                }
            }
        }

        let module = unsafe { PeView::module(self.base as *const _) };
        let scanner = module.scanner();
        let pattern = pattern::parse(pat).expect("Failed to parse pattern");
        let mut save = [Rva::default(); 2];
        if scanner.finds_code(&pattern, &mut save) {
            Some(self.base + save[1] as usize)
        } else {
            None
        }
    }
}

pub fn pause_threads() {
    unsafe {
        let process = GetCurrentProcessId();
        let current_thread_id = GetCurrentThreadId();

        for te32 in tlhelp32::Snapshot::new_thread().expect("Failed to create snapshot") {
            if te32.owner_process_id == process && te32.thread_id != current_thread_id {
                let thread = OpenThread(THREAD_ALL_ACCESS, 0, te32.thread_id);
                assert_ne!(thread, INVALID_HANDLE_VALUE, "Failed to open thread");
                SuspendThread(thread);
                CloseHandle(thread);
            }
        }
    }
}

pub fn alloc_console() {
    unsafe { AllocConsole() };
}

pub fn exit_process(code: u32) {
    std::process::exit(code as i32);
}
