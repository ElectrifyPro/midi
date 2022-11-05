use crate::note::Note;
use midly::{MetaMessage, MidiMessage, TrackEvent, TrackEventKind};
use std::iter::Peekable;

/// Represents a chord, which is a set of notes.
#[derive(Clone, Debug)]
pub struct Chord {
    /// The new BPM when this chord begins. The value is interpreted as "microseconds per beat".
    pub bpm: Option<u32>,

    /// The notes in the chord.
    pub notes: Vec<Note>,

    /// The duration of the chord in MIDI ticks.
    pub duration: u32,
}

impl Chord {
    /// Given an iterator of MIDI events, parse the next chord from the events.
    pub fn next<'a>(iter: &mut Peekable<impl Iterator<Item = &'a TrackEvent<'a>>>) -> Option<Self> {
        let mut chord = Chord {
            bpm: None,
            notes: Vec::new(),
            duration: 0,
        };

        // as we find notes with non-zero velocities, we add them to the chord, and this vec
        // when we see the same note with a zero velocity, we remove it from this vec
        // once the vec is empty, we have found the end of the chord
        let mut notes_on = Vec::new();

        while let Some(e) = iter.next() {
            if let TrackEvent { delta, kind: TrackEventKind::Midi { channel: _, message: MidiMessage::NoteOn { key, vel } } } = e {
                if *vel != 0 {
                    let note = Note::new(key.as_int(), vel.as_int());
                    chord.notes.push(note);
                    notes_on.push(note);
                } else {
                    let i = notes_on.iter().position(|n| n.num == key.as_int()).unwrap();
                    notes_on.remove(i);
                    chord.duration += delta.as_int();
                }

                if notes_on.is_empty() {
                    break;
                }
            }
        }
        
        if chord.notes.is_empty() {
            None
        } else {
            if let Some(next) = iter.peek() {
                chord.duration += next.delta.as_int();
            }
            Some(chord)
        }
    }

    /// Maps the chord to MIDI events.
    pub fn to_events(&self) -> Vec<TrackEvent<'static>> {
        let mut events = Vec::new();

        if let Some(bpm) = self.bpm {
            events.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(bpm.into())),
            });
        }

        for note in &self.notes {
            events.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn {
                        key: note.num.into(),
                        vel: note.vel.into(),
                    },
                },
            });
        }

        for (i, note) in self.notes.iter().enumerate() {
            events.push(TrackEvent {
                delta: if i == 0 { self.duration.into() } else { 0.into() },
                kind: TrackEventKind::Midi {
                    channel: 0.into(),
                    message: MidiMessage::NoteOn {
                        key: note.num.into(),
                        vel: 0.into(),
                    },
                },
            });
        }

        events
    }
}