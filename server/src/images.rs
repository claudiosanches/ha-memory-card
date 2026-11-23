//! Checks images before they are stored and strips their metadata. Phone
//! photos carry the GPS position and the device in EXIF, and every viewer
//! can open the stored files, so only what draws the image is kept. A JPEG
//! keeps its orientation, or a portrait photo would show sideways.

/// An image ready to store.
#[derive(Debug, PartialEq, Eq)]
pub struct Cleaned {
    pub bytes: Vec<u8>,
    pub extension: &'static str,
}

/// Why an image was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum Rejected {
    /// Not a well-formed JPEG, PNG or WebP.
    Invalid,
    /// More than [`PIXELS_MAX`] pixels.
    TooLarge,
}

/// More than any camera takes; a small file can claim a canvas big enough
/// to run a viewer's browser out of memory.
pub const PIXELS_MAX: u64 = 100_000_000;

/// Width and height.
type Size = (u32, u32);

/// The image without its metadata.
pub fn clean(bytes: &[u8]) -> Result<Cleaned, Rejected> {
    let image = match bytes {
        [0xFF, 0xD8, 0xFF, ..] => jpeg(bytes).map(|(bytes, size)| (bytes, size, "jpg")),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => {
            png(bytes).map(|(bytes, size)| (bytes, size, "png"))
        }
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'E',
            b'B',
            b'P',
            ..,
        ] => webp(bytes).map(|(bytes, size)| (bytes, size, "webp")),
        _ => None,
    };
    let (bytes, (width, height), extension) = image.ok_or(Rejected::Invalid)?;
    if width == 0 || height == 0 {
        return Err(Rejected::Invalid);
    }
    if u64::from(width) * u64::from(height) > PIXELS_MAX {
        return Err(Rejected::TooLarge);
    }
    Ok(Cleaned { bytes, extension })
}

const SOI: [u8; 2] = [0xFF, 0xD8];
const EOI: [u8; 2] = [0xFF, 0xD9];

/// JFIF's identifier, version, density and thumbnail size, before the thumbnail.
const JFIF_HEADER: usize = 14;

/// Walks the segments, keeping those that decode the image, and the image
/// data of every scan; a progressive JPEG has several, with segments between
/// them. Anything after the end marker, such as a second preview image, goes.
fn jpeg(bytes: &[u8]) -> Option<(Vec<u8>, Size)> {
    let mut out = SOI.to_vec();
    let mut at = 2;
    let mut scanned = false;
    let mut size = None;
    loop {
        if *bytes.get(at)? != 0xFF {
            return None;
        }
        // Any number of 0xFF may pad before a marker.
        while bytes.get(at) == Some(&0xFF) {
            at += 1;
        }
        let marker = *bytes.get(at)?;
        at += 1;
        match marker {
            0xD9 if scanned => {
                out.extend(EOI);
                return Some((out, size?));
            }
            // A file that ends before any image data.
            0xD9 => return None,
            // Markers without a length.
            0x01 | 0xD0..=0xD7 => {
                out.extend([0xFF, marker]);
                continue;
            }
            _ => {}
        }
        let length = usize::from(u16::from_be_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]));
        let segment = bytes
            .get(at..at.checked_add(length)?)
            .filter(|_| length >= 2)?;
        let data = &segment[2..];
        at += length;
        // Start of frame: precision, then height and width. Browsers size the
        // image from the first, so a second one is refused.
        if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
            if size.is_some() {
                return None;
            }
            let height = u16::from_be_bytes([*data.get(1)?, *data.get(2)?]);
            let width = u16::from_be_bytes([*data.get(3)?, *data.get(4)?]);
            size = Some((u32::from(width), u32::from(height)));
        }
        let keep = match marker {
            0xDA => {
                out.extend([0xFF, marker]);
                out.extend(segment);
                let end = at + scan_length(&bytes[at..]);
                out.extend(&bytes[at..end]);
                at = end;
                scanned = true;
                // Some cameras leave off the end marker.
                if at == bytes.len() {
                    out.extend(EOI);
                    return Some((out, size?));
                }
                continue;
            }
            0xE1 => {
                if let Some(orientation) = exif_orientation(data).filter(|o| *o != 1) {
                    out.extend(orientation_segment(orientation));
                }
                false
            }
            // JFIF's header stays, without the thumbnail it may carry.
            0xE0 => {
                if data.starts_with(b"JFIF\0") && data.len() >= JFIF_HEADER {
                    out.extend([0xFF, 0xE0, 0, 2 + JFIF_HEADER as u8]);
                    out.extend(&data[..JFIF_HEADER - 2]);
                    out.extend([0, 0]);
                }
                false
            }
            0xE2 => data.starts_with(b"ICC_PROFILE\0"),
            // Says how to read the colors of some CMYK files.
            0xEE => data.starts_with(b"Adobe"),
            // Other application segments and comments are metadata.
            0xE3..=0xED | 0xEF | 0xFE => false,
            _ => true,
        };
        if keep {
            out.extend([0xFF, marker]);
            out.extend(segment);
        }
    }
}

