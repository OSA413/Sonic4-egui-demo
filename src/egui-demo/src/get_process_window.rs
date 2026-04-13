use windows::{
    Win32::{
        Foundation::{
            HWND,
            LPARAM,
            BOOL,
        },
        UI::WindowsAndMessaging::{
            EnumWindows,
            GetWindow,
            GetWindowThreadProcessId,
            IsWindowVisible,
            GW_OWNER,
        },
    },
};

// Taken from https://github.com/ohchase/shroud/tree/copilot/fix-4fe0fc7d-d5ea-4f92-b5a9-792b1abf5541
// Which should by under the MIT license by ohchase
pub fn get_process_window() -> Option<HWND> {
    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, l_param: LPARAM) -> BOOL {
        let mut wnd_proc_id: u32 = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut wnd_proc_id));
            if std::process::id() != wnd_proc_id {
                return true.into();
            }
            // windows has an owner => not a main window
            if GetWindow(hwnd, GW_OWNER).is_ok() {
                return true.into();
            }
            // skip invisible windows
            if !IsWindowVisible(hwnd).as_bool() {
                return true.into();
            }

            *(l_param.0 as *mut HWND) = hwnd;
        }
        false.into()
    }

    let mut output: HWND = HWND::default();
    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            std::mem::transmute::<_, LPARAM>(&mut output as *mut HWND),
        );
    };

    match output.is_invalid() {
        true => None,
        false => Some(output),
    }
}