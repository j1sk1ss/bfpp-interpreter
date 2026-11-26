#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TapeMode {
    Fixed,
    Grow,
}

pub(crate) struct Tape {
    cells: Vec<u8>,
    position: usize,
    limit: usize,
}

impl Tape {
    pub(crate) fn new(mode: TapeMode) -> Self {
        let (initial, limit) = match mode {
            TapeMode::Fixed => (30_000, 30_000),
            TapeMode::Grow => (1024, 1024 * 1024),
        };
        Self {
            cells: vec![0; initial],
            position: 0,
            limit,
        }
    }

    pub(crate) fn get(&self) -> u8 {
        self.cells[self.position]
    }

    pub(crate) fn set(&mut self, value: u8) {
        self.cells[self.position] = value;
    }

    pub(crate) fn move_by(&mut self, delta: i32) -> bool {
        let Some(position) = self.position.checked_add_signed(delta as isize) else {
            return false;
        };
        if position >= self.limit {
            return false;
        }
        if position >= self.cells.len() {
            self.cells.resize((position + 1).next_power_of_two(), 0);
        }
        self.position = position;
        true
    }
}
