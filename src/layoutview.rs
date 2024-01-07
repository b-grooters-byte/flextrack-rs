use crate::direct2d::create_style;
use std::sync::Once;
use windows::{
    core::{Result, HSTRING},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::{
            Direct2D::{
                ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1SolidColorBrush, ID2D1StrokeStyle,
            },
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
                DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
            },
            Gdi::CreateSolidBrush,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetWindowLongPtrA, LoadCursorW, RegisterClassW,
            SetWindowLongPtrA, CREATESTRUCTA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA,
            HMENU, IDC_ARROW, WINDOW_EX_STYLE, WM_CREATE, WNDCLASSW, WS_CHILDWINDOW,
            WS_CLIPSIBLINGS, WS_VISIBLE,
        },
    },
};

static REGISTER_WINDOW_CLASS: Once = Once::new();

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;

pub(crate) struct LayoutView<'a> {
    handle: HWND,
    factory: &'a ID2D1Factory1,
    target: Option<ID2D1HwndRenderTarget>,
    text_format: IDWriteTextFormat,
    line_style: ID2D1StrokeStyle,
    default_brush: Option<ID2D1SolidColorBrush>,
    dpix: f32,
    dpiy: f32,
}

impl<'a> LayoutView<'a> {
    pub(crate) fn new(parent: HWND, factory: &'a ID2D1Factory1) -> Result<Box<Self>> {
        let instance = unsafe { GetModuleHandleW(None)? };
        let write_factory: IDWriteFactory =
            unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        let line_style = create_style(factory, None)?;
        let text_format = unsafe {
            write_factory.CreateTextFormat(
                &HSTRING::from("San Serif"),
                None,
                DWRITE_FONT_WEIGHT_BOLD,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                14.0,
                &HSTRING::from("en-US"),
            )?
        };

        REGISTER_WINDOW_CLASS.call_once(|| {
            // use defaults for all other fields
            let class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc),
                hInstance: instance.into(),
                hCursor: unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).ok().unwrap() },
                hbrBackground: unsafe { CreateSolidBrush(COLORREF(0)) },
                lpszClassName: windows::core::w!("bytetrail.window.layoutview"),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let mut dpix = 0.0;
        let mut dpiy = 0.0;
        unsafe { factory.GetDesktopDpi(&mut dpix, &mut dpiy) };

        let mut view = Box::new(LayoutView {
            handle: HWND(0),
            factory,
            text_format,
            target: None,
            line_style,
            default_brush: None,
            dpix,
            dpiy,
        });

        let _window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                windows::core::w!("bytetrail.window.layoutview"),
                &HSTRING::from(""),
                WS_VISIBLE | WS_CLIPSIBLINGS | WS_CHILDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                DEFAULT_WIDTH as i32,
                DEFAULT_HEIGHT as i32,
                parent,
                HMENU(0),
                instance,
                Some(view.as_mut() as *mut _ as _),
            )
        };
        Ok(view)
    }

    pub(crate) fn hwnd(&self) -> HWND {
        self.handle
    }

    fn release_device(&mut self) {
        self.target = None;
        self.release_device_resources();
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            _ => unsafe { DefWindowProcW(HWND(self.handle.0 as _), message, wparam, lparam) },
        }
    }

    fn release_device_resources(&mut self) {
        self.default_brush = None;
        self.target = None;
    }
    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_CREATE {
            let create_struct = lparam.0 as *const CREATESTRUCTA;
            let this = (*create_struct).lpCreateParams as *mut Self;
            (*this).handle = window;

            SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
        } else {
            let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;

            if !this.is_null() {
                return (*this).message_handler(message, wparam, lparam);
            }
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}
