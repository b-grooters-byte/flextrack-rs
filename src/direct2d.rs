use std::ptr::null;

use windows::{
    core::*,
    Foundation::Numerics::Matrix3x2,
    Win32::Graphics::Direct2D::Common::*,
    Win32::{
        Foundation::GENERIC_READ,
        Graphics::{
            Direct2D::*,
            Imaging::{
                CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICImagingFactory,
                WICBitmapDitherTypeNone, WICBitmapPaletteTypeMedianCut,
                WICDecodeMetadataCacheOnLoad,
            },
        },
        System::Com::{CoCreateInstance, CLSCTX_ALL},
    },
};

/// Creates a single threaded Direct2D factory with default options.
pub fn create_factory() -> Result<ID2D1Factory1> {
    let mut options = D2D1_FACTORY_OPTIONS::default();

    if cfg!(debug_assertions) {
        options.debugLevel = D2D1_DEBUG_LEVEL_INFORMATION;
    }

    unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, Some(&options)) }
}

pub fn create_image_factory() -> Result<IWICImagingFactory> {
    unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_ALL) }
}

/// Create a stroke style with the specified dash pattern
pub fn create_style(factory: &ID2D1Factory1, dashes: Option<&[f32]>) -> Result<ID2D1StrokeStyle1> {
    let mut props = D2D1_STROKE_STYLE_PROPERTIES1 {
        startCap: D2D1_CAP_STYLE_ROUND,
        endCap: D2D1_CAP_STYLE_ROUND,
        ..Default::default()
    };
    if dashes.is_some() {
        props.dashStyle = D2D1_DASH_STYLE_CUSTOM;
    }
    unsafe { factory.CreateStrokeStyle(&props, dashes) }
}

pub fn color_rgb(color: u32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: ((color >> 16) & 0xff) as f32 / 255.0,
        g: ((color >> 8) & 0xff) as f32 / 255.0,
        b: (color & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

pub fn create_brush_rgb(
    target: &ID2D1HwndRenderTarget,
    color: u32,
) -> Result<ID2D1SolidColorBrush> {
    let color = D2D1_COLOR_F {
        r: ((color >> 16) & 0xff) as f32 / 255.0,
        g: ((color >> 8) & 0xff) as f32 / 255.0,
        b: (color & 0xff) as f32 / 255.0,
        a: 1.0,
    };
    unsafe { target.CreateSolidColorBrush(&color, None) }
}

pub fn create_brush_argb(
    target: &ID2D1HwndRenderTarget,
    color: u32,
) -> Result<ID2D1SolidColorBrush> {
    let color = D2D1_COLOR_F {
        r: ((color >> 16) & 0xff) as f32 / 255.0,
        g: ((color >> 8) & 0xff) as f32 / 255.0,
        b: (color & 0xff) as f32 / 255.0,
        a: ((color >> 24) & 0xff) as f32 / 255.0,
    };
    unsafe { target.CreateSolidColorBrush(&color, None) }
}

pub fn load_bitmap(
    filename: &HSTRING,
    target: &ID2D1HwndRenderTarget,
    factory: &IWICImagingFactory,
) -> Result<ID2D1Bitmap> {
    unsafe {
        let decoder = factory.CreateDecoderFromFilename(
            filename,
            Some(null()),
            GENERIC_READ,
            WICDecodeMetadataCacheOnLoad,
        )?;
        let frame = decoder.GetFrame(0)?;
        let converter = factory.CreateFormatConverter()?;
        converter.Initialize(
            &frame,
            &GUID_WICPixelFormat32bppPBGRA,
            WICBitmapDitherTypeNone,
            None,
            0.0,
            WICBitmapPaletteTypeMedianCut,
        )?;
        target.CreateBitmapFromWicBitmap(&converter, None)
    }
}
