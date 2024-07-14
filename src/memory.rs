use std::arch::asm;
use std::ptr::addr_of_mut;

use windows::Win32::System::Memory::{PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect};

/// 阳关 基址
const BASE_ADDRESS: u32 = 0x006A9EC0;
/// 第一个偏移
const OFFSET1: u32 = 0x768;
/// 第二个偏移
const OFFSET2: u32 = 0x5560;

/// 植物冷却地址
const PLANT_COOLING_RATE_ADDRESS_OFFSET: u32 = 0x8728C;


/// 获取阳光
pub fn get_sunlight() -> u32 {
    unsafe {
        let address_ptr = BASE_ADDRESS as *const u32;
        if !address_ptr.is_aligned() {
            return 0;
        }
        let value = *address_ptr;
        let address_ptr = (value + OFFSET1) as *const u32;
        if !address_ptr.is_aligned() {
            return 0;
        }
        let value = *address_ptr;
        let address_ptr = (value + OFFSET2) as *const u32;
        if !address_ptr.is_aligned() {
            return 0;
        }
        *address_ptr
    }
}


/// 修改阳光值
pub fn update_sunlight(sunlight: u32) {
    unsafe {
        let address_ptr = BASE_ADDRESS as *const u32;
        let value = *address_ptr;
        let address_ptr = (value + OFFSET1) as *const u32;
        let value = *address_ptr;
        let address_ptr = (value + OFFSET2) as *mut u32;
        address_ptr.write(sunlight);
    }
}
/// 读取植物冷却速度
pub fn read_cooling_accelerate(base_module_address: u32) -> u8 {
    let ptr = (base_module_address + PLANT_COOLING_RATE_ADDRESS_OFFSET) as *const u8;
    unsafe { ptr.offset(3).read() }
}

///
/// 植物冷却加速
pub fn cooling_accelerate(base_module_address: u32, value: u8) -> String {
    let mut flags = PAGE_PROTECTION_FLAGS(0);
    let flags_ptr = addr_of_mut!(flags);
    let ptr = (base_module_address + PLANT_COOLING_RATE_ADDRESS_OFFSET) as *mut u8;
    let offset_ptr = unsafe { ptr.offset(3) };
    let _ = unsafe { VirtualProtect(offset_ptr as _, 1, PAGE_EXECUTE_READWRITE, flags_ptr) };
    unsafe { offset_ptr.write(value) };
    let _ = unsafe { VirtualProtect(offset_ptr as _, 1, flags, flags_ptr) };
    String::from("修改成功")
}

/// 增加僵尸
/// ```8mov edi,dword ptr ds:[006a9f38],
/// mov edi,dword ptr ds:[edi+768],
/// push 1,    // 坐标
/// push 2,    // 类型
/// mov eax,edi,  // 对象
/// call 40ddc0
pub fn increase_zombie() {
    let address = 0x006a9f38u32;
    let offset = 0x768u32;
    let ca = 0x40ddc0u32;
    unsafe {
        asm!(
        "mov edi,dword ptr ds:[{address}]",
        "mov edi,dword ptr ds:[edi+{offset}]",
        "push 1",
        "push 2",
        "mov eax,edi",
        "call {ca}",
        address=in(reg)  address,
        offset=in(reg)  offset,
        ca=in(reg)  ca,
        )
    };
}