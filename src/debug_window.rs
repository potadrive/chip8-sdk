use minifb::{Window, WindowOptions, Scale, Key};
use crate::chip8::Chip8;

pub struct DebugWindow {
    window: Window,
    bg_color: u32,
}

impl DebugWindow {
    pub fn new(chip8: &Chip8) -> Option<Self> {
        let width = 800;
        let height = 200;
        let bg_color = chip8.display.bg_color;
        match Window::new(
            "Chip8 Debug Info",
            width,
            height,
            WindowOptions {
                resize: true,
                scale: Scale::X1,
                ..WindowOptions::default()
            },
        ) {
            Ok(window) => Some(Self { window, bg_color }),
            Err(e) => {
                eprintln!("Failed to create debug window: {}", e);
                None
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self, chip8: &Chip8) -> bool {
	    if self.window.is_key_down(Key::F1) {
            return true;
        }

        let (width, height) = self.window.get_size();
        
        let buffer = vec![self.bg_color; width * height];
        
        if let Err(e) = self.window.update_with_buffer(&buffer, width, height) {
            eprintln!("Debug window update error: {}", e);
        }
        
        self.window.set_title(&format!(
            "PC={:03X} I={:03X} DT={} ST={} | V0={:02X} V1={:02X} V2={:02X} V3={:02X} V4={:02X} V5={:02X} V6={:02X} V7={:02X} V8={:02X} V9={:02X} VA={:02X} VB={:02X} VC={:02X} VD={:02X} VE={:02X} VF={:02X}",
            chip8.pc, chip8.i, chip8.dt, chip8.st,
            chip8.v[0], chip8.v[1], chip8.v[2], chip8.v[3],
            chip8.v[4], chip8.v[5], chip8.v[6], chip8.v[7],
            chip8.v[8], chip8.v[9], chip8.v[10], chip8.v[11],
            chip8.v[12], chip8.v[13], chip8.v[14], chip8.v[15]
        ));

	false
    }
}
