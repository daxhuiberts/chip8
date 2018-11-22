extern crate structopt;
extern crate piston_window;
extern crate rand;

mod chip8;
mod instruction;

use chip8::*;
use piston_window::*;
use structopt::StructOpt;

const PIXEL_SIZE: usize = 10;

#[derive(StructOpt, Debug)]
#[structopt(name = "chip8")]
struct Opt {
    #[structopt(name = "ROM", parse(from_os_str))]
    rom: std::path::PathBuf,
    #[structopt(short = "s", long = "speed", default_value = "1")]
    speed: u8,
}

fn main() {
    let opt = Opt::from_args();
    let bytes = &std::fs::read(opt.rom).unwrap();
    let mut chip8 = Chip8::new(bytes);

    let window_settings = WindowSettings::new("Chip8", [(chip8::WIDTH * PIXEL_SIZE) as u32, (chip8::HEIGHT * PIXEL_SIZE) as u32]);
    let mut window: PistonWindow = window_settings.exit_on_esc(true).build().unwrap();

    let mut event_settings = window.get_event_settings();
    event_settings.set_ups(100);
    event_settings.set_ups_reset(0);
    event_settings.set_max_fps(60);
    window.set_event_settings(event_settings);

    let mut step = false;
    let mut next = false;

    while let Some(event) = window.next() {
        if let Some(_render) = event.render_args() {
            window.draw_2d(&event, |context, graphics| {
                clear([0.0, 0.0, 0.0, 1.0], graphics);

                chip8.get_display().iter().enumerate().for_each(|(index, &bit)| {
                    let (x, y) = (index % chip8::WIDTH as usize, index / chip8::WIDTH as usize);
                    let color = if bit { [1.0, 1.0, 0.0, 1.0] } else { [0.0, 0.0, 0.0, 1.0] };
                    let position = [(x * PIXEL_SIZE as usize) as f64, (y * PIXEL_SIZE as usize) as f64, PIXEL_SIZE as f64, PIXEL_SIZE as f64];
                    rectangle(color, position, context.transform, graphics);
                });
            });
        }

        if let Some(press) = event.press_args() {
            if let Button::Keyboard(key) = press {
                if let Some(index) = match_key(key) {
                    chip8.apply_keypad_value(index, true)
                }

                match key {
                    Key::Space => if step { next = true },
                    _ => (),
                }
            }
        }

        if let Some(release) = event.release_args() {
            if let Button::Keyboard(key) = release {
                if let Some(index) = match_key(key) {
                    chip8.apply_keypad_value(index, false)
                }

                match key {
                    Key::LShift => step = !step,
                    Key::Tab => chip8 = Chip8::new(bytes),
                    _ => (),
                }
            }
        }

        if let Some(_update) = event.update_args() {
            if step {
                if next {
                    let instruction = chip8.tick();
                    println!("{:?}", instruction);
                    next = false;
                }
            } else {
                for _i in 0..(opt.speed * 5) {
                    chip8.tick();
                }
                chip8.decrement_counter();
            }
        }
    }
}

fn match_key(key: Key) -> Option<u8> {
    match key {
        Key::D1 => Some(0x1),
        Key::D2 => Some(0x2),
        Key::D3 => Some(0x3),
        Key::D4 => Some(0xC),
        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),
        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}
