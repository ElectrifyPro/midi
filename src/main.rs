mod chord;
mod note;

use chord::Chord;
use midly::{MetaMessage, MidiMessage, Timing, TrackEventKind};
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
        if let TrackEventKind::Meta(message) = &smf.tracks[0][i].kind.clone() {
            if let MetaMessage::Tempo(micros_per_beat) = message {
                println!("Micros per beat: {}", micros_per_beat);
                start_micros_per_beat = micros_per_beat.as_int() as f64;
                micros_per_tick = start_micros_per_beat / ticks_per_beat;
                println!("Micros per tick: {}", micros_per_tick);
            }
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