/// How long the image data that starts `bytes` runs: up to the next marker.
/// In the data, 0xFF00 stands for 0xFF and restart markers belong to it.
fn scan_length(bytes: &[u8]) -> usize {
    let mut at = 0;
    while let Some(offset) = bytes[at..].iter().position(|&b| b == 0xFF) {
        let ff = at + offset;
        match bytes.get(ff + 1) {
            Some(0x00 | 0xD0..=0xD7) => at = ff + 2,
            Some(_) => return ff,
            None => break,
        }
    }
    bytes.len()
}

/// The EXIF orientation, 1 to 8, from an APP1 segment.
fn exif_orientation(data: &[u8]) -> Option<u16> {
    let tiff = data.strip_prefix(b"Exif\0\0")?;
    let big_endian = match tiff.get(..4)? {
        b"MM\0*" => true,
        b"II*\0" => false,
        _ => return None,
    };
    let u16_at = |i: usize| {
        let b: [u8; 2] = tiff.get(i..i.checked_add(2)?)?.try_into().ok()?;
        Some(if big_endian {
            u16::from_be_bytes(b)
        } else {
            u16::from_le_bytes(b)
        })
    };
    let u32_at = |i: usize| {
        let b: [u8; 4] = tiff.get(i..i.checked_add(4)?)?.try_into().ok()?;
        Some(if big_endian {
            u32::from_be_bytes(b)
        } else {
            u32::from_le_bytes(b)
        })
    };
    let ifd = usize::try_from(u32_at(4)?).ok()?;
    // No more entries than the segment holds, whatever the count says.
    let room = tiff.len().saturating_sub(ifd + 2) / 12;
    let entries = usize::from(u16_at(ifd)?).min(room);
    (0..entries).find_map(|n| {
        let entry = ifd + 2 + n * 12;
        // A SHORT value sits at the start of the entry's value field.
        (u16_at(entry)? == 0x0112)
            .then(|| u16_at(entry + 8))
            .flatten()
            .filter(|value| (1..=8).contains(value))
    })
}

/// An APP1 segment whose EXIF holds only the orientation.
fn orientation_segment(orientation: u16) -> Vec<u8> {
    let mut segment = vec![0xFF, 0xE1, 0x00, 0x22];
    segment.extend(b"Exif\0\0MM\0*");
    segment.extend(8u32.to_be_bytes());
    segment.extend(1u16.to_be_bytes());
    segment.extend(0x0112u16.to_be_bytes());
    segment.extend(3u16.to_be_bytes());
    segment.extend(1u32.to_be_bytes());
    segment.extend(orientation.to_be_bytes());
    segment.extend([0, 0]);
    segment.extend(0u32.to_be_bytes());
    segment
}

/// Chunks that draw the image, its colors and animation. Text, EXIF, time
/// and unknown chunks go.
const PNG_KEPT: [&[u8; 4]; 15] = [
    b"IHDR", b"PLTE", b"IDAT", b"IEND", b"tRNS", b"gAMA", b"cHRM", b"sRGB", b"iCCP", b"sBIT",
    b"pHYs", b"bKGD", b"acTL", b"fcTL", b"fdAT",
];

fn png(bytes: &[u8]) -> Option<(Vec<u8>, Size)> {
    let mut out = bytes[..8].to_vec();
    let mut at = 8;
    let read = |at: usize| Some(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    // IHDR comes first and starts with the width and height.
    let size = (read(16)?, read(20)?);
    loop {
        let length =
            usize::try_from(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?)).ok()?;
        let kind: &[u8; 4] = bytes.get(at + 4..at + 8)?.try_into().ok()?;
        if (at == 8) != (kind == b"IHDR") {
            return None;
        }
        // Length, type, data and CRC.
        let end = at.checked_add(12)?.checked_add(length)?;
        let chunk = bytes.get(at..end)?;
        if PNG_KEPT.contains(&kind) {
            out.extend(chunk);
        }
        at = end;
        if kind == b"IEND" {
            return Some((out, size));
        }
    }
}

