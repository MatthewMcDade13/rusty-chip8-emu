mod util;
mod chip8;
extern crate sdl2;
extern crate rand;
extern crate libc;

use chip8::Chip8;
use util::Flat2DArray;

use sdl2::rect::Rect;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture};
use std::time::Duration;

pub fn main() -> Result<(), String> {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Rusty Chip8", 800, 600)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas()
        // .present_vsync()
        .accelerated()
        .build()
        .unwrap();
 
    canvas.set_draw_color(Color::BLACK);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Chip8::new();

    let prog = "KeypadTest.ch8";
    if let Err(e) = chip8.load_program(prog) {
        return Err(format!("Error loading program at path {}\nstd::io::Error {}", prog, e))
    }

    let texture_creator = canvas.texture_creator();

    let mut chip8_display = texture_creator.create_texture_streaming(PixelFormatEnum::RGB888, 64, 32).unwrap();


    'running: loop {
            
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running Ok(());
                },
                _ => {}
            }
            chip8.process_input(&event);
        }
        // The rest of the game loop goes here...

        if let Ok(true) = chip8.cycle() {

            let format = chip8_display.query().format;
            // update the SDL texture we draw every frame with chip8 gfx buffer            
            let _ = chip8_display.update(None, chip8.render_to_pixels().as_slice(), format.byte_size_of_pixels(Chip8::DISPLAY_W as usize));
        }
        
        canvas.clear();
        // TODO/NOTE :: Change/play with last parameter of copy (dest) for different display sizes
        canvas.copy(&chip8_display, None, None)?;
        canvas.present();
    }
}
