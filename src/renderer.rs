extern crate sdl2;

use std::borrow::Borrow;

use sdl2::image::{InitFlag, Sdl2ImageContext};
use sdl2::pixels::Color;
use sdl2::render::{Canvas, TextureCreator };
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

pub struct SdlRenderer {
    sdl_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    image_context: Sdl2ImageContext,
    ttf_context: Sdl2TtfContext,
    //window: Window,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>
}

impl SdlRenderer {
    pub fn init(window_width: u32, window_height: u32) -> Result<Self, String> {
        // open SDL context
        let sdl_ctx = sdl2::init().map_err(|e| e.to_string())?;
        // open video subsystem
        let video_subsys = sdl_ctx.video().map_err(|e| e.to_string())?;
        // open image context
        let image_ctx = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();
        //open font context
        let ttf_ctx = sdl2::ttf::init().map_err(|e| e.to_string())?;
        // create window
        let w = video_subsys.window("rust-sdl2 demo", window_width, window_height)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        // create canvas
        let canv = w.into_canvas()
            .build()
            .expect("failed to build window's canvas");
        
        //create texture create
        let tex_creator = canv.texture_creator();
        Ok(SdlRenderer {
            sdl_context: sdl_ctx,
            video_subsystem: video_subsys,
            image_context: image_ctx,
            ttf_context: ttf_ctx,
            //window: w,
            canvas: canv,
            texture_creator: tex_creator
        })
    }

    pub fn get_sdl_context(&self) -> &sdl2::Sdl {
        &self.sdl_context
    } 

    pub fn get_ttf_context(&self) -> &sdl2::ttf::Sdl2TtfContext {
        &self.ttf_context
    }

    pub fn clear_screen(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
    }

    pub fn refresh(&mut self) {
        self.canvas.present().borrow();
    }

    pub fn draw_text(&mut self, font: &sdl2::ttf::Font<'_, '_>, text: String, x: i32, y:i32) -> Result<(), String> {
        let surface = font
            .render(text.as_str())
            .solid(Color::RGBA(255, 0, 0, 255))
            .map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        let font_texture = self.texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
        
        let size = font.size_of(&text.as_str()).map_err(|e| e.to_string())?;
        let target = Rect::new(x,y, size.0, size.1);
    
        self.canvas.copy(&font_texture, None, Some(target)).map_err(|e| e.to_string());
        Ok(())
    }
}