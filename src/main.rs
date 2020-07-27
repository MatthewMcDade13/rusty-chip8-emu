mod util;
mod chip8;
extern crate sdl2;
extern crate rand;
extern crate libc;

use chip8::Chip8;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Instant};

// TODO: Make this into program parameter
const TARGET_DELTA: f32 = 1.0/200.0;

pub fn main() -> Result<(), String> {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Rusty Chip8", 1280, 720)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas()
        .accelerated()
        .build()
        .unwrap();
 
    canvas.set_draw_color(Color::BLACK);

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Chip8::new();

    {
        let prog = get_program_name()?;
        if let Err(e) = chip8.load_program(&prog) {
            return Err(format!("Error loading program at path '{}' :: std::io::Error {}", prog, e))
        }
    }

    let texture_creator = canvas.texture_creator();

    let mut chip8_display = texture_creator.create_texture_streaming(PixelFormatEnum::RGB888, Chip8::DISPLAY_W, Chip8::DISPLAY_H).unwrap();

    let mut clock = Instant::now();
    let mut frame_accumulator: f32 = 0.0;
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
        
        frame_accumulator += clock.elapsed().as_secs_f32();
        clock = Instant::now(); 

        if frame_accumulator > TARGET_DELTA {
            frame_accumulator = 0.0;
            if let Ok(true) = chip8.cycle() {
                let format = chip8_display.query().format;
                // update the SDL texture we draw every frame with chip8 gfx buffer            
                let _ = chip8_display.update(None, chip8.render_to_pixels().as_slice(), format.byte_size_of_pixels(Chip8::DISPLAY_W as usize));
            }
        }

        canvas.clear();
        // TODO/NOTE :: Change/play with last parameter of copy (dest) for different display sizes
        canvas.copy(&chip8_display, None, None)?;
        canvas.present();
    }
}

fn get_program_name() -> Result<String, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() > 1 {
        return Err(format!("Multiple arguments not yet implemented. Please only pass the relative path of the chip8 program to be loaded."));
    }

    let name = if args.len() == 1 {
        String::from(&args[0])
    } else {  
        match std::env::current_dir() {
            // TODO/BUG :: cur_dir is current calling directory, so this doesnt work if we put chippin in PATH.
            Ok(cur_dir) => cur_dir.into_os_string().into_string().unwrap() + "/test_opcode.ch8",
            Err(_) => return Err(format!("Error loading default program :: Unable to get current path of executable. Please provide a full path to program to be loaded"))
        }
    };
    Ok(name)
}