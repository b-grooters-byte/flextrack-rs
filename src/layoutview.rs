use crate::direct2d::{color_rgb, create_brush_rgb, create_style};
use std::sync::Once;
use windows::{
    core::{Result, HSTRING},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{
            Direct2D::{
                Common::D2D_RECT_F, ID2D1Factory1, ID2D1HwndRenderTarget, ID2D1SolidColorBrush,
                ID2D1StrokeStyle1, D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS,
                D2D1_RENDER_TARGET_PROPERTIES,
            },
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
                DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
            },
            Gdi::{BeginPaint, CreateSolidBrush, EndPaint, PAINTSTRUCT},
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowLongPtrA, LoadCursorW,
            RegisterClassW, SetWindowLongPtrA, CREATESTRUCTA, CS_HREDRAW, CS_VREDRAW,
            CW_USEDEFAULT, GWLP_USERDATA, HMENU, IDC_ARROW, WINDOW_EX_STYLE, WM_CREATE, WM_DESTROY,
            WM_PAINT, WNDCLASSW, WS_CHILDWINDOW, WS_CLIPSIBLINGS, WS_VISIBLE,
        },
    },
};

static REGISTER_WINDOW_CLASS: Once = Once::new();

const DEFAULT_LAYOUT_COLOR: u32 = 0x5acd7d;
const DEFAULT_BRUSH_COLOR: u32 = 0x000000;
pub(crate) struct LayoutView<'a> {
    handle: HWND,
    factory: &'a ID2D1Factory1,
    target: Option<ID2D1HwndRenderTarget>,
    text_format: IDWriteTextFormat,
    line_style: ID2D1StrokeStyle1,
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
                hbrBackground: unsafe { CreateSolidBrush(COLORREF(0x7CD595)) },
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

        // get the parent size
        let mut rect = Default::default();
        unsafe {
            let _ = GetClientRect(parent, &mut rect);
        };

        let _window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                windows::core::w!("bytetrail.window.layoutview"),
                &HSTRING::from(""),
                WS_VISIBLE | WS_CLIPSIBLINGS | WS_CHILDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
                parent,
                HMENU(0),
                instance,
                Some(view.as_mut() as *mut _ as _),
            )
        };
        Ok(view)
    }

    fn release_device(&mut self) {
        self.target = None;
        self.release_device_resources();
    }

    fn release_device_resources(&mut self) {
        self.default_brush = None;
        self.target = None;
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_PAINT => {
                println!("WM_PAINT");
                let mut ps = PAINTSTRUCT::default();
                unsafe {
                    BeginPaint(self.handle, &mut ps);
                    self.render().expect("unable to render");
                    if !bool::from(EndPaint(self.handle, &ps)) {
                        return LRESULT(-1);
                    }
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                self.release_device();
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(HWND(self.handle.0 as _), message, wparam, lparam) },
        }
    }

    fn render(&mut self) -> Result<()> {
        if self.target.is_none() {
            self.create_render_target()?;
            self.create_resources()?;
            println!("Created render target");
        }

        let target = self.target.as_ref().unwrap();

        unsafe {
            target.BeginDraw();
            target.Clear(Some(&color_rgb(DEFAULT_LAYOUT_COLOR)));
            target.DrawRectangle(
                &D2D_RECT_F {
                    left: 10.0,
                    top: 10.0,
                    right: 50.0,
                    bottom: 50.0,
                },
                self.default_brush.as_ref().unwrap(),
                1.0,
                None,
            );
            target.EndDraw(None, None)?;
        }
        Ok(())
    }

    fn create_resources(&mut self) -> Result<()> {
        let target = self.target.as_ref().unwrap();
        self.default_brush = Some(create_brush_rgb(target, DEFAULT_BRUSH_COLOR)?);
        Ok(())
    }

    fn create_render_target(&mut self) -> Result<()> {
        let mut rect = RECT::default();
        unsafe { GetClientRect(self.handle, &mut rect)? };
        let props = D2D1_RENDER_TARGET_PROPERTIES::default();
        let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd: self.handle,
            pixelSize: windows::Win32::Graphics::Direct2D::Common::D2D_SIZE_U {
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            },
            presentOptions: D2D1_PRESENT_OPTIONS::default(),
        };
        let target = unsafe { self.factory.CreateHwndRenderTarget(&props, &hwnd_props)? };
        self.target = Some(target);
        Ok(())
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
