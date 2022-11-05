mod chord;
mod note;

use chord::Chord;
use midly::{MetaMessage, MidiMessage, Timing, TrackEvent, TrackEventKind};
use std::time::Instant;

fn main() {
    let input = "no";
    let bytes = std::fs::read(format!("C:/Users/tam00/OneDrive/Documents/MuseScore3/Scores/{}.mid", input)).unwrap();
    let mut smf = midly::Smf::parse(&bytes).unwrap();

    let ticks_per_beat = if let Timing::Metrical(val) = smf.header.timing {
        val.as_int() as f64
    } else {
        panic!("Not metrical timing");
    };
    let mut start_micros_per_beat = 0f64;
    let mut micros_per_tick = 0f64;
    println!("Ticks per beat: {}", ticks_per_beat);

    let mut chords = {
        let mut vec = Vec::new();
        let mut events = smf.tracks[0].iter().peekable();
        while let Some(chord) = Chord::next(&mut events) {
            vec.push(chord);
        }
        vec
    };
    
    //let mut prev_note = None; // Option<(start, end)> delta tick of previous note (becomes Some(_) after first note)
    let mut i = 0;
    while i < smf.tracks[0].len() {
        match &smf.tracks[0][i].kind.clone() {
            /*TrackEventKind::Midi { channel: _, message } => {
                if let MidiMessage::NoteOn { key: _, vel } = message {
                    // musescore always adds 2 NoteOn events for each note
                    // the first (smf.tracks[0][i]) is the start of the note
                    // the second (smf.tracks[0][i + 1]) is the end of the note ("NoteOff", but it's actually a NoteOn with velocity 0)
                    // the second event indicates the length of the note, so the first event's delta is always close to 0
                    let user_micros_between_notes = if *vel != 0 { // this indicates the start of a note
                        let now = Instant::now();
                        std::io::stdin().read_line(&mut String::new()).unwrap();
                        now.elapsed().as_micros() as f64
                    } else {
                        0.0 // do not use this!
                    };

                    match prev_note {
                        None => { // this is triggered for the first note only
                            prev_note = Some((smf.tracks[0][i].delta.as_int() as f64, smf.tracks[0][i + 1].delta.as_int() as f64));
                            i += 1;
                        }
                        Some(prev) => {
                            let current_note = (smf.tracks[0][i].delta.as_int() as f64, smf.tracks[0][i + 1].delta.as_int() as f64);
                            
                            // time between the start of the previous note and the start of the current note
                            let ticks_between_notes = current_note.0 + prev.0 + prev.1;

                            // microseconds between the notes in the midi file (not the user input)
                            // tks * (beat / tks) * (micros / beat)
                            let file_micros_between_notes = ticks_between_notes / ticks_per_beat * start_micros_per_beat;

                            let frac_of_tempo = file_micros_between_notes / start_micros_per_beat;

                            smf.tracks[0].insert(i - 2, TrackEvent {
                                delta: 0.into(),
                                kind: TrackEventKind::Meta(MetaMessage::Tempo(
                                    ((user_micros_between_notes / frac_of_tempo) as u32).into()
                                ))
                            });

                            i += 2;

                            prev_note = Some(current_note);
                        }
                    }
                }
            }*/
            TrackEventKind::Meta(message) => {
                if let MetaMessage::Tempo(micros_per_beat) = message {
                    println!("Micros per beat: {}", micros_per_beat);
                    start_micros_per_beat = micros_per_beat.as_int() as f64;
                    micros_per_tick = start_micros_per_beat / ticks_per_beat;
                    println!("Micros per tick: {}", micros_per_tick);
                }
            }

            _ => {}
        }
        i += 1;
    }

    let mut prev_chord = None;
    let mut i = 0;
    while i < chords.len() {
        let user_micros_between_chords = {
            let now = Instant::now();
            std::io::stdin().read_line(&mut String::new()).unwrap();
            now.elapsed().as_micros() as f64
        };

        match prev_chord {
            None => prev_chord = Some(chords[i].clone()),
            Some(prev) => {
                let current_chord = chords[i].clone();

                let ticks_between_chords = prev.duration as f64;

                let file_micros_between_chords = ticks_between_chords / ticks_per_beat * start_micros_per_beat;

                let frac_of_tempo = file_micros_between_chords / start_micros_per_beat;

                chords[i - 1].bpm = Some((user_micros_between_chords / frac_of_tempo) as u32);

                prev_chord = Some(current_chord);
            }
        }

        i += 1;
    }

    // find the range of the notes in the midi file
    let info = {
        let start = smf.tracks[0].iter().position(|event| {
            if let TrackEventKind::Midi { channel: _, message } = &event.kind {
                if let MidiMessage::NoteOn { key: _, vel } = message {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }).unwrap();

        (start, smf.tracks[0].len() - 1) // last event is always a MetaMessage::EndOfTrack
    };

    // replace all the notes with our mapped chords
    smf.tracks[0].splice(info.0..info.1, chords.into_iter()
        .map(|c| c.to_events())
        .flatten()
    );

    for e in smf.tracks[0].iter() {
        println!("{:?}", e);
    }

    smf.save(format!("C:/Users/tam00/OneDrive/Documents/MuseScore3/Scores/{}_new.mid", input)).unwrap();
}

// tks / beat = 480
// micros / beat = 500000
// micros / beat * beat / tks = micros / tks