/// Chunks that draw the image, its colors and animation.
const WEBP_KEPT: [&[u8; 4]; 7] = [
    b"VP8 ", b"VP8L", b"VP8X", b"ALPH", b"ANIM", b"ANMF", b"ICCP",
];
/// Chunks that draw an animation frame; others may sit in it too.
const WEBP_FRAME_KEPT: [&[u8; 4]; 3] = [b"ALPH", b"VP8 ", b"VP8L"];
/// A frame's position, size, duration and flags, before its chunks.
const ANMF_HEADER: usize = 16;
/// VP8X flags for the EXIF and XMP chunks.
const VP8X_METADATA: u8 = 0x08 | 0x04;

fn webp(bytes: &[u8]) -> Option<(Vec<u8>, Size)> {
    let size = usize::try_from(u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?)).ok()?;
    let end = size.checked_add(8)?.min(bytes.len());
    let chunks = riff_chunks(bytes.get(12..end)?)?;
    let (first, data) = *chunks.first()?;
    let size = webp_size(first, data)?;
    let mut out = b"RIFF".to_vec();
    out.extend([0; 4]);
    out.extend(b"WEBP");
    for (kind, data) in chunks {
        match kind {
            b"VP8X" => {
                let mut data = data.to_vec();
                *data.first_mut()? &= !VP8X_METADATA;
                push_chunk(&mut out, kind, &data)?;
            }
            b"ANMF" => {
                let mut frame = data.get(..ANMF_HEADER)?.to_vec();
                for (kind, data) in riff_chunks(&data[ANMF_HEADER..])? {
                    if WEBP_FRAME_KEPT.contains(&kind) {
                        push_chunk(&mut frame, kind, data)?;
                    }
                }
                push_chunk(&mut out, kind, &frame)?;
            }
            _ if WEBP_KEPT.contains(&kind) => push_chunk(&mut out, kind, data)?,
            _ => {}
        }
    }
    let riff_size = u32::try_from(out.len() - 8).ok()?;
    out[4..8].copy_from_slice(&riff_size.to_le_bytes());
    Some((out, size))
}

