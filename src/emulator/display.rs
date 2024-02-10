use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use crate::emulator::state::SystemState;
use crate::emulator::colors::ntsc_to_rgb;

use super::memory::DeviceMemory;

pub struct Renderer {
    pub state: Arc<Mutex<SystemState>>
}

impl Renderer{
    pub fn start(&self) -> Result<(), String> {
        // let sdl_context = sdl2::init()?;
        // let video_subsystem = sdl_context.video()?;
    
        // let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        //     .position_centered()
        //     .build()
        //     .expect("could not initialize video subsystem");
    
        // let mut canvas = window.into_canvas().build()
        //     .expect("could not make a canvas");
    
        // canvas.set_draw_color(Color::RGB(0, 255, 255));
        // canvas.clear();
        // canvas.present();
        // let mut event_pump = sdl_context.event_pump()?;
        // let mut i = 0;
        // 'running: loop {
        //     let background_color = ntsc_to_rgb(self.state.lock().unwrap().fetch_memory(u8::try_from(DeviceMemory::COLUBK).unwrap().into()).unwrap());

        //     canvas.set_draw_color(Color::RGB(background_color.0, background_color.1, background_color.2));
        //     canvas.clear();
        //     for event in event_pump.poll_iter() {
        //         match event {
        //             Event::Quit {..} |
        //             Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
        //                 break 'running;
        //             },
        //             _ => {}
        //         }
        //     }
        //     // The rest of the game loop goes here...
    
        //     canvas.present();
        //     ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        // }
        Ok(())
    }
}