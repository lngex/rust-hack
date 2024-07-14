use windows::Win32::{Foundation::HINSTANCE,
                     System::SystemServices::DLL_PROCESS_ATTACH};

mod support;
mod memory;
mod pvzui;
mod hook;

#[no_mangle]
extern "system" fn DllMain(_dll_module: HINSTANCE, call_reason: u32, _reserved: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(move || {
            pvzui::run();
        });
    }
    true
}

#[cfg(test)]
mod tests {
    use imgui::Condition;
    use crate::support;

    const BASE_ADDRESS: u32 = 0x006A9EC0;

    #[test]
    fn test() {

    }


    #[test]
    fn test1() {
        // let buf = PathBuf::from(r"E:\rust\project\pvz-imgui-rs\target\i686-pc-windows-msvc\release\a_pvz_imgui_rs.dll");
        // Process::by_name("MyTargetApplication.exe").unwrap().inject(buf).unwrap();
    }
}