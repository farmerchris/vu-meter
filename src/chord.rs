//! Naming chords from the notes sounding: their pitch classes are matched
//! against chord shapes on every root, preferring the reading whose root is
//! the lowest note (so C E G A is C6, not Am7 over C).

use crate::analyzer::Frame;

const NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Chord shapes as semitones above the root, and how each is written after
/// the root's name. Larger shapes first, so a seventh is not read as the
/// triad it contains (the match is exact anyway).
const SHAPES: [(&[u8], &str); 13] = [
    (&[0, 4, 7, 11], " maj7"),
    (&[0, 4, 7, 10], "7"),
    (&[0, 3, 7, 10], " m7"),
    (&[0, 3, 6, 10], " m7b5"),
    (&[0, 3, 6, 9], " dim7"),
    (&[0, 4, 7, 9], "6"),
    (&[0, 3, 7, 9], " m6"),
    (&[0, 4, 7], " major"),
    (&[0, 3, 7], " minor"),
    (&[0, 3, 6], " dim"),
    (&[0, 4, 8], " aug"),
    (&[0, 2, 7], " sus2"),
    (&[0, 5, 7], " sus4"),
];

/// The chord the MIDI `notes` make, e.g. "C# major", "G7", or "C major/E"
/// when its root is not the lowest note. None for fewer than three distinct
/// pitch classes, or a set that is no known chord.
pub fn name(notes: &[i32]) -> Option<String> {
    let bass = *notes.iter().min()?;
    let mut classes = [false; 12];
    for &n in notes {
        classes[n.rem_euclid(12) as usize] = true;
    }
    let count = classes.iter().filter(|&&c| c).count();
    if count < 3 {
        return None;
    }
    let fits = |root: usize, shape: &[u8]| {
        shape.len() == count && shape.iter().all(|&i| classes[(root + i as usize) % 12])
    };
    // Every reading, the one rooted on the bass first.
    let bass_pc = bass.rem_euclid(12) as usize;
    let roots = std::iter::once(bass_pc).chain((0..12).filter(|&r| r != bass_pc));
    for root in roots {
        if !classes[root] {
            continue;
        }
        if let Some((_, suffix)) = SHAPES.iter().find(|(shape, _)| fits(root, shape)) {
            let slash = if root == bass_pc {
                String::new()
            } else {
                format!("/{}", NAMES[bass_pc])
            };
            return Some(format!("{}{suffix}{slash}", NAMES[root]));
        }
    }
    None
}

/// MIDI notes sounding in `f`: its strongest tone first, then the others.
pub fn frame_notes(f: &Frame) -> Vec<i32> {
    std::iter::once(f.peak_freq.filter(|&(_, db)| db > -100.0))
        .chain(f.other_tones)
        .flatten()
        .map(|(hz, _)| (69.0 + 12.0 * (hz / 440.0).log2()).round() as i32)
        .collect()
}

/// The chord sounding in `f`, if its notes make one.
pub fn of_frame(f: &Frame) -> Option<String> {
    if f.peak_freq.is_none_or(|(_, db)| db <= -100.0) {
        return None;
    }
    name(&frame_notes(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert_eq!(name(&[60, 64, 67]).as_deref(), Some("C major"));
        assert_eq!(name(&[61, 65, 68]).as_deref(), Some("C# major"));
        assert_eq!(name(&[57, 60, 64]).as_deref(), Some("A minor"));
        assert_eq!(name(&[55, 59, 62, 65]).as_deref(), Some("G7"));
        assert_eq!(name(&[60, 64, 67, 71]).as_deref(), Some("C maj7"));
        assert_eq!(name(&[59, 62, 65]).as_deref(), Some("B dim"));
        assert_eq!(name(&[60, 64, 68]).as_deref(), Some("C aug"));
        assert_eq!(name(&[62, 67, 69]).as_deref(), Some("D sus4"));
        // The same notes in any order and octave.
        assert_eq!(name(&[76, 55, 72]).as_deref(), Some("C major/G"));
        // Rooted on the bass when two readings fit: C6, not Am7/C.
        assert_eq!(name(&[60, 64, 67, 69]).as_deref(), Some("C6"));
        assert_eq!(name(&[57, 60, 64, 67]).as_deref(), Some("A m7"));
        // Too few notes, or no chord.
        assert_eq!(name(&[60, 67]), None);
        assert_eq!(name(&[60, 72, 84]), None);
        assert_eq!(name(&[60, 61, 62]), None);
        assert_eq!(name(&[]), None);
    }

    #[test]
    fn the_generators_chords_are_named() {
        use crate::generator::CHORDS;
        let want = [
            "C major", "A minor", "F major", "G7", "C maj7", "D m7", "E major", "B dim",
        ];
        for ((_, notes), want) in CHORDS.iter().zip(want) {
            assert_eq!(name(notes).as_deref(), Some(want));
        }
    }
}
