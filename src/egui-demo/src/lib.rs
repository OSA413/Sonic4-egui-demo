/* This code contains portions from
 * https://github.com/unknowntrojan/egui-d3d9/blob/master/example/src/lib.rs
 * https://github.com/ohchase/shroud/blob/master/examples/shroud-debug/src/lib.rs
 
    That are made by ohchase and unknowntrojan under the MIT license

    As far as I know othervise it doesn't work
*/
mod egui_window;
mod get_process_window;

use windows::{
    core::HRESULT,
    Win32::{
        System::SystemServices::{
            DLL_PROCESS_ATTACH,
            DLL_PROCESS_DETACH,
        },
        Foundation::{
            HMODULE,
            HWND,
            LPARAM,
            BOOL,
            LRESULT,
            RECT,
            WPARAM
        },
        Graphics::{
            Direct3D9::{
                IDirect3DDevice9,
                D3DPRESENT_PARAMETERS,
            },
            Gdi::RGNDATA,
        },
        UI::WindowsAndMessaging::{
            CallWindowProcW,
            SetWindowLongPtrA,
            GWLP_WNDPROC,
            WNDPROC,
        },
    },
};
use retour::static_detour;
use egui_d3d9::EguiDx9;
use std::{
    sync::Once,
    time::Duration,
};

#[unsafe(no_mangle)]
unsafe extern "C" fn Init() {
    println!("Hello world!");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn PostInit() {
    println!("PostInit called");
}

static mut APP: Option<EguiDx9<i32>> = None;
static mut OLD_WND_PROC: Option<WNDPROC> = None;

static_detour! {
    static PresentHook: unsafe extern "stdcall" fn(
        IDirect3DDevice9,
        *const RECT,
        *const RECT,
        HWND,
        *const RGNDATA
    ) -> HRESULT;

    static ResetHook: unsafe extern "stdcall" fn(
        IDirect3DDevice9,
        *const D3DPRESENT_PARAMETERS
    ) -> HRESULT;
}

type FnPresent = unsafe extern "stdcall" fn(
    IDirect3DDevice9,
    *const RECT,
    *const RECT,
    HWND,
    *const RGNDATA,
) -> HRESULT;

type FnReset = unsafe extern "stdcall" fn(
    IDirect3DDevice9,
    *const D3DPRESENT_PARAMETERS
) -> HRESULT;

fn hk_present(
    dev: IDirect3DDevice9,
    source_rect: *const RECT,
    dest_rect: *const RECT,
    window: HWND,
    rgn_data: *const RGNDATA,
) -> HRESULT {
    unsafe {
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            let window = get_process_window::get_process_window().unwrap();

            APP = Some(EguiDx9::init(&dev, window, egui_window::ui, 0, true));

            OLD_WND_PROC = Some(std::mem::transmute(SetWindowLongPtrA(
                window,
                GWLP_WNDPROC,
                hk_wnd_proc as usize as _,
            )));
        });

        APP.as_mut().unwrap().present(&dev);

        PresentHook.call(dev, source_rect, dest_rect, window, rgn_data)
    }
}

fn hk_reset(
    dev: IDirect3DDevice9,
    presentation_parameters: *const D3DPRESENT_PARAMETERS,
) -> HRESULT {
    unsafe {
        APP.as_mut().unwrap().pre_reset();

        ResetHook.call(dev, presentation_parameters)
    }
}

unsafe extern "stdcall" fn hk_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    APP.as_mut().unwrap().wnd_proc(msg, wparam, lparam);
    CallWindowProcW(OLD_WND_PROC.unwrap(), hwnd, msg, wparam, lparam)
}

unsafe extern "system" fn start_routine(_parameter: *mut std::ffi::c_void) -> u32 {
    std::thread::sleep(Duration::from_secs(5));
    main_thread(_parameter as usize);
    0
}

unsafe fn main_thread(_hinst: usize) {
    let methods = shroud::directx9::methods().unwrap();

    let reset = methods.device_vmt()[16];
    let present = methods.device_vmt()[17];

    let present: FnPresent = std::mem::transmute(present);
    let reset: FnReset = std::mem::transmute(reset);

    PresentHook
        .initialize(present, hk_present)
        .unwrap()
        .enable()
        .unwrap();

    ResetHook
        .initialize(reset, hk_reset)
        .unwrap()
        .enable()
        .unwrap();
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(dll_module: HMODULE, reason: u32, _reserved: usize) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            let thread = unsafe {
                windows::Win32::System::Threading::CreateThread(
                    None,
                    0,
                    Some(start_routine),
                    Some(dll_module.0 as *const std::ffi::c_void),
                    windows::Win32::System::Threading::THREAD_CREATION_FLAGS(0),
                    None,
                )
            };

            match thread {
                Ok(_handle) => {
                    println!("Created thread")
                }
                Err(e) => {
                    panic!("Unable to create thread {e:?}")
                }
            }
        },
        DLL_PROCESS_DETACH => {
        },
        _ => {},
    };
    return BOOL::from(true);
}