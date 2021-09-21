use pelite::pe::{Pe, PeView};
use winapi::um::consoleapi::AllocConsole;
use winapi::um::handleapi::CloseHandle;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::processthreadsapi::ExitProcess;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::processthreadsapi::OpenThread;
use winapi::um::processthreadsapi::SuspendThread;
use winapi::um::tlhelp32::CreateToolhelp32Snapshot;
use winapi::um::tlhelp32::Thread32First;
use winapi::um::tlhelp32::Thread32Next;
use winapi::um::tlhelp32::TH32CS_SNAPTHREAD;
use winapi::um::tlhelp32::THREADENTRY32;
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
            let mut version = "".to_string();
            if let Ok(version_info) = resources.version_info() {
                if let Some(lang) = version_info.translation().get(0) {
                    if let Some(product_version) = version_info.value(*lang, "ProductVersion") {
                        version = product_version.replace("\0", "").to_string();
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

    pub fn scan_memory<F, R>(&self, callback: F) -> Option<R>
    where
        F: Fn(&[u8]) -> Option<R>,
    {
        let mut remain = self.image_size as usize;
        let process = unsafe { GetCurrentProcess() };
        let mut buffer = [0u8; 0x2000];
        let mut last_page_size = 0;
        loop {
            if remain == 0 {
                return None;
            }
            let page_size = if remain % 0x1000 != 0 {
                remain % 0x1000
            } else {
                0x1000
            };
            remain -= page_size;
            unsafe {
                let offset = (self.base + remain) as *const _;
                if IsBadReadPtr(offset, page_size) != 0 {
                    last_page_size = 0;
                    continue;
                }
                let copy_src = &buffer[0..last_page_size] as *const _ as *const u8;
                let copy_dst =
                    &mut buffer[page_size..page_size + last_page_size] as *mut _ as *mut u8;
                core::ptr::copy(copy_src, copy_dst, last_page_size);
                let read_dst = &mut buffer[0..page_size] as *mut _ as *mut _;
                if ReadProcessMemory(process, offset, read_dst, page_size, core::ptr::null_mut())
                    == 0
                {
                    last_page_size = 0;
                    continue;
                }
            }
            if let Some(result) = callback(&buffer[0..page_size + last_page_size]) {
                return Some(result);
            }
        }
    }
}

pub fn pause_threads() {
    unsafe {
        let process = GetCurrentProcessId();
        let current_thread_id = GetCurrentThreadId();
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, process);
        if snapshot == INVALID_HANDLE_VALUE {
            panic!("Snapshot invalid handle!");
        }
        let mut te32: THREADENTRY32 = core::mem::zeroed();
        te32.dwSize = core::mem::size_of::<THREADENTRY32>() as u32;
        if Thread32First(snapshot, &mut te32) == 0 {
            CloseHandle(snapshot);
            panic!("Failed to iterate thread!");
        }
        loop {
            if te32.th32OwnerProcessID == process && te32.th32ThreadID != current_thread_id {
                let thread = OpenThread(THREAD_ALL_ACCESS, 0, te32.th32ThreadID);
                if thread == INVALID_HANDLE_VALUE {
                    panic!("Thread invalid handle!");
                }
                SuspendThread(thread);
                CloseHandle(thread);
            }
            if Thread32Next(snapshot, &mut te32) == 0 {
                break;
            }
        }
        CloseHandle(snapshot);
    }
}

pub fn alloc_console() {
    unsafe {
        AllocConsole();
    }
}

pub fn exit_process(code: u32) {
    unsafe {
        ExitProcess(code);
    }
}
