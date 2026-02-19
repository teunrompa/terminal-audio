use std::str::FromStr;

use ratatui::widgets::{Block, Paragraph, Widget};

macro_rules! notes {
    ($($name:ident),*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Note {
            $($name),*
        }
    }
}

notes!(
    C0, Cs0, D0, Ds0, E0, F0, Fs0, G0, Gs0, A0, As0, B0, C1, Cs1, D1, Ds1, E1, F1, Fs1, G1, Gs1,
    A1, As1, B1, C2, Cs2, D2, Ds2, E2, F2, Fs2, G2, Gs2, A2, As2, B2, C3, Cs3, D3, Ds3, E3, F3,
    Fs3, G3, Gs3, A3, As3, B3, C4, Cs4, D4, Ds4, E4, F4, Fs4, G4, Gs4, A4, As4, B4, C5, Cs5, D5,
    Ds5, E5, F5, Fs5, G5, Gs5, A5, As5, B5, C6, Cs6, D6, Ds6, E6, F6, Fs6, G6, Gs6, A6, As6, B6,
    C7, Cs7, D7, Ds7, E7, F7, Fs7, G7, Gs7, A7, As7, B7, C8
);

impl Note {
    pub fn midi(&self) -> u8 {
        *self as u8 + 12 // MIDI C0 = 12
    }

    pub fn freq(&self) -> f32 {
        let semitones = self.midi() as f32 - 69.0; // A4 = 69
        440.0 * 2f32.powf(semitones / 12.0)
    }
}

pub struct ParseNoteError;

impl Widget for ParseNoteError {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let text = Paragraph::new("Error the input that you provided is not supported");

        text.block(Block::default()).render(area, buf);
    }
}

impl FromStr for Note {
    type Err = ParseNoteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Split into note name + octave, e.g. "C#4" -> ("C#", 4) or "Cs4" -> ("Cs", 4)
        let (name, octave_str) = s
            .find(|c: char| c.is_ascii_digit())
            .map(|i| s.split_at(i))
            .ok_or(ParseNoteError)?;

        let octave: u8 = octave_str.parse().map_err(|_| ParseNoteError)?;

        // Semitone offset within octave
        let semitone: u8 = match name {
            "C" => 0,
            "C#" | "Cs" | "Db" => 1,
            "D" => 2,
            "D#" | "Ds" | "Eb" => 3,
            "E" => 4,
            "F" => 5,
            "F#" | "Fs" | "Gb" => 6,
            "G" => 7,
            "G#" | "Gs" | "Ab" => 8,
            "A" => 9,
            "A#" | "As" | "Bb" => 10,
            "B" => 11,
            _ => return Err(ParseNoteError),
        };

        let idx = octave as usize * 12 + semitone as usize;

        const ALL: &[Note] = &[
            // Octave 0
            Note::C0,
            Note::Cs0,
            Note::D0,
            Note::Ds0,
            Note::E0,
            Note::F0,
            Note::Fs0,
            Note::G0,
            Note::Gs0,
            Note::A0,
            Note::As0,
            Note::B0,
            // Octave 1
            Note::C1,
            Note::Cs1,
            Note::D1,
            Note::Ds1,
            Note::E1,
            Note::F1,
            Note::Fs1,
            Note::G1,
            Note::Gs1,
            Note::A1,
            Note::As1,
            Note::B1,
            // Octave 2
            Note::C2,
            Note::Cs2,
            Note::D2,
            Note::Ds2,
            Note::E2,
            Note::F2,
            Note::Fs2,
            Note::G2,
            Note::Gs2,
            Note::A2,
            Note::As2,
            Note::B2,
            // Octave 3
            Note::C3,
            Note::Cs3,
            Note::D3,
            Note::Ds3,
            Note::E3,
            Note::F3,
            Note::Fs3,
            Note::G3,
            Note::Gs3,
            Note::A3,
            Note::As3,
            Note::B3,
            // Octave 4
            Note::C4,
            Note::Cs4,
            Note::D4,
            Note::Ds4,
            Note::E4,
            Note::F4,
            Note::Fs4,
            Note::G4,
            Note::Gs4,
            Note::A4,
            Note::As4,
            Note::B4,
            // Octave 5
            Note::C5,
            Note::Cs5,
            Note::D5,
            Note::Ds5,
            Note::E5,
            Note::F5,
            Note::Fs5,
            Note::G5,
            Note::Gs5,
            Note::A5,
            Note::As5,
            Note::B5,
            // Octave 6
            Note::C6,
            Note::Cs6,
            Note::D6,
            Note::Ds6,
            Note::E6,
            Note::F6,
            Note::Fs6,
            Note::G6,
            Note::Gs6,
            Note::A6,
            Note::As6,
            Note::B6,
            // Octave 7
            Note::C7,
            Note::Cs7,
            Note::D7,
            Note::Ds7,
            Note::E7,
            Note::F7,
            Note::Fs7,
            Note::G7,
            Note::Gs7,
            Note::A7,
            Note::As7,
            Note::B7,
            // Octave 8
            Note::C8,
        ];

        ALL.get(idx).copied().ok_or(ParseNoteError)
    }
}
