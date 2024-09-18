//! # mus2midi-rs
//!
//! The [mus2midi] library, ported to Rust from its vendored form in [SLADE3] to
//! provide conversion from the [DMXMUS] file format to [`midly`] sheets.
//!
//! [mus2midi]: https://github.com/sirjuddington/SLADE/blob/d7b5e6efd0a567098f536820b9063f2c4540e100/thirdparty/mus2mid/mus2mid.cpp
//! [SLADE3]: https://slade.mancubus.net/
//! [DMXMUS]: https://doomwiki.org/wiki/MUS

#![doc(
    html_favicon_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech.png",
    html_logo_url = "https://media.githubusercontent.com/media/jerome-trc/viletech/master/assets/viletech.png"
)]

use std::io::Cursor;

use byteorder::ReadBytesExt;
use midly::{self, num::u28, MidiMessage, PitchBend, Smf, TrackEvent, TrackEventKind};

#[must_use]
pub fn is_dmxmus(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }

    bytes[0] == b'M' && bytes[1] == b'U' && bytes[2] == b'S' && bytes[3] == 0x1A
}

/// From SLADE's port of mus2midi.
pub fn to_midi(bytes: &[u8]) -> Result<Smf, Error> {
    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::AnyBitPattern)]
    struct MusHeader {
        id: [u8; 4],
        score_len: u16,
        score_start: u16,
        channels_1: u16,
        channels_2: u16,
        instrument_count: u16,
    }

    #[derive(Debug)]
    struct Channels {
        mapping: [i16; 16],
        velocities: [u8; 16],
    }

    impl Default for Channels {
        fn default() -> Self {
            Self {
                mapping: [Self::UNDEF; 16],
                velocities: [127; 16],
            }
        }
    }

    impl Channels {
        const UNDEF: i16 = -1;
        const MUS_PERCUSSION: i16 = 15;
        const MIDI_PERCUSSION: i16 = 9;

        #[must_use]
        fn allocate(&self) -> i16 {
            let mut ret = self.mapping.iter().copied().max().unwrap_or(Self::UNDEF) + 1;

            // (mus2midi) Don't allocate the MIDI percussion channel!
            if ret == Self::MIDI_PERCUSSION {
                ret += 1;
            }

            ret
        }

        #[must_use]
        fn get_or_allocate(&mut self, channel: u8) -> u8 {
            let i = channel as usize;

            if channel == (Self::MUS_PERCUSSION as u8) {
                return Self::MIDI_PERCUSSION as u8;
            }

            if self.mapping[i] == Self::UNDEF {
                self.mapping[i] = self.allocate();
            }

            self.mapping[i] as u8
        }
    }

    const CONTROLLER_MAP: [u8; 15] = [
        0x00, 0x20, 0x01, 0x07, 0x0A, 0x0B, 0x5B, 0x5D, 0x40, 0x43, 0x78, 0x7B, 0x7E, 0x7F, 0x79,
    ];

    #[must_use]
    fn pass_or_reset_delta(delta: &mut u32) -> u28 {
        let mut time = *delta;
        let mut buf = *delta & 0x7F;

        loop {
            time >>= 7;

            if time == 0 {
                break;
            }

            buf <<= 8;
            buf |= (time & 0x7F) | 0x80;
        }

        let ret = *delta;

        loop {
            if (buf & 0x80) != 0 {
                buf >>= 8;
            } else {
                *delta = 0;
                break;
            }
        }

        ret.into()
    }

    if bytes.len() < std::mem::size_of::<MusHeader>() {
        return Err(Error::Undersize(bytes.len()));
    }

    let mut channels = Channels::default();

    let header_slice = &bytes[0..std::mem::size_of::<MusHeader>()];
    let header: &MusHeader = bytemuck::from_bytes(header_slice);

    if header.id != [b'M', b'U', b'S', 0x1A] {
        return Err(Error::MagicNumber([
            header.id[0],
            header.id[1],
            header.id[2],
            header.id[3],
        ]));
    }

    if (bytes.len() as u64) < (header.score_start as u64) {
        return Err(Error::NoData {
            len: bytes.len(),
            score_start: header.score_start,
        });
    }

    let mut cursor = Cursor::new(bytes);
    cursor.set_position(header.score_start as u64);

    let mut track = midly::Track::new();

    let mut score_finished = false;
    let mut delta = 0_u32;

    while !score_finished {
        // (mus2mid) Handle a block of events.
        while !score_finished {
            let edesc = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)?;
            let channel = channels.get_or_allocate(edesc & 0x0F);

            match edesc & 0x70 {
                0 => {
                    let key = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)?;

                    track.push(TrackEvent {
                        delta: pass_or_reset_delta(&mut delta),
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOff {
                                key: key.into(),
                                vel: 0.into(),
                            },
                        },
                    });
                }
                16 => {
                    let key = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)?;

                    if (key & 0x80) != 0 {
                        let vel = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)?;
                        channels.velocities[channel as usize] = vel & 0x7F;
                    }

                    track.push(TrackEvent {
                        delta: pass_or_reset_delta(&mut delta),
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::NoteOn {
                                key: key.into(),
                                vel: channels.velocities[channel as usize].into(),
                            },
                        },
                    });
                }
                32 => {
                    let key = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)? as u16;

                    track.push(TrackEvent {
                        delta: pass_or_reset_delta(&mut delta),
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::PitchBend {
                                bend: PitchBend((key * 64).into()),
                            },
                        },
                    });
                }
                48 => {
                    let ctrl_num = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)?;

                    if !(10..=14).contains(&ctrl_num) {
                        return Err(Error::InvalidControllerNumber {
                            pos: cursor.position(),
                            num: ctrl_num,
                        });
                    }

                    track.push(TrackEvent {
                        delta: pass_or_reset_delta(&mut delta),
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::Controller {
                                controller: CONTROLLER_MAP[ctrl_num as usize].into(),
                                value: 0.into(),
                            },
                        },
                    });
                }
                64 => {
                    let ctrl_num = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)?;
                    let ctrl_val = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)?;

                    if ctrl_num == 0 {
                        track.push(TrackEvent {
                            delta: pass_or_reset_delta(&mut delta),
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::ProgramChange {
                                    program: ctrl_val.into(),
                                },
                            },
                        });
                    } else {
                        if ctrl_num > 9 {
                            return Err(Error::InvalidControllerNumber {
                                pos: cursor.position(),
                                num: ctrl_num,
                            });
                        }

                        track.push(TrackEvent {
                            delta: pass_or_reset_delta(&mut delta),
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::Controller {
                                    controller: CONTROLLER_MAP[ctrl_num as usize].into(),
                                    value: ctrl_val.into(),
                                },
                            },
                        });
                    }
                }
                96 => {
                    score_finished = true;
                }
                other => {
                    return Err(Error::UnknownEvent {
                        pos: cursor.position(),
                        desc: other,
                    })
                }
            }

            if (edesc & 0x80) != 0 {
                break;
            }
        }

        // (mus2midi) Now read the time code.

        if !score_finished {
            delta = 0_u32;

            loop {
                let working = cursor.read_u8().map_err(|_| Error::UnexpectedEnd)? as u32;

                delta = delta * 128 + (working & 0x7F);

                if (working & 0x80) == 0 {
                    break;
                }
            }
        }
    }

    let mut ret = Smf::new(midly::Header {
        format: midly::Format::SingleTrack,
        timing: midly::Timing::Metrical(70.into()),
    });

    track.push(TrackEvent {
        delta: pass_or_reset_delta(&mut delta),
        kind: TrackEventKind::Meta(midly::MetaMessage::EndOfTrack),
    });

    ret.tracks.push(track);

    Ok(ret)
}

/// Possible failure modes of DMXMUS-to-MIDI conversion.
#[derive(Debug)]
pub enum Error {
    /// The MUS data is not even large enough to fit a header (14 bytes).
    Undersize(usize),
    MagicNumber([u8; 4]),
    NoData {
        len: usize,
        score_start: u16,
    },
    UnknownEvent {
        pos: u64,
        desc: u8,
    },
    UnexpectedEnd,
    InvalidControllerNumber {
        pos: u64,
        num: u8,
    },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Undersize(size) => {
                write!(
                    f,
                    "expected at least 12 bytes for the header; found only {size}"
                )
            }
            Error::MagicNumber(magic) => write!(
                f,
                "expected magic number `0x4D 0x55 0x53 0x1A`, found: {magic:#?}"
            ),
            Error::NoData { len, score_start } => write!(
                f,
                "expected score to start at byte {score_start}, but data is only {len}B long"
            ),
            Error::UnknownEvent { pos, desc } => {
                write!(f, "unknown event {desc} at byte {pos}")
            }
            Error::UnexpectedEnd => todo!(),
            Error::InvalidControllerNumber { pos, num } => {
                write!(f, "invalid controller number {num} as byte {pos}")
            }
        }
    }
}
