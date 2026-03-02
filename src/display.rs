struct Display {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    bg_color: u32,
    fg_color: u32,
}

impl Display {
    fn new(width: usize, height: usize, bg_color: u32, fg_color: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![bg_color; width * height],
            bg_color,
            fg_color,
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.buffer.fill(self.bg_color);
    }

    fn draw_sprite(&mut self, source: &[u8], x: u8, y: u8, height: u8) -> bool {
        let mut is_erased = false;
        let px_mask = 0b1000_0000;

        for dy in 0..height {
            for dx in 0..8 {
                if source[dy as usize] & (px_mask >> dx) == 0 { continue; }

                let idx = ((y + dy) as usize % self.height) as usize * self.width + ((x + dx) as usize % self.width) as usize;
                self.buffer[idx] = if self.buffer[idx] == self.bg_color {
                    self.fg_color
                } else {
                    is_erased = true;
                    self.bg_color
                }
            }
        }
        is_erased
    }
}