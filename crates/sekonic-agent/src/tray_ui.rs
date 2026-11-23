use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};

pub enum UiCommand {
    CheckUpdate { manual: bool },
    PerformUpdate { download_url: Option<String> },
    ToggleAutoUpdate(bool),
    Shutdown,
}

pub struct TraySharedState {
    pub current_version: String,
    pub port: u16,
    pub scanning: AtomicBool,
    pub updating: AtomicBool,
    pub auto_update: AtomicBool,
    pub update_available: AtomicBool,
    pub latest_version: Mutex<Option<String>>,
    pub download_url: Mutex<Option<String>>,
    pub status_line: Mutex<String>,
    pub update_line: Mutex<String>,
    pub pending_balloon: Mutex<Option<(String, String)>>,
    pub hwnd: AtomicIsize,
}

impl TraySharedState {
    pub fn new(port: u16, auto_update: bool) -> Arc<Self> {
        let ver = env!("CARGO_PKG_VERSION").to_string();
        Arc::new(Self {
            current_version: ver.clone(),
            port,
            scanning: AtomicBool::new(false),
            updating: AtomicBool::new(false),
            auto_update: AtomicBool::new(auto_update),
            update_available: AtomicBool::new(false),
            latest_version: Mutex::new(None),
            download_url: Mutex::new(None),
            status_line: Mutex::new(format!("Hazır — ws://127.0.0.1:{port}")),
            update_line: Mutex::new(format!("Sürüm v{ver} — Güncelleme kontrol ediliyor...")),
            pending_balloon: Mutex::new(None),
            hwnd: AtomicIsize::new(0),
        })
    }

    pub fn notify_refresh(&self) {
        #[cfg(windows)]
        {
            let h = self.hwnd.load(Ordering::Acquire);
            if h != 0 {
                unsafe {
                    win32::PostMessageW(h as _, win32::WM_APP_REFRESH, 0, 0);
                }
            }
        }
    }

    pub fn show_balloon(&self, title: &str, body: &str) {
        if let Ok(mut guard) = self.pending_balloon.lock() {
            *guard = Some((title.to_string(), body.to_string()));
        }
        #[cfg(windows)]
        {
            let h = self.hwnd.load(Ordering::Acquire);
            if h != 0 {
                unsafe {
                    win32::PostMessageW(h as _, win32::WM_APP_BALLOON, 0, 0);
                }
            }
        }
    }

    pub fn request_exit(&self) {
        #[cfg(windows)]
        {
            let h = self.hwnd.load(Ordering::Acquire);
            if h != 0 {
                unsafe {
                    win32::PostMessageW(h as _, win32::WM_APP_EXIT, 0, 0);
                }
            }
        }
    }
}

#[cfg(windows)]
pub fn try_focus_existing_instance() -> bool {
    unsafe { win32::try_focus_existing() }
}

#[cfg(not(windows))]
pub fn try_focus_existing_instance() -> bool {
    false
}

#[cfg(windows)]
pub fn run_tray_ui_blocking(
    shared: Arc<TraySharedState>,
    cmd_tx: tokio::sync::mpsc::UnboundedSender<UiCommand>,
) {
    unsafe {
        win32::run_loop(shared, cmd_tx);
    }
}