/// The canvas a WebP's first chunk gives.
fn webp_size(kind: &[u8; 4], data: &[u8]) -> Option<Size> {
    let le = |at: usize, len: usize| {
        let mut word = [0u8; 4];
        word[..len].copy_from_slice(data.get(at..at + len)?);
        Some(u32::from_le_bytes(word))
    };
    match kind {
        b"VP8X" => Some((le(4, 3)? + 1, le(7, 3)? + 1)),
        // A key frame's start code, then 14-bit width and height.
        b"VP8 " if data.get(3..6)? == [0x9D, 0x01, 0x2A] => {
            Some((le(6, 2)? & 0x3FFF, le(8, 2)? & 0x3FFF))
        }
        b"VP8L" if *data.first()? == 0x2F => {
            let bits = le(1, 4)?;
            Some(((bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1))
        }
        _ => None,
    }
}

/// A RIFF body's chunks, as kind and data; `None` when one runs past the end.
fn riff_chunks(bytes: &[u8]) -> Option<Vec<(&[u8; 4], &[u8])>> {
    let mut chunks = Vec::new();
    let mut at = 0;
    while at + 8 <= bytes.len() {
        let kind: &[u8; 4] = bytes[at..at + 4].try_into().ok()?;
        let length =
            usize::try_from(u32::from_le_bytes(bytes[at + 4..at + 8].try_into().ok()?)).ok()?;
        let data_end = at.checked_add(8)?.checked_add(length)?;
        chunks.push((kind, bytes.get(at + 8..data_end)?));
        // Chunks are padded to an even length.
        at = data_end + length % 2;
    }
    Some(chunks)
}

fn push_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) -> Option<()> {
    out.extend(kind);
    out.extend(u32::try_from(data.len()).ok()?.to_le_bytes());
    out.extend(data);
    if data.len() % 2 == 1 {
        out.push(0);
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_what_is_not_an_image() {
        let invalid = Err(Rejected::Invalid);
        assert_eq!(clean(b"<svg></svg>"), invalid);
        assert_eq!(clean(b""), invalid);
        assert_eq!(clean(b"\xFF\xD8\xFF\xE0\x00\x10JFIF"), invalid, "cut short");
        assert_eq!(clean(b"\x89PNG\r\n\x1a\nnot really"), invalid);
        assert_eq!(clean(b"RIFF\x04\x00\x00\x00WEBP"), invalid);
    }

    /// Photos with GPS EXIF, a device name, XMP, a comment and, for the
    /// JPEG, a preview after the end of the image.
    const PHOTOS: [(&[u8], &str); 3] = [
        (include_bytes!("../tests/fixtures/images/photo.jpg"), "jpg"),
        (include_bytes!("../tests/fixtures/images/photo.png"), "png"),
        (
            include_bytes!("../tests/fixtures/images/photo.webp"),
            "webp",
        ),
    ];

    #[test]
    fn strips_metadata_and_keeps_the_image() {
        for (photo, extension) in PHOTOS {
            let cleaned = clean(photo).unwrap();
            assert_eq!(cleaned.extension, extension);
            let text = String::from_utf8_lossy(&cleaned.bytes);
            for secret in ["PhoneCo", "secret place", "shot at home", "trailing"] {
                assert!(!text.contains(secret), "{extension} kept {secret}");
            }
            assert!(cleaned.bytes.len() < photo.len(), "{extension}");
            assert_eq!(
                clean(&cleaned.bytes).unwrap(),
                cleaned,
                "{extension} is stable"
            );
        }
    }

    #[test]
    fn a_progressive_jpeg_keeps_every_scan_and_drops_what_sits_between() {
        let photo = include_bytes!("../tests/fixtures/images/progressive.jpg");
        let cleaned = clean(photo).unwrap().bytes;
        let text = String::from_utf8_lossy(&cleaned);
        assert!(!text.contains("shot at home") && !text.contains("PhoneCo"));
        let scans = |b: &[u8]| b.windows(2).filter(|w| *w == [0xFF, 0xDA]).count();
        assert_eq!(scans(&cleaned), scans(photo));
        assert!(cleaned.ends_with(&EOI));
        assert_eq!(clean(&cleaned).unwrap().bytes, cleaned, "stable");
    }

    #[test]
    fn a_jpeg_keeps_its_jfif_header_without_the_thumbnail() {
        let header = b"JFIF\0\x01\x02\x00\x00\x48\x00\x48";
        let photo = [
            &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x13][..],
            header,
            &[0x01, 0x01],
            b"TMB",
            &frame(1, 1),
        ]
        .concat();
        let cleaned = [
            &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10][..],
            header,
            &[0x00, 0x00],
            &frame(1, 1),
        ]
        .concat();
        assert_eq!(clean(&photo).unwrap().bytes, cleaned);
        assert_eq!(clean(&cleaned).unwrap().bytes, cleaned);
    }

    #[test]
    fn a_jpeg_keeps_only_its_orientation() {
        let cleaned = clean(PHOTOS[0].0).unwrap().bytes;
        let at = cleaned.windows(6).position(|w| w == b"Exif\0\0").unwrap();
        assert_eq!(exif_orientation(&cleaned[at..]), Some(6));
        assert!(cleaned.ends_with(&EOI));
    }

    #[test]
    fn a_webp_drops_its_metadata_flags() {
        let cleaned = clean(PHOTOS[2].0).unwrap().bytes;
        if let Some(at) = cleaned.windows(4).position(|w| w == b"VP8X") {
            assert_eq!(cleaned[at + 8] & VP8X_METADATA, 0);
        }
        let size = u32::from_le_bytes(cleaned[4..8].try_into().unwrap()) as usize;
        assert_eq!(size + 8, cleaned.len());
    }

    /// A JPEG's frame header for one gray channel, then a scan and the end.
    fn frame(width: u16, height: u16) -> Vec<u8> {
        let [h1, h2] = height.to_be_bytes();
        let [w1, w2] = width.to_be_bytes();
        let mut bytes = vec![
            0xFF, 0xC0, 0x00, 0x0B, 0x08, h1, h2, w1, w2, 0x01, 0x01, 0x11, 0x00,
        ];
        bytes.extend([0xFF, 0xDA, 0x00, 0x02, 0x12, 0x34, 0xFF, 0xD9]);
        bytes
    }

    fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        push_chunk(&mut out, kind, data).unwrap();
        out
    }

    fn riff(chunks: &[Vec<u8>]) -> Vec<u8> {
        let body = chunks.concat();
        let size = u32::try_from(body.len() + 4).unwrap().to_le_bytes();
        [&b"RIFF"[..], &size, b"WEBP", &body].concat()
    }

    fn vp8x(width: u32, height: u32, flags: u8) -> Vec<u8> {
        let mut data = vec![flags, 0, 0, 0];
        data.extend(&(width - 1).to_le_bytes()[..3]);
        data.extend(&(height - 1).to_le_bytes()[..3]);
        chunk(b"VP8X", &data)
    }

    #[test]
    fn an_animation_frame_keeps_only_what_draws_it() {
        let lossless = chunk(b"VP8L", &[0x2F, 0, 0, 0, 0, 0x10]);
        let header = [0u8; ANMF_HEADER];
        let frame = |inside: &[Vec<u8>]| chunk(b"ANMF", &[&header[..], &inside.concat()].concat());
        let anim = chunk(b"ANIM", &[0; 6]);
        let photo = riff(&[
            vp8x(1, 1, 0x02 | VP8X_METADATA),
            anim.clone(),
            frame(&[chunk(b"PRIV", b"secret place"), lossless.clone()]),
        ]);
        let cleaned = clean(&photo).unwrap().bytes;
        assert_eq!(cleaned, riff(&[vp8x(1, 1, 0x02), anim, frame(&[lossless])]));
        assert_eq!(clean(&cleaned).unwrap().bytes, cleaned, "stable");
    }

    #[test]
    fn images_past_100_megapixels_are_too_large() {
        assert!(clean(&[&SOI[..], &frame(10_000, 10_000)].concat()).is_ok());
        assert_eq!(
            clean(&[&SOI[..], &frame(12_000, 9_000)].concat()),
            Err(Rejected::TooLarge)
        );
        assert_eq!(
            clean(&[&SOI[..], &frame(0, 9_000)].concat()),
            Err(Rejected::Invalid)
        );

        let ihdr = |width: u32, height: u32| {
            let mut data = width.to_be_bytes().to_vec();
            data.extend(height.to_be_bytes());
            data.extend([8, 6, 0, 0, 0]);
            let mut chunk = 13u32.to_be_bytes().to_vec();
            chunk.extend(b"IHDR");
            chunk.extend(data);
            chunk.extend([0; 4]);
            chunk
        };
        let iend = [0, 0, 0, 0, b'I', b'E', b'N', b'D', 0, 0, 0, 0];
        let png = |width, height| [&PHOTOS[1].0[..8], &ihdr(width, height), &iend].concat();
        assert!(clean(&png(4_000, 3_000)).is_ok());
        assert_eq!(clean(&png(65_535, 65_535)), Err(Rejected::TooLarge));

        assert_eq!(
            clean(&riff(&[vp8x(16_384, 16_384, 0)])),
            Err(Rejected::TooLarge)
        );
        let lossy = |width: u16, height: u16| {
            let mut data = vec![0, 0, 0, 0x9D, 0x01, 0x2A];
            data.extend(width.to_le_bytes());
            data.extend(height.to_le_bytes());
            riff(&[chunk(b"VP8 ", &data)])
        };
        assert!(clean(&lossy(640, 480)).is_ok());
        assert_eq!(clean(&lossy(16_383, 16_383)), Err(Rejected::TooLarge));

        // A small second header cannot stand in for a huge first one.
        let huge = frame(23_000, 23_000);
        assert_eq!(
            clean(&[&SOI[..], &huge[..huge.len() - 2], &frame(1, 1)].concat()),
            Err(Rejected::Invalid)
        );
        let twice = [&PHOTOS[1].0[..8], &ihdr(1, 1), &ihdr(1, 1), &iend].concat();
        assert_eq!(clean(&twice), Err(Rejected::Invalid));
    }

    #[test]
    fn reads_orientation_in_either_byte_order() {
        let big = b"Exif\0\0MM\0*\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0\x06\0\0\0\0\0\0";
        assert_eq!(exif_orientation(big), Some(6));
        let little = b"Exif\0\0II*\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x03\0\0\0\0\0\0\0";
        assert_eq!(exif_orientation(little), Some(3));
        assert_eq!(exif_orientation(&orientation_segment(8)[4..]), Some(8));
        assert_eq!(exif_orientation(b"Exif\0\0MM\0*\0\0\0\xFF"), None);
    }

    #[test]
    fn an_entry_count_past_the_segment_reads_only_what_is_there() {
        // Says 65,535 entries, holds one.
        let short = b"Exif\0\0MM\0*\0\0\0\x08\xFF\xFF\x01\x12\0\x03\0\0\0\x01\0\x06\0\0";
        assert_eq!(exif_orientation(short), Some(6));
        assert_eq!(exif_orientation(&short[..short.len() - 1]), None);
    }
}
