use std::fs::File;
use std::io::Write;

use imgui::Condition;
use windows::core::s;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Threading::GetCurrentThreadId;

use crate::hook;
use crate::memory;
use crate::support;

pub fn run() {
    let thread_id = unsafe { GetCurrentThreadId() };
    // 模块地址
    let base_module_address = unsafe { GetModuleHandleA(s!("PlantsVsZombies.exe")).map(|h| h.0 as u32) }.unwrap();
    // 阳光值
    let mut binding = "".to_string();
    // 冷却速度
    let mut cool_speed = "".to_string();
    let mut is_hook = false;
    let mut binding_hook = hook::Hook::new(0x00430A11, 0, 0, 0, vec![thread_id]);
    let mut msg = String::new();
    support::simple_init(file!(), move |_, ui| {
        ui.window("Hello world")
            .size([300.0, 320.0], Condition::FirstUseEver)
            .build(|| {
                ui.text_wrapped(format!("阳光值->{}", memory::get_sunlight()));
                ui.text_wrapped(format!("冷却速度->{}", memory::read_cooling_accelerate(base_module_address)));
                let _ = ui.input_text("阳光", &mut binding).build();
                if ui.button("修改阳光") {
                    match binding.parse::<u32>() {
                        Ok(value) => { memory::update_sunlight(value) }
                        Err(_) => { binding = "输入的不是数字".to_string() }
                    }
                }
                let _ = ui.input_text("冷却", &mut cool_speed).build();
                if ui.button("修改冷却速度") {
                    match cool_speed.parse::<u8>() {
                        Ok(value) => { cool_speed = memory::cooling_accelerate(base_module_address, value) }
                        Err(_) => { cool_speed = "输入的不是数字".to_string() }
                    }
                }
                if ui.button("僵尸增加") {
                    memory::increase_zombie()
                }
                if ui.checkbox("hook", &mut is_hook) {
                    if is_hook {
                        binding_hook.set_hook_fn();
                        let (b,c,m,v) = binding_hook.hook();
                        if b {
                            msg = format!("hook成功,hook线程数{},msg:{},hook的线程id:{:?},自身线程id：{}",c,m,v,thread_id).to_string();
                            let mut  log = File::create("E:/rust/project/pvz-imgui-rs/log/hook.log").unwrap();
                           log.write(msg.as_bytes());
                        } else {
                            msg = "hook失败".to_string();
                        }
                    } else {
                        binding_hook.unhook();
                    }
                }
                ui.text_wrapped(&msg);
            });
    });
}