#[cfg(not(windows))]
pub fn run_tray_ui_blocking(
    _shared: Arc<TraySharedState>,
    _cmd_tx: tokio::sync::mpsc::UnboundedSender<UiCommand>,
) {
    // Non-Windows platforms run headless in terminal
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

#[cfg(windows)]
mod win32 {
    use super::{TraySharedState, UiCommand};
    use crate::updater::WEB_APP_URL;
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};
    use std::sync::atomic::{AtomicIsize, Ordering};
    use std::sync::{Arc, OnceLock};

    pub type HWND = *mut c_void;
    pub type HINSTANCE = *mut c_void;
    pub type HICON = *mut c_void;
    pub type HMENU = *mut c_void;
    pub type HFONT = *mut c_void;
    pub type HBRUSH = *mut c_void;
    pub type HDC = *mut c_void;
    pub type HANDLE = *mut c_void;
    pub type WPARAM = usize;
    pub type LPARAM = isize;
    pub type LRESULT = isize;

    pub const WM_APP: u32 = 0x8000;
    pub const WM_APP_TRAY: u32 = WM_APP + 1;
    pub const WM_APP_REFRESH: u32 = WM_APP + 2;
    pub const WM_APP_BALLOON: u32 = WM_APP + 3;
    pub const WM_APP_SHOW_STATUS: u32 = WM_APP + 4;
    pub const WM_APP_EXIT: u32 = WM_APP + 5;

    const WM_CREATE: u32 = 0x0001;
    const WM_DESTROY: u32 = 0x0002;
    const WM_PAINT: u32 = 0x000F;
    const WM_CLOSE: u32 = 0x0010;
    const WM_COMMAND: u32 = 0x0111;
    const WM_CTLCOLORSTATIC: u32 = 0x0138;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_LBUTTONDBLCLK: u32 = 0x0203;
    const WM_RBUTTONUP: u32 = 0x0205;
    const WM_CONTEXTMENU: u32 = 0x007B;

    const WS_OVERLAPPED: u32 = 0x0000_0000;
    const WS_CAPTION: u32 = 0x00C0_0000;
    const WS_SYSMENU: u32 = 0x0008_0000;
    const WS_MINIMIZEBOX: u32 = 0x0002_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_TABSTOP: u32 = 0x0001_0000;
    const BS_PUSHBUTTON: u32 = 0x0000_0000;
    const BS_DEFPUSHBUTTON: u32 = 0x0000_0001;
    const BS_AUTOCHECKBOX: u32 = 0x0000_0003;
    const SS_LEFT: u32 = 0x0000_0000;

    const SW_HIDE: i32 = 0;
    const SW_SHOWNORMAL: i32 = 1;
    const SW_SHOW: i32 = 5;

    const NIM_ADD: u32 = 0x0000_0000;
    const NIM_MODIFY: u32 = 0x0000_0001;
    const NIM_DELETE: u32 = 0x0000_0002;
    const NIF_MESSAGE: u32 = 0x0000_0001;
    const NIF_ICON: u32 = 0x0000_0002;
    const NIF_TIP: u32 = 0x0000_0004;
    const NIF_INFO: u32 = 0x0000_0010;
    const NIIF_INFO: u32 = 0x0000_0001;

    const MF_STRING: u32 = 0x0000_0000;
    const MF_GRAYED: u32 = 0x0000_0001;
    const MF_CHECKED: u32 = 0x0000_0008;
    const MF_SEPARATOR: u32 = 0x0000_0800;
    const TPM_RIGHTALIGN: u32 = 0x0008;
    const TPM_BOTTOMALIGN: u32 = 0x0020;

    const IDM_SHOW_STATUS: usize = 1001;
    const IDM_OPEN_WEB: usize = 1002;
    const IDM_CHECK_OR_UPDATE: usize = 1003;
    const IDM_TOGGLE_AUTO: usize = 1004;
    const IDM_EXIT: usize = 1005;

    const IDC_LBL_STATUS: isize = 2001;
    const IDC_LBL_UPDATE: isize = 2002;
    const IDC_CHK_AUTO: isize = 2003;
    const IDC_BTN_UPDATE: isize = 2004;
    const IDC_BTN_WEB: isize = 2005;
    const IDC_BTN_HIDE: isize = 2006;
    const IDC_BTN_EXIT: isize = 2007;

    const BM_SETCHECK: u32 = 0x00F1;
    const BM_GETCHECK: u32 = 0x00F0;
    const BST_CHECKED: usize = 1;
    const BST_UNCHECKED: usize = 0;
    const WM_SETFONT: u32 = 0x0030;
    const TRANSPARENT: i32 = 1;

    const CLASS_NAME: &str = "SekonicAgentTrayWndClass";
    const ICON_PNG: &[u8] = include_bytes!("../../../apps/sekonic-bridge/src-tauri/icons/32x32.png");

    #[repr(C)]
    struct WNDCLASSEXW {
        cb_size: u32,
        style: u32,
        lpfn_wnd_proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
        cb_cls_extra: i32,
        cb_wnd_extra: i32,
        h_instance: HINSTANCE,
        h_icon: HICON,
        h_cursor: *mut c_void,
        hbr_background: HBRUSH,
        lpsz_menu_name: *const u16,
        lpsz_class_name: *const u16,
        h_icon_sm: HICON,
    }

    #[repr(C)]
    struct MSG {
        hwnd: HWND,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
        time: u32,
        pt: POINT,
    }

    #[repr(C)]
    #[derive(Default)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Default)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct PAINTSTRUCT {
        hdc: HDC,
        f_erase: i32,
        rc_paint: RECT,
        f_restore: i32,
        f_inc_update: i32,
        rgb_reserved: [u8; 32],
    }

    #[repr(C)]
    struct NOTIFYICONDATAW {
        cb_size: u32,
        h_wnd: HWND,
        u_id: u32,
        u_flags: u32,
        u_callback_message: u32,
        h_icon: HICON,
        sz_tip: [u16; 128],
        dw_state: u32,
        dw_state_mask: u32,
        sz_info: [u16; 256],
        u_timeout_or_version: u32,
        sz_info_title: [u16; 64],
        dw_info_flags: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(lp_module_name: *const u16) -> HINSTANCE;
        fn CreateMutexW(lp_mutex_attributes: *mut c_void, b_initial_owner: i32, lp_name: *const u16) -> HANDLE;
        fn GetLastError() -> u32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
        fn CreateWindowExW(
            dw_ex_style: u32,
            lp_class_name: *const u16,
            lp_window_name: *const u16,
            dw_style: u32,
            x: i32,
            y: i32,
            n_width: i32,
            n_height: i32,
            h_wnd_parent: HWND,
            h_menu: HMENU,
            h_instance: HINSTANCE,
            lp_param: *mut c_void,
        ) -> HWND;
        fn FindWindowW(lp_class_name: *const u16, lp_window_name: *const u16) -> HWND;
        fn DefWindowProcW(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
        fn GetMessageW(lp_msg: *mut MSG, h_wnd: HWND, w_msg_filter_min: u32, w_msg_filter_max: u32) -> i32;
        fn TranslateMessage(lp_msg: *const MSG) -> i32;
        fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
        fn PostQuitMessage(n_exit_code: i32);
        pub fn PostMessageW(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> i32;
        fn SendMessageW(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
        fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> i32;
        fn IsWindowVisible(h_wnd: HWND) -> i32;
        fn SetForegroundWindow(h_wnd: HWND) -> i32;
        fn DestroyWindow(h_wnd: HWND) -> i32;
        fn GetDlgItem(h_dlg: HWND, n_id_dlg_item: i32) -> HWND;
        fn SetWindowTextW(h_wnd: HWND, lp_string: *const u16) -> i32;
        fn EnableWindow(h_wnd: HWND, b_enable: i32) -> i32;
        fn GetSystemMetrics(n_index: i32) -> i32;
        fn LoadIconW(h_instance: HINSTANCE, lp_icon_name: *const u16) -> HICON;
        fn LoadCursorW(h_instance: HINSTANCE, lp_cursor_name: *const u16) -> *mut c_void;
        fn CreateIconFromResourceEx(
            presbits: *const u8,
            dw_res_size: u32,
            f_icon: i32,
            dw_ver: u32,
            cx_desired: i32,
            cy_desired: i32,
            flags: u32,
        ) -> HICON;
        fn CreatePopupMenu() -> HMENU;
        fn AppendMenuW(h_menu: HMENU, u_flags: u32, u_id_new_item: usize, lp_new_item: *const u16) -> i32;
        fn TrackPopupMenu(
            h_menu: HMENU,
            u_flags: u32,
            x: i32,
            y: i32,
            n_reserved: i32,
            h_wnd: HWND,
            prc_rect: *const RECT,
        ) -> i32;
        fn DestroyMenu(h_menu: HMENU) -> i32;
        fn GetCursorPos(lp_point: *mut POINT) -> i32;
        fn BeginPaint(h_wnd: HWND, lp_paint: *mut PAINTSTRUCT) -> HDC;
        fn EndPaint(h_wnd: HWND, lp_paint: *const PAINTSTRUCT) -> i32;
        fn FillRect(h_dc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
        fn DrawTextW(h_dc: HDC, lpch_text: *const u16, cch_text: i32, lprc: *mut RECT, format: u32) -> i32;
        fn InvalidateRect(h_wnd: HWND, lp_rect: *const RECT, b_erase: i32) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateSolidBrush(color: u32) -> HBRUSH;
        fn CreateFontW(
            c_height: i32,
            c_width: i32,
            c_escapement: i32,
            c_orientation: i32,
            c_weight: i32,
            b_italic: u32,
            b_underline: u32,
            b_strike_out: u32,
            i_char_set: u32,
            i_out_precision: u32,
            i_clip_precision: u32,
            i_quality: u32,
            i_pitch_and_family: u32,
            psz_face_name: *const u16,
        ) -> HFONT;
        fn SelectObject(hdc: HDC, h: *mut c_void) -> *mut c_void;
        fn SetBkMode(hdc: HDC, mode: i32) -> i32;
        fn SetTextColor(hdc: HDC, color: u32) -> u32;
    }

    #[link(name = "shell32")]
    extern "system" {
        fn Shell_NotifyIconW(dw_message: u32, lp_data: *const NOTIFYICONDATAW) -> i32;
        fn ShellExecuteW(
            hwnd: HWND,
            lp_operation: *const u16,
            lp_file: *const u16,
            lp_parameters: *const u16,
            lp_directory: *const u16,
            n_show_cmd: i32,
        ) -> HINSTANCE;
    }

    struct UiCtx {
        shared: Arc<TraySharedState>,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<UiCommand>,
        h_icon: HICON,
        h_font: HFONT,
        h_font_bold: HFONT,
        h_bg_brush: HBRUSH,
        h_header_brush: HBRUSH,
    }

    unsafe impl Send for UiCtx {}
    unsafe impl Sync for UiCtx {}

    static UI_CTX: OnceLock<UiCtx> = OnceLock::new();
    static SINGLE_INSTANCE_MUTEX: AtomicIsize = AtomicIsize::new(0);

    fn to_w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn copy_w_slice(dst: &mut [u16], s: &str) {
        dst.fill(0);
        for (i, ch) in s.encode_utf16().take(dst.len().saturating_sub(1)).enumerate() {
            dst[i] = ch;
        }
    }

    pub unsafe fn try_focus_existing() -> bool {
        let mutex_name = to_w("Local\\SekonicAgentSingleInstance");
        let handle = CreateMutexW(null_mut(), 0, mutex_name.as_ptr());
        const ERROR_ALREADY_EXISTS: u32 = 183;
        if !handle.is_null() && GetLastError() == ERROR_ALREADY_EXISTS {
            let cls = to_w(CLASS_NAME);
            let existing = FindWindowW(cls.as_ptr(), null());
            if !existing.is_null() {
                PostMessageW(existing, WM_APP_SHOW_STATUS, 0, 0);
            }
            return true;
        }
        SINGLE_INSTANCE_MUTEX.store(handle as isize, Ordering::Release);
        false
    }

    unsafe fn load_app_icon() -> HICON {
        let icon = CreateIconFromResourceEx(
            ICON_PNG.as_ptr(),
            ICON_PNG.len() as u32,
            1,
            0x0003_0000,
            32,
            32,
            0,
        );
        if !icon.is_null() {
            return icon;
        }
        LoadIconW(null_mut(), 32512 as *const u16) // IDI_APPLICATION
    }

    unsafe fn update_tray_icon(hwnd: HWND, cmd: u32, balloon: Option<(&str, &str)>) {
        let Some(ctx) = UI_CTX.get() else {
            return;
        };
        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.h_wnd = hwnd;
        nid.u_id = 1;
        nid.u_callback_message = WM_APP_TRAY;
        nid.h_icon = ctx.h_icon;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;

        let tip = if ctx.shared.updating.load(Ordering::Relaxed) {
            format!("Sekonic Ajan v{} — Güncelleniyor...", ctx.shared.current_version)
        } else if ctx.shared.scanning.load(Ordering::Relaxed) {
            format!("Sekonic Ajan v{} — Tarama Aktif", ctx.shared.current_version)
        } else if ctx.shared.update_available.load(Ordering::Relaxed) {
            let latest = ctx.shared.latest_version.lock().ok().and_then(|g| g.clone()).unwrap_or_default();
            format!("Sekonic Ajan v{} — Güncelleme Mevcut (v{latest})", ctx.shared.current_version)
        } else {
            format!("Sekonic Ajan v{} — Hazır (:{} )", ctx.shared.current_version, ctx.shared.port)
        };
        copy_w_slice(&mut nid.sz_tip, &tip);

        if let Some((title, body)) = balloon {
            nid.u_flags |= NIF_INFO;
            nid.dw_info_flags = NIIF_INFO;
            copy_w_slice(&mut nid.sz_info_title, title);
            copy_w_slice(&mut nid.sz_info, body);
        }

        Shell_NotifyIconW(cmd, &nid);
    }

    unsafe fn refresh_controls(hwnd: HWND) {
        let Some(ctx) = UI_CTX.get() else {
            return;
        };
        if let Ok(st) = ctx.shared.status_line.lock() {
            let lbl = GetDlgItem(hwnd, IDC_LBL_STATUS as i32);
            if !lbl.is_null() {
                let w = to_w(&st);
                SetWindowTextW(lbl, w.as_ptr());
            }
        }
        if let Ok(up) = ctx.shared.update_line.lock() {
            let lbl = GetDlgItem(hwnd, IDC_LBL_UPDATE as i32);
            if !lbl.is_null() {
                let w = to_w(&up);
                SetWindowTextW(lbl, w.as_ptr());
            }
        }
        let chk = GetDlgItem(hwnd, IDC_CHK_AUTO as i32);
        if !chk.is_null() {
            let checked = if ctx.shared.auto_update.load(Ordering::Relaxed) {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            };
            SendMessageW(chk, BM_SETCHECK, checked, 0);
        }
        let btn_up = GetDlgItem(hwnd, IDC_BTN_UPDATE as i32);
        if !btn_up.is_null() {
            let updating = ctx.shared.updating.load(Ordering::Relaxed);
            EnableWindow(btn_up, if updating { 0 } else { 1 });
            let text = if updating {
                "Güncelleniyor..."
            } else if ctx.shared.update_available.load(Ordering::Relaxed) {
                "Ajanı Güncelle"
            } else {
                "Güncelleme Kontrol Et"
            };
            let w = to_w(text);
            SetWindowTextW(btn_up, w.as_ptr());
        }
        update_tray_icon(hwnd, NIM_MODIFY, None);
        InvalidateRect(hwnd, null(), 0);
    }

    unsafe fn show_context_menu(hwnd: HWND) {
        let Some(ctx) = UI_CTX.get() else {
            return;
        };
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return;
        }

        let header = if ctx.shared.updating.load(Ordering::Relaxed) {
            format!("Sekonic Ajan v{} — Güncelleniyor", ctx.shared.current_version)
        } else if ctx.shared.scanning.load(Ordering::Relaxed) {
            format!("Sekonic Ajan v{} — Tarama Aktif", ctx.shared.current_version)
        } else {
            format!("Sekonic Ajan v{} — Hazır (:{} )", ctx.shared.current_version, ctx.shared.port)
        };
        AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, to_w(&header).as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, IDM_SHOW_STATUS, to_w("Durum Penceresini Göster").as_ptr());
        AppendMenuW(menu, MF_STRING, IDM_OPEN_WEB, to_w("Web Arayüzünü Aç (sekonic.dijios.com)").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());

        if ctx.shared.update_available.load(Ordering::Relaxed) {
            let latest = ctx.shared.latest_version.lock().ok().and_then(|g| g.clone()).unwrap_or_default();
            let label = format!("Ajanı Güncelle (v{} → v{latest})", ctx.shared.current_version);
            AppendMenuW(menu, MF_STRING, IDM_CHECK_OR_UPDATE, to_w(&label).as_ptr());
        } else {
            AppendMenuW(menu, MF_STRING, IDM_CHECK_OR_UPDATE, to_w("Güncellemeleri Kontrol Et").as_ptr());
        }

        let auto_flags = if ctx.shared.auto_update.load(Ordering::Relaxed) {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING
        };
        AppendMenuW(menu, auto_flags, IDM_TOGGLE_AUTO, to_w("Otomatik Güncelleme").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, IDM_EXIT, to_w("Çıkış").as_ptr());

        let mut pt = POINT::default();
        GetCursorPos(&raw mut pt);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_RIGHTALIGN | TPM_BOTTOMALIGN, pt.x, pt.y, 0, hwnd, null());
        DestroyMenu(menu);
    }

    unsafe fn open_web_app() {
        let open_w = to_w("open");
        let url_w = to_w(WEB_APP_URL);
        ShellExecuteW(null_mut(), open_w.as_ptr(), url_w.as_ptr(), null(), null(), SW_SHOWNORMAL);
    }

    unsafe fn trigger_update_or_check(ctx: &UiCtx) {
        if ctx.shared.updating.load(Ordering::Relaxed) {
            return;
        }
        if ctx.shared.update_available.load(Ordering::Relaxed) {
            let dl = ctx.shared.download_url.lock().ok().and_then(|g| g.clone());
            let _ = ctx.cmd_tx.send(UiCommand::PerformUpdate { download_url: dl });
        } else {
            let _ = ctx.cmd_tx.send(UiCommand::CheckUpdate { manual: true });
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                if let Some(ctx) = UI_CTX.get() {
                    let h_inst = GetModuleHandleW(null());
                    let static_cls = to_w("STATIC");
                    let btn_cls = to_w("BUTTON");

                    let mk_ctrl = |cls: &[u16], text: &str, style: u32, x: i32, y: i32, w: i32, h: i32, id: isize, bold: bool| {
                        let tw = to_w(text);
                        let ctrl = CreateWindowExW(
                            0,
                            cls.as_ptr(),
                            tw.as_ptr(),
                            WS_CHILD | WS_VISIBLE | style,
                            x,
                            y,
                            w,
                            h,
                            hwnd,
                            id as HMENU,
                            h_inst,
                            null_mut(),
                        );
                        if !ctrl.is_null() {
                            SendMessageW(
                                ctrl,
                                WM_SETFONT,
                                if bold { ctx.h_font_bold } else { ctx.h_font } as usize,
                                1,
                            );
                        }
                    };

                    mk_ctrl(&static_cls, "Durum: Hazır", SS_LEFT, 22, 82, 390, 24, IDC_LBL_STATUS, true);
                    mk_ctrl(&static_cls, "Güncelleme durumu kontrol ediliyor...", SS_LEFT, 22, 112, 390, 24, IDC_LBL_UPDATE, false);
                    mk_ctrl(&btn_cls, "Güncellemeleri otomatik kontrol et ve kur", BS_AUTOCHECKBOX | WS_TABSTOP, 22, 150, 360, 24, IDC_CHK_AUTO, false);

                    mk_ctrl(&btn_cls, "Güncelleme Kontrol Et", BS_DEFPUSHBUTTON | WS_TABSTOP, 22, 196, 185, 34, IDC_BTN_UPDATE, true);
                    mk_ctrl(&btn_cls, "Web Arayüzünü Aç", BS_PUSHBUTTON | WS_TABSTOP, 220, 196, 185, 34, IDC_BTN_WEB, false);
                    mk_ctrl(&btn_cls, "Tepsiye Küçült", BS_PUSHBUTTON | WS_TABSTOP, 22, 242, 185, 32, IDC_BTN_HIDE, false);
                    mk_ctrl(&btn_cls, "Ajanı Kapat", BS_PUSHBUTTON | WS_TABSTOP, 220, 242, 185, 32, IDC_BTN_EXIT, false);

                    update_tray_icon(
                        hwnd,
                        NIM_ADD,
                        Some((
                            &format!("Sekonic Yerel Ajan v{}", ctx.shared.current_version),
                            "Sistem tepsisinde hazır. Web arayüzünden otomatik bağlantı bekleniyor.",
                        )),
                    );
                    refresh_controls(hwnd);
                }
                0
            }
            WM_PAINT => {
                if let Some(ctx) = UI_CTX.get() {
                    let mut ps: PAINTSTRUCT = std::mem::zeroed();
                    let hdc = BeginPaint(hwnd, &raw mut ps);
                    let hdr = RECT {
                        left: 0,
                        top: 0,
                        right: 440,
                        bottom: 64,
                    };
                    FillRect(hdc, &raw const hdr, ctx.h_header_brush);
                    SetBkMode(hdc, TRANSPARENT);
                    SetTextColor(hdc, 0x00FF_FFFF);
                    SelectObject(hdc, ctx.h_font_bold as _);
                    let title = format!("  Sekonic Yerel Ajan   •   v{}", ctx.shared.current_version);
                    let mut tw = to_w(&title);
                    let mut tr = RECT {
                        left: 16,
                        top: 18,
                        right: 420,
                        bottom: 50,
                    };
                    DrawTextW(hdc, tw.as_mut_ptr(), -1, &raw mut tr, 0x0020);
                    EndPaint(hwnd, &raw const ps);
                }
                0
            }
            WM_CTLCOLORSTATIC => {
                if let Some(ctx) = UI_CTX.get() {
                    SetBkMode(w_param as HDC, TRANSPARENT);
                    SetTextColor(w_param as HDC, 0x001E_293B);
                    return ctx.h_bg_brush as isize;
                }
                DefWindowProcW(hwnd, msg, w_param, l_param)
            }
            WM_APP_TRAY => {
                let ev = (l_param as u32) & 0xFFFF;
                if ev == WM_RBUTTONUP || ev == WM_CONTEXTMENU {
                    show_context_menu(hwnd);
                } else if ev == WM_LBUTTONUP || ev == WM_LBUTTONDBLCLK {
                    if IsWindowVisible(hwnd) != 0 {
                        ShowWindow(hwnd, SW_HIDE);
                    } else {
                        ShowWindow(hwnd, SW_SHOW);
                        SetForegroundWindow(hwnd);
                    }
                }
                0
            }
            WM_APP_SHOW_STATUS => {
                ShowWindow(hwnd, SW_SHOW);
                SetForegroundWindow(hwnd);
                0
            }
            WM_APP_REFRESH => {
                refresh_controls(hwnd);
                0
            }
            WM_APP_BALLOON => {
                if let Some(ctx) = UI_CTX.get() {
                    let msg_opt = ctx.shared.pending_balloon.lock().ok().and_then(|mut g| g.take());
                    if let Some((t, b)) = msg_opt {
                        update_tray_icon(hwnd, NIM_MODIFY, Some((&t, &b)));
                    }
                }
                0
            }
            WM_APP_EXIT => {
                DestroyWindow(hwnd);
                0
            }
            WM_COMMAND => {
                let id = w_param & 0xFFFF;
                if let Some(ctx) = UI_CTX.get() {
                    match id {
                        IDM_SHOW_STATUS => {
                            ShowWindow(hwnd, SW_SHOW);
                            SetForegroundWindow(hwnd);
                        }
                        IDM_OPEN_WEB | 2005 => {
                            open_web_app();
                        }
                        IDM_CHECK_OR_UPDATE | 2004 => {
                            trigger_update_or_check(ctx);
                        }
                        IDM_TOGGLE_AUTO => {
                            let next = !ctx.shared.auto_update.load(Ordering::Relaxed);
                            ctx.shared.auto_update.store(next, Ordering::Relaxed);
                            let _ = ctx.cmd_tx.send(UiCommand::ToggleAutoUpdate(next));
                            refresh_controls(hwnd);
                        }
                        2003 => {
                            let chk = GetDlgItem(hwnd, IDC_CHK_AUTO as i32);
                            let is_checked = SendMessageW(chk, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                            ctx.shared.auto_update.store(is_checked, Ordering::Relaxed);
                            let _ = ctx.cmd_tx.send(UiCommand::ToggleAutoUpdate(is_checked));
                        }
                        2006 => {
                            ShowWindow(hwnd, SW_HIDE);
                        }
                        IDM_EXIT | 2007 => {
                            let _ = ctx.cmd_tx.send(UiCommand::Shutdown);
                            DestroyWindow(hwnd);
                        }
                        _ => {}
                    }
                }
                0
            }
            WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                0
            }
            WM_DESTROY => {
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.h_wnd = hwnd;
                nid.u_id = 1;
                Shell_NotifyIconW(NIM_DELETE, &nid);
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }

    pub unsafe fn run_loop(
        shared: Arc<TraySharedState>,
        cmd_tx: tokio::sync::mpsc::UnboundedSender<UiCommand>,
    ) {
        let h_inst = GetModuleHandleW(null());
        let h_icon = load_app_icon();
        let font_face = to_w("Segoe UI");
        let h_font = CreateFontW(-14, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, font_face.as_ptr());
        let h_font_bold = CreateFontW(-15, 0, 0, 0, 600, 0, 0, 0, 1, 0, 0, 5, 0, font_face.as_ptr());
        let h_bg_brush = CreateSolidBrush(0x00FC_FAF8); // #F8FAFC in BGR
        let h_header_brush = CreateSolidBrush(0x002A_170F); // #0F172A in BGR

        let _ = UI_CTX.set(UiCtx {
            shared: shared.clone(),
            cmd_tx,
            h_icon,
            h_font,
            h_font_bold,
            h_bg_brush,
            h_header_brush,
        });

        let cls_w = to_w(CLASS_NAME);
        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wnd_proc: wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: h_inst,
            h_icon,
            h_cursor: LoadCursorW(null_mut(), 32512 as *const u16),
            hbr_background: h_bg_brush,
            lpsz_menu_name: null(),
            lpsz_class_name: cls_w.as_ptr(),
            h_icon_sm: h_icon,
        };
        RegisterClassExW(&raw const wc);

        let win_w = 440;
        let win_h = 330;
        let screen_w = GetSystemMetrics(0);
        let screen_h = GetSystemMetrics(1);
        let x = (screen_w - win_w) / 2;
        let y = (screen_h - win_h) / 2;

        let title_w = to_w("Sekonic Yerel Ajan");
        let hwnd = CreateWindowExW(
            0,
            cls_w.as_ptr(),
            title_w.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            x,
            y,
            win_w,
            win_h,
            null_mut(),
            null_mut(),
            h_inst,
            null_mut(),
        );

        if hwnd.is_null() {
            return;
        }
        shared.hwnd.store(hwnd as isize, Ordering::Release);

        // Start minimized in System Tray (balloon notification informs user)
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&raw mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&raw const msg);
            DispatchMessageW(&raw const msg);
        }
    }
}
