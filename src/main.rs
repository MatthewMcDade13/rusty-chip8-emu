mod util;
mod chip8;
extern crate sdl2;


use util::Flat2DArray;

use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;


pub fn main() -> Result<(), String> {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .opengl()
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .build()
        .unwrap();
 
    canvas.set_draw_color(Color::BLACK);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = chip8::Chip8::new();
    
    'running: loop {
            
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running Ok(());
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        if let Ok(true) = chip8.cycle() {
            canvas.clear();

            canvas.set_draw_color(Color::RED);
            {
                let (x, y) = canvas.window().size();
                canvas.fill_rect(Rect::new(x as i32 / 2, y as i32 / 2, 50, 50))?;
            }
            canvas.set_draw_color(Color::BLACK);
    
            canvas.present();
        }
       
    }
}

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}
