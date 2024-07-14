use std::fs::OpenOptions;
use std::io::Write;
use std::mem::size_of_val;
use std::ptr::addr_of_mut;

use windows::core::imp::CloseHandle;
use windows::Win32::Foundation::{EXCEPTION_SINGLE_STEP, FALSE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, CONTEXT, CONTEXT_ALL_X86, EXCEPTION_CONTINUE_EXECUTION, EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS, GetThreadContext, PVECTORED_EXCEPTION_HANDLER, RemoveVectoredExceptionHandler, SetThreadContext};
use windows::Win32::System::Diagnostics::ToolHelp::{TH32CS_SNAPTHREAD, Thread32First, Thread32Next, THREADENTRY32};
use windows::Win32::System::Threading::{GetCurrentProcessId, OpenThread, ResumeThread, SuspendThread, THREAD_ALL_ACCESS};

/// 硬件hook结构体
pub struct Hook {
    count: u32,
    /// 设置回调函数时返回的地址
    result_ptr: Option<*mut core::ffi::c_void>,
    /// hook的地址
    dr0: u64,
    dr1: u64,
    dr2: u64,
    dr3: u64,
    /// 0x55
    dr7: u64,
    /// 不用hook的线程id
    no_hook_thread: Vec<u32>,
    /// 异常时的回调函数
    hook_callback: PVECTORED_EXCEPTION_HANDLER,
}

impl Hook {
    /// 构建hook对象
    pub fn new(dr0: u64,
               dr1: u64,
               dr2: u64,
               dr3: u64,
               no_hook_thread: Vec<u32>) -> Self {
        Self { count: 0, result_ptr: None, dr0, dr1, dr2, dr3, dr7: 0x55, no_hook_thread, hook_callback: Some(sunlight) }
    }

    /// hook
    pub fn hook(&self) -> (bool, i32, String, Vec<u32>) {
        let mut vec: Vec<u32> = Vec::new();
        let mut count = 0;
        let mut msg: String = String::new();
        unsafe {
            let result = windows::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if result.is_err() {
                // 如果出现错误返回失败
                return (false, count, msg, vec);
            }
            let process_snapsho_handle = result.unwrap();
            if process_snapsho_handle == INVALID_HANDLE_VALUE {
                // 如果句柄无效返回失败
                return (false, count, msg, vec);
            }
            // 创建一个变量存储线程信息
            let mut threadentry32: THREADENTRY32 = THREADENTRY32::default();
            threadentry32.dwSize = size_of_val(&threadentry32) as u32;
            let threadentry32_ptr = addr_of_mut!(threadentry32);
            let result1 = Thread32First(process_snapsho_handle, threadentry32_ptr);
            // 获取第一个线程
            if result1.is_ok() {
                msg.push_str("第一个线程获取到");
                loop {
                    if threadentry32.th32OwnerProcessID == GetCurrentProcessId() {
                        // 判断线程是否可以hook
                        if self.is_hook_thread(threadentry32.th32ThreadID) {
                            // 如果可以则获取线程权限以及上下文
                            match OpenThread(THREAD_ALL_ACCESS, FALSE, threadentry32.th32ThreadID) {
                                Ok(thread_handle) => {
                                    // 暂停线程
                                    SuspendThread(thread_handle);
                                    let mut context = CONTEXT::default();
                                    context.ContextFlags = CONTEXT_ALL_X86;
                                    let context_ptr = addr_of_mut!(context);
                                    // 获取线程上下文
                                    let _ = GetThreadContext(thread_handle, context_ptr);
                                    // 设置断点
                                    context.Dr0 = self.dr0 as _;
                                    context.Dr1 = self.dr1 as _;
                                    context.Dr2 = self.dr2 as _;
                                    context.Dr3 = self.dr3 as _;
                                    context.Dr7 = self.dr7 as _;
                                    // 设置线程上下文
                                    let _ = SetThreadContext(thread_handle, context_ptr);
                                    ResumeThread(thread_handle);
                                    CloseHandle(thread_handle.0);
                                    count += 1;
                                    vec.push(threadentry32.th32ThreadID);
                                }
                                Err(err) => {
                                    msg.push_str("线程句柄获取失败");
                                    msg.push_str(err.to_string().as_str());
                                }
                            }
                        }
                    }
                    // 如果线程还有则获取，没有跳出循环
                    if !Thread32Next(process_snapsho_handle, threadentry32_ptr).is_ok() {
                        break;
                    }
                }
            } else {
                msg.push_str(result1.err().unwrap().to_string().as_str())
            }
            // 释放句柄
            CloseHandle(process_snapsho_handle.0);
        }
        return (true, count, msg, vec);
    }

    /// 安装hook函数
    pub fn set_hook_fn(&mut self) {
        if self.count == 0 {
            self.count = 1;
            unsafe {
                let handler = AddVectoredExceptionHandler(0, self.hook_callback);
                self.result_ptr = Some(handler);
            }
        }
    }

    /// 取消hokk
    pub fn unhook(&mut self) {
        self.dr0 = 0;
        self.dr1 = 0;
        self.dr2 = 0;
        self.dr3 = 0;
        self.dr7 = 0;
        let _ = self.hook();
        unsafe { RemoveVectoredExceptionHandler(self.result_ptr.unwrap()) };
        if self.count == 1 {
            self.count = 0;
        }
    }

    /// 判断线程是否可以hook
    fn is_hook_thread(&self, thread_id: u32) -> bool {
        !self.no_hook_thread.contains(&thread_id)
    }
}


///  阳光回调函数
#[no_mangle]
unsafe extern "system" fn sunlight(exceptioninfo: *mut EXCEPTION_POINTERS) -> i32 {
    let mut exceptioninfo_value = exceptioninfo.read();
    let mut record_value = exceptioninfo_value.ExceptionRecord.read();
    if record_value.ExceptionCode == EXCEPTION_SINGLE_STEP {
        if (record_value.ExceptionAddress as u32) == 0x00430A11 {
            let mut context = exceptioninfo_value.ContextRecord.read();
            context.Ecx = 32;
            // 执行当前hook的汇编
            let ptr = (context.Eax + 0x5560) as *mut u32;
            ptr.write(ptr.read() + context.Ecx);
            // 我们更改后执行下一个汇编
            context.Eip = context.Eip + 6;
            return EXCEPTION_CONTINUE_EXECUTION;
        }
    }
    return EXCEPTION_CONTINUE_SEARCH;
}