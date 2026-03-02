//! TidaLuna-themed progress window for Windows.
//!
//! Shows a dark-themed window with a teal progress bar and status text
//! during the build/install process.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::*;
use winapi::um::winuser::*;

// TidaLuna dark theme colors (teal accent matching the logo)
const BG_COLOR: COLORREF = 0x00221710; // #101722 dark bg
const BAR_BG_COLOR: COLORREF = 0x001a110a; // #0a111a darker
const BAR_FILL_COLOR: COLORREF = 0x00edfe32; // #32feed teal/cyan accent
const TEXT_COLOR: COLORREF = 0x00f5f3f2; // #f2f3f5
const STATUS_TEXT_COLOR: COLORREF = 0x00c1bab5; // #b5bac1

const WINDOW_WIDTH: i32 = 450;
const WINDOW_HEIGHT: i32 = 180;
const BAR_HEIGHT: i32 = 20;
const BAR_MARGIN: i32 = 30;
const BAR_TOP: i32 = 95;

const WM_UPDATE_PROGRESS: u32 = WM_USER + 1;
const WM_CLOSE_PROGRESS: u32 = WM_USER + 2;

struct ProgressState {
    title: String,
    status: String,
    step: u32,
    total_steps: u32,
}

static mut PROGRESS_STATE: Option<Arc<Mutex<ProgressState>>> = None;

