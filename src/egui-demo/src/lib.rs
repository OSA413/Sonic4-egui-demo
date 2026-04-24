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
use windows_core::BOOL;
use std::{
    mem::MaybeUninit,
    sync::{LazyLock, Mutex, Once, RwLock},
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

static APP: LazyLock<Mutex<RwLock<MaybeUninit<EguiDx9<i32>>>>> = LazyLock::new(|| Mutex::new(RwLock::new(MaybeUninit::uninit())));
static OLD_WND_PROC: LazyLock<RwLock<WNDPROC>> = LazyLock::new(|| RwLock::new(None));

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
    static INIT: Once = Once::new();
    
    INIT.call_once(|| {
        let window = get_process_window::get_process_window().unwrap();

        unsafe {
            {
                let mut app_writable = APP.lock().unwrap();
                let app_writable = app_writable.get_mut().unwrap();
                app_writable.write(EguiDx9::init(&dev, window, egui_window::ui, 0, true));
            }
            
            {
                let mut old_wnd_proc_writable = OLD_WND_PROC.write().unwrap();
                *old_wnd_proc_writable = std::mem::transmute(SetWindowLongPtrA(
                    window,
                    GWLP_WNDPROC,
                    hk_wnd_proc as *const() as _,
                ));
            }
        }
    });
        
    unsafe {
        APP.try_lock().unwrap().get_mut().unwrap().assume_init_mut().present(&dev);

        PresentHook.call(dev, source_rect, dest_rect, window, rgn_data)
    }
}

fn hk_reset(
    dev: IDirect3DDevice9,
    presentation_parameters: *const D3DPRESENT_PARAMETERS,
) -> HRESULT {
    unsafe {
        APP.lock().unwrap().get_mut().unwrap().assume_init_mut().pre_reset();

        ResetHook.call(dev, presentation_parameters)
    }
}

unsafe extern "stdcall" fn hk_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // This method is spammed like 60 frames per second ...
    unsafe {
        {
            // This part usually doesn't crash
            // This is OLD_WND_PROC that needs special treatment
            APP.lock().unwrap().get_mut().unwrap().assume_init_mut().wnd_proc(msg, wparam, lparam);
        }
        // ... and sometimes, SOMETIMES it pushes two events at the same time making
        // it freeze unless you handle the lock and skip some of the events.
        // The "sometimes" means "after you release the mouse button after clicking or dragging someting".
        // The way to deal with it is to use a structure that allows multiple readers
        // e.g. RwLock
        let result = {
            match OLD_WND_PROC.try_read() {
                Ok(old_wnd_proc) => {
                    CallWindowProcW(old_wnd_proc.clone(), hwnd, msg, wparam, lparam)
                },
                Err(_) => {
                    println!("Event skipped! hwnd={:?}, msg={:?}, wparam={:?}, lparam={:?}", hwnd, msg, wparam, lparam);
                    LRESULT(0)
                }
            }
        };
        
        result
    }
}

unsafe extern "system" fn start_routine(_parameter: *mut std::ffi::c_void) -> u32 {
    std::thread::sleep(Duration::from_secs(5));
    unsafe {
        main_thread(_parameter as usize);
    }
    0
}

unsafe fn main_thread(_hinst: usize) {
    let methods = shroud::directx9::methods().unwrap();

    let reset = methods.device_vmt()[16];
    let present = methods.device_vmt()[17];
    
    unsafe {
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