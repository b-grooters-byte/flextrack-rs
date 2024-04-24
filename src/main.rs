use std::sync::Once;

use windows::{
    core::{w, Result, HSTRING},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::{
            Direct2D::ID2D1Factory1,
            Gdi::{COLOR_WINDOW, HBRUSH},
        },
        System::{
            Com::{CoInitializeEx, COINIT_MULTITHREADED},
            LibraryLoader::GetModuleHandleW,
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongPtrW,
            LoadCursorW, RegisterClassW, SetWindowLongPtrW, ShowWindow, CREATESTRUCTW, CS_HREDRAW,
            CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, HMENU, IDC_ARROW, MSG, SW_SHOW,
            WINDOW_EX_STYLE, WM_CREATE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

mod bezier;
mod direct2d;
mod flextrack;
mod geometry;
mod layoutview;

static REGISTER_WINDOW_CLASS: Once = Once::new();

fn main() -> windows::core::Result<()> {
    unsafe {
        let result = CoInitializeEx(None, COINIT_MULTITHREADED);
        if result.is_err() {
            return Err(result.into());
        }
    }
    let factory = direct2d::create_factory()?;
    let _m = AppWindow::new("FlexTrack", &factory);
    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

struct AppWindow<'a> {
    handle: HWND,
    layout_view: Option<Box<layoutview::LayoutView<'a>>>,
    factory: &'a ID2D1Factory1,
}

impl<'a> AppWindow<'a> {
    pub(crate) fn new(title: &'static str, factory: &'a ID2D1Factory1) -> Result<Box<Self>> {
        let instance = unsafe { GetModuleHandleW(None)? };
        // synchronization for a one time initialization of FFI call
        REGISTER_WINDOW_CLASS.call_once(|| {
            // use defaults for all other fields
            let class = WNDCLASSW {
                lpfnWndProc: Some(Self::wnd_proc),
                hbrBackground: HBRUSH(COLOR_WINDOW.0 as isize),
                hInstance: instance.into(),
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                lpszClassName: w!("bytetrail.window.minesweeper"),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });
        let mut app_window = Box::new(AppWindow {
            handle: HWND(0),
            layout_view: None,
            factory,
        });
        // create the window using Self reference
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("bytetrail.window.minesweeper"),
                &HSTRING::from(title),
                WS_VISIBLE | WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                400,
                300,
                HWND(0),
                HMENU(0),
                instance,
                Some(app_window.as_mut() as *mut _ as _),
            )
        };
        unsafe { ShowWindow(window, SW_SHOW) };
        Ok(app_window)
    }

    fn message_handler(
        &mut self,
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
        }
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_CREATE {
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            let app_window = (*create_struct).lpCreateParams as *mut AppWindow;
            (*app_window).handle = window;
            SetWindowLongPtrW(window, GWLP_USERDATA, app_window as _);
        }
        let app_window = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut AppWindow;
        if !app_window.is_null() {
            return (*app_window).message_handler(window, message, wparam, lparam);
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}