/// Wrapper to allow sending HWND across threads.
/// This is safe because we only use PostMessage which is thread-safe.
struct SendHwnd(HWND);
unsafe impl Send for SendHwnd {}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            let mut rect: RECT = unsafe { std::mem::zeroed() };
            unsafe { GetClientRect(hwnd, &mut rect) };

            // Fill background
            let bg_brush = unsafe { CreateSolidBrush(BG_COLOR) };
            unsafe { FillRect(hdc, &rect, bg_brush) };
            unsafe { DeleteObject(bg_brush as *mut _) };

            // Set up text rendering
            unsafe { SetBkMode(hdc, TRANSPARENT as i32) };

            // Create font
            let font = unsafe {
                CreateFontW(
                    20,
                    0,
                    0,
                    0,
                    FW_SEMIBOLD as i32,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET,
                    OUT_DEFAULT_PRECIS,
                    CLIP_DEFAULT_PRECIS,
                    CLEARTYPE_QUALITY,
                    DEFAULT_PITCH | FF_DONTCARE,
                    to_wide("Segoe UI").as_ptr(),
                )
            };
            let old_font = unsafe { SelectObject(hdc, font as *mut _) };

            if let Some(state_arc) = unsafe { &*std::ptr::addr_of!(PROGRESS_STATE) } {
                if let Ok(state) = state_arc.lock() {
                    // Draw title
                    unsafe { SetTextColor(hdc, TEXT_COLOR) };
                    let title_wide = to_wide(&state.title);
                    let mut title_rect = RECT {
                        left: BAR_MARGIN,
                        top: 20,
                        right: rect.right - BAR_MARGIN,
                        bottom: 55,
                    };
                    unsafe {
                        DrawTextW(
                            hdc,
                            title_wide.as_ptr(),
                            -1,
                            &mut title_rect,
                            DT_LEFT | DT_SINGLELINE | DT_NOPREFIX,
                        )
                    };

                    // Draw progress bar background
                    let bar_rect = RECT {
                        left: BAR_MARGIN,
                        top: BAR_TOP,
                        right: rect.right - BAR_MARGIN,
                        bottom: BAR_TOP + BAR_HEIGHT,
                    };
                    let bar_bg_brush = unsafe { CreateSolidBrush(BAR_BG_COLOR) };
                    let bar_bg_rgn = unsafe {
                        CreateRoundRectRgn(
                            bar_rect.left,
                            bar_rect.top,
                            bar_rect.right,
                            bar_rect.bottom,
                            10,
                            10,
                        )
                    };
                    unsafe { FillRgn(hdc, bar_bg_rgn, bar_bg_brush) };
                    unsafe { DeleteObject(bar_bg_rgn as *mut _) };
                    unsafe { DeleteObject(bar_bg_brush as *mut _) };

                    // Draw progress bar fill
                    if state.total_steps > 0 && state.step > 0 {
                        let bar_width = bar_rect.right - bar_rect.left;
                        let fill_width = (bar_width as f64 * state.step as f64
                            / state.total_steps as f64)
                            as i32;
                        if fill_width > 0 {
                            let fill_brush = unsafe { CreateSolidBrush(BAR_FILL_COLOR) };
                            let fill_rgn = unsafe {
                                CreateRoundRectRgn(
                                    bar_rect.left,
                                    bar_rect.top,
                                    bar_rect.left + fill_width,
                                    bar_rect.bottom,
                                    10,
                                    10,
                                )
                            };
                            unsafe { FillRgn(hdc, fill_rgn, fill_brush) };
                            unsafe { DeleteObject(fill_rgn as *mut _) };
                            unsafe { DeleteObject(fill_brush as *mut _) };
                        }
                    }

                    // Draw step counter
                    let step_text = format!("Step {}/{}", state.step, state.total_steps);
                    let step_wide = to_wide(&step_text);
                    unsafe { SetTextColor(hdc, STATUS_TEXT_COLOR) };
                    let small_font = unsafe {
                        CreateFontW(
                            15,
                            0,
                            0,
                            0,
                            FW_NORMAL as i32,
                            0,
                            0,
                            0,
                            DEFAULT_CHARSET,
                            OUT_DEFAULT_PRECIS,
                            CLIP_DEFAULT_PRECIS,
                            CLEARTYPE_QUALITY,
                            DEFAULT_PITCH | FF_DONTCARE,
                            to_wide("Segoe UI").as_ptr(),
                        )
                    };
                    unsafe { SelectObject(hdc, small_font as *mut _) };

                    let mut step_rect = RECT {
                        left: BAR_MARGIN,
                        top: BAR_TOP - 22,
                        right: rect.right - BAR_MARGIN,
                        bottom: BAR_TOP - 2,
                    };
                    unsafe {
                        DrawTextW(
                            hdc,
                            step_wide.as_ptr(),
                            -1,
                            &mut step_rect,
                            DT_RIGHT | DT_SINGLELINE | DT_NOPREFIX,
                        )
                    };

                    // Draw status text
                    let status_wide = to_wide(&state.status);
                    let mut status_rect = RECT {
                        left: BAR_MARGIN,
                        top: BAR_TOP + BAR_HEIGHT + 10,
                        right: rect.right - BAR_MARGIN,
                        bottom: BAR_TOP + BAR_HEIGHT + 35,
                    };
                    unsafe {
                        DrawTextW(
                            hdc,
                            status_wide.as_ptr(),
                            -1,
                            &mut status_rect,
                            DT_LEFT | DT_SINGLELINE | DT_NOPREFIX | DT_END_ELLIPSIS,
                        )
                    };

                    unsafe { DeleteObject(small_font as *mut _) };
                }
            }

            unsafe { SelectObject(hdc, old_font) };
            unsafe { DeleteObject(font as *mut _) };

            unsafe { EndPaint(hwnd, &ps) };
            0
        }
        WM_UPDATE_PROGRESS => {
            unsafe { InvalidateRect(hwnd, std::ptr::null(), TRUE) };
            0
        }
        WM_CLOSE_PROGRESS => {
            unsafe { DestroyWindow(hwnd) };
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// A handle to the progress window, used to send updates from other threads.
pub struct ProgressWindow {
    hwnd: HWND,
    thread: Option<std::thread::JoinHandle<()>>,
    state: Arc<Mutex<ProgressState>>,
}

// HWND is a raw pointer, safe to send across threads for PostMessage
unsafe impl Send for ProgressWindow {}
unsafe impl Sync for ProgressWindow {}

impl ProgressWindow {
    /// Create and show the progress window on a background thread.
    pub fn new(title: &str, total_steps: u32) -> Self {
        let state = Arc::new(Mutex::new(ProgressState {
            title: title.to_string(),
            status: String::new(),
            step: 0,
            total_steps,
        }));

        let state_clone = state.clone();
        let (tx, rx) = mpsc::channel::<SendHwnd>();

        let thread = std::thread::spawn(move || unsafe {
            // Store the state globally for the wndproc
            PROGRESS_STATE = Some(state_clone);

            let class_name = to_wide("TidaLunaProgress");
            let hinstance = GetModuleHandleW(std::ptr::null());

            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: std::ptr::null_mut(),
                hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };

            RegisterClassW(&wc);

            // Center on screen
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let x = (screen_w - WINDOW_WIDTH) / 2;
            let y = (screen_h - WINDOW_HEIGHT) / 2;

            let window_title = to_wide("TidaLuna Launcher");
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST,
                class_name.as_ptr(),
                window_title.as_ptr(),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );

            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            // Send HWND back to the caller
            let _ = tx.send(SendHwnd(hwnd));

            // Message loop
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Clean up
            PROGRESS_STATE = None;
            UnregisterClassW(class_name.as_ptr(), hinstance);
        });

        let SendHwnd(hwnd) = rx
            .recv()
            .expect("Failed to receive HWND from progress window thread");

        ProgressWindow {
            hwnd,
            thread: Some(thread),
            state,
        }
    }

    /// Update the progress bar step and status text.
    pub fn update(&self, step: u32, status: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.step = step;
            state.status = status.to_string();
        }
        unsafe {
            PostMessageW(self.hwnd, WM_UPDATE_PROGRESS, 0, 0);
        }
    }

    /// Close the progress window and wait for the thread to finish.
    pub fn close(mut self) {
        unsafe {
            PostMessageW(self.hwnd, WM_CLOSE_PROGRESS, 0, 0);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ProgressWindow {
    fn drop(&mut self) {
        unsafe {
            // If the window hasn't been closed yet, close it
            if IsWindow(self.hwnd) != 0 {
                PostMessageW(self.hwnd, WM_CLOSE_PROGRESS, 0, 0);
            }
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
