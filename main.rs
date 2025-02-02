use midly::num::{u4, u7, u15, u28};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};
use std::str::FromStr;

// 音乐元素数据结构
#[derive(Debug, PartialEq)]
pub enum MusicElement {
    Note(Note),
    Rest(Rest),
    Chord(Chord),
}

#[derive(Debug, PartialEq)]
pub struct Note {
    pub pitch: Pitch,
    pub duration: f32,
    pub velocity: u8,
}

#[derive(Debug, PartialEq)]
pub struct Rest {
    pub duration: f32,
}

#[derive(Debug, PartialEq)]
pub struct Chord {
    pub notes: Vec<Pitch>,
    pub duration: f32,
    pub velocity: u8,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Pitch {
    pub note: NoteName,
    pub octave: i8,
pub accidental: Accidental,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NoteName {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
}

// 解析器实现
pub fn parse_music(input: &str) -> Result<Vec<MusicElement>, String> {
    let (remaining, elements) = parse_elements(input)
        .map_err(|e| format!("Parse error: {:?}", e))?;
    if !remaining.is_empty() {
        return Err(format!("Remaining unparsed input: {}", remaining));
    }
    Ok(elements)
}

fn parse_elements(input: &str) -> IResult<&str, Vec<MusicElement>> {
    separated_list1(multispace0, alt((parse_chord, parse_note, parse_rest)))(input)
}

fn parse_note(input: &str) -> IResult<&str, MusicElement> {
    map(
        separated_pair(
            parse_pitch,
            tag(":"),
            separated_pair(parse_duration, opt(tag("@")), opt(parse_velocity)),
        ),
        |(pitch, (duration, velocity))| {
            MusicElement::Note(Note {
                pitch,
                duration,
                velocity: velocity.unwrap_or(127),
            })
        },
    )(input)
}

fn parse_rest(input: &str) -> IResult<&str, MusicElement> {
    map(
        preceded(tag("R:"), parse_duration),
        |duration| MusicElement::Rest(Rest { duration }),
    )(input)
}

fn parse_chord(input: &str) -> IResult<&str, MusicElement> {
    map(
        separated_pair(
            delimited(
                tag("("),
                separated_list1(multispace0, parse_pitch),
                tag(")"),
            ),
            tag(":"),
            separated_pair(parse_duration, opt(tag("@")), opt(parse_velocity)),
        ),
        |(notes, (duration, velocity))| {
            MusicElement::Chord(Chord {
                notes,
                duration,
                velocity: velocity.unwrap_or(127),
            })
        },
    )(input)
}

fn parse_pitch(input: &str) -> IResult<&str, Pitch> {
    map(
        pair(
            alt((
                tag("C"), tag("D"), tag("E"), tag("F"),
                tag("G"), tag("A"), tag("B"),
            )),
            pair(opt(alt((tag("#"), tag("b")))),
            terminated(digit1, multispace0)),
        ),
        |(note, (acc, octave))| {
            let note_name = match note {
                "C" => NoteName::C,
                "D" => NoteName::D,
                "E" => NoteName::E,
                "F" => NoteName::F,
                "G" => NoteName::G,
                "A" => NoteName::A,
                "B" => NoteName::B,
                _ => unreachable!(),
            };

            let accidental = match acc {
                Some("#") => Accidental::Sharp,
                Some("b") => Accidental::Flat,
                _ => Accidental::Natural,
            };

            let octave = i8::from_str(octave).unwrap_or(4);

            Pitch {
                note: note_name,
                octave,
                accidental,
            }
        },
    )(input)
}

fn parse_duration(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize(separated_pair(digit1, opt(tag("/")), opt(digit1))),
        |s: &str| {
            if let Some((n, d)) = s.split_once('/') {
                Ok(n.parse::<f32>()? / d.parse::<f32>()?)
            } else {
                Ok(1.0 / s.parse::<f32>()?)
            }
        },
    )(input)
}

fn parse_velocity(input: &str) -> IResult<&str, u8> {
    map_res(digit1, |s: &str| s.parse::<u8>())(input)
}

// 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note() {
        assert_eq!(
            parse_music("C4:4"),
            Ok(vec![MusicElement::Note(Note {
                pitch: Pitch {
                    note: NoteName::C,
                    octave: 4,
                    accidental: Accidental::Natural
                },
                duration: 0.25,
                velocity: 127
            })])
        );
    }

    #[test]
    fn test_parse_chord() {
        assert_eq!(
            parse_music("(C4 E4 G4):2@100"),
            Ok(vec![MusicElement::Chord(Chord {
                notes: vec![
                    Pitch {
                        note: NoteName::C,
                        octave: 4,
                        accidental: Accidental::Natural
                    },
                    Pitch {
                        note: NoteName::E,
                        octave: 4,
                        accidental: Accidental::Natural
                    },
                    Pitch {
                        note: NoteName::G,
                        octave: 4,
                        accidental: Accidental::Natural
                    },
                ],
                duration: 0.5,
                velocity: 100
            })])
        );
    }

    #[test]
    fn test_parse_rest() {
        assert_eq!(
            parse_music("R:1"),
            Ok(vec![MusicElement::Rest(Rest { duration: 1.0 })])
        );
    }

    #[test]
    fn test_complex_sequence() {
        assert_eq!(
            parse_music("C4:8 D#5:16 R:4 (A3 Bb4):2@80"),
            Ok(vec![
                MusicElement::Note(Note {
                    pitch: Pitch {
                        note: NoteName::C,
                        octave: 4,
                        accidental: Accidental::Natural
                    },
                    duration: 0.125,
                    velocity: 127
                }),
                MusicElement::Note(Note {
                    pitch: Pitch {
                        note: NoteName::D,
                        octave: 5,
                        accidental: Accidental::Sharp
                    },
                    duration: 0.0625,
                    velocity: 127
                }),
                MusicElement::Rest(Rest { duration: 0.25 }),
                MusicElement::Chord(Chord {
                    notes: vec![
                        Pitch {
                            note: NoteName::A,
                            octave: 3,
                            accidental: Accidental::Natural
                        },
                        Pitch {
                            note: NoteName::B,
                            octave: 4,
                            accidental: Accidental::Flat
                        },
                    ],
                    duration: 0.5,
                    velocity: 80
                })
            ])
        );
    }
}