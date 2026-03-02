mod chip8;
mod display;

use std::{env, fs, thread, time::{Duration, Instant}, hint, process};
use minifb::{Window, WindowOptions, Scale, Key};
use chip8::Chip8;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut chip8 = Chip8::new();
    chip8.load_fontset(&Chip8::SYSFONT);
    chip8.load_program(&args[1]);

    let mut window = Window::new(
        &args[1], 
        chip8.display.width, 
        chip8.display.height,
        WindowOptions{
            scale: Scale::X16,
            ..Default::default()
        },
    )
    .unwrap();

    let target_frame_duration = Duration::from_nanos(1_000_000_000 / Chip8::TARGET_FPS as u64); 
    let mut prev_frame_bt = Instant::now();
    const IPF: u16 = Chip8::IPS / 60;

    loop {
        let frame_bt = Instant::now();
        let _dt = frame_bt - prev_frame_bt;
        prev_frame_bt = frame_bt;

        let alignment_et = frame_bt + target_frame_duration;
        let awake_time = alignment_et - Duration::from_millis(1); 

        if !window.is_open() || window.is_key_down(Key::Escape) {
            break;
        }

        chip8.update_timers();
        for _ in 0..IPF {
            for (i, key) in Chip8::KEYBOARD_LAYOUT.iter().enumerate() {
                chip8.keyboard[i] = window.is_key_down(*key);
            }
            chip8.do_instruction_cycle();
        }

        window.update_with_buffer(&chip8.display.buffer, chip8.display.width, chip8.display.height).unwrap();

        let frame_et = Instant::now();
        if frame_et < awake_time {
            thread::sleep(awake_time - frame_et);
        }

        while Instant::now() < alignment_et {
            hint::spin_loop();
        }
    }
}