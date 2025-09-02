pub struct Replay {
    data: Vec<u8>,
    position: usize,
}

impl Replay {
    pub fn new(solution_bytes: Vec<u8>, has_header: bool) -> Self {
        let data = solution_bytes;
        let mut pos = 0;
        if has_header {
            pos = 0x400;
        }
        Replay { data, position: pos }
    }
}

impl Iterator for Replay {
    type Item = (u16, i8, i8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position + 4 > self.data.len() {
            return None;
        }

        let bytes_read = &self.data[self.position..self.position + 4];
        self.position += 4;

        let button = ((bytes_read[0] as u16) << 8) | (bytes_read[1] as u16);
        let stick_x = bytes_read[2] as i8;
        let stick_y = bytes_read[3] as i8;

        // let stick_x = if stick_x >= 128 { stick_x - 256 } else { stick_x };
        // let stick_y = if stick_y >= 128 { stick_y - 256 } else { stick_y };

        Some((button, stick_x, stick_y))
    }
}
