#[derive(Default)]
pub struct KeyBuffer {
    digits: String,
}

impl KeyBuffer {
    pub fn new() -> Self {
        KeyBuffer::default()
    }

    pub fn push_digit(&mut self, c: char) -> bool {
        if c.is_ascii_digit() {
            self.digits.push(c);
            if self.digits.len() > 3 {
                self.digits.remove(0);
            }
            true
        } else {
            false
        }
    }

    pub fn take_count(&mut self) -> usize {
        let count = self.digits.parse().unwrap_or(0);
        self.digits.clear();
        count
    }

    pub fn clear(&mut self) {
        self.digits.clear()
    }

    pub fn get_count(&mut self) -> Option<usize> {
        self.digits.parse().ok()
    }

    pub fn pending(&self) -> Option<&str> {
        match !self.digits.is_empty() {
            true => Some(&self.digits),
            false => None,
        }
    }
}
