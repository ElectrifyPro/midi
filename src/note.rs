/// Represents a note.
#[derive(Clone, Copy, Debug)]
pub struct Note {
    /// The note's MIDI note number (0-127).
    pub num: u8,

    /// The note's velocity (0-127).
    pub vel: u8,
}

impl Note {
    /// Create a new note.
    pub fn new(num: u8, vel: u8) -> Self {
        Self { num, vel }
    }
}