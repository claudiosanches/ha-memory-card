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

/// The image without its metadata, or `None` when it is not a well-formed
/// JPEG, PNG or WebP.
pub fn clean(bytes: &[u8]) -> Option<Cleaned> {
    let (bytes, extension) = match bytes {
        [0xFF, 0xD8, 0xFF, ..] => (jpeg(bytes)?, "jpg"),
        [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => (png(bytes)?, "png"),
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
        ] => (webp(bytes)?, "webp"),
        _ => return None,
    };
    Some(Cleaned { bytes, extension })
}

const SOI: [u8; 2] = [0xFF, 0xD8];
const EOI: [u8; 2] = [0xFF, 0xD9];

/// Walks the segments up to the first scan, keeping those that decode the
/// image, then keeps the image data up to its end marker. Anything after it,
/// such as a second preview image, goes.
fn jpeg(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut out = SOI.to_vec();
    let mut at = 2;
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
        let keep = match marker {
            0xDA => {
                out.extend([0xFF, marker]);
                out.extend(segment);
                let rest = &bytes[at..];
                match rest.windows(2).position(|pair| pair == EOI) {
                    Some(end) => out.extend(&rest[..end + 2]),
                    // Some cameras leave off the end marker.
                    None => {
                        out.extend(rest);
                        out.extend(EOI);
                    }
                }
                return Some(out);
            }
            0xE1 => {
                if let Some(orientation) = exif_orientation(data).filter(|o| *o != 1) {
                    out.extend(orientation_segment(orientation));
                }
                false
            }
            0xE0 => data.starts_with(b"JFIF\0"),
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
    let entries = usize::from(u16_at(ifd)?);
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

fn png(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut out = bytes[..8].to_vec();
    let mut at = 8;
    loop {
        let length =
            usize::try_from(u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?)).ok()?;
        let kind: &[u8; 4] = bytes.get(at + 4..at + 8)?.try_into().ok()?;
        if at == 8 && kind != b"IHDR" {
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
            return Some(out);
        }
    }
}

/// Chunks that draw the image, its colors and animation.
const WEBP_KEPT: [&[u8; 4]; 7] = [
    b"VP8 ", b"VP8L", b"VP8X", b"ALPH", b"ANIM", b"ANMF", b"ICCP",
];
/// VP8X flags for the EXIF and XMP chunks.
const VP8X_METADATA: u8 = 0x08 | 0x04;

fn webp(bytes: &[u8]) -> Option<Vec<u8>> {
    let size = usize::try_from(u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?)).ok()?;
    let end = size.checked_add(8)?.min(bytes.len());
    let mut chunks = Vec::new();
    let mut at = 12;
    while at + 8 <= end {
        let kind: &[u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
        let length = usize::try_from(u32::from_le_bytes(
            bytes.get(at + 4..at + 8)?.try_into().ok()?,
        ))
        .ok()?;
        let data_end = at.checked_add(8)?.checked_add(length)?;
        let chunk = bytes.get(at..data_end)?;
        if chunks.is_empty() && !matches!(kind, b"VP8 " | b"VP8L" | b"VP8X") {
            return None;
        }
        if WEBP_KEPT.contains(&kind) {
            let start = chunks.len();
            chunks.extend(chunk);
            if kind == b"VP8X" {
                *chunks.get_mut(start + 8)? &= !VP8X_METADATA;
            }
            // Chunks are padded to an even length.
            if length % 2 == 1 {
                chunks.push(0);
            }
        }
        at = data_end + length % 2;
    }
    if chunks.is_empty() {
        return None;
    }
    let mut out = b"RIFF".to_vec();
    out.extend(u32::try_from(chunks.len() + 4).ok()?.to_le_bytes());
    out.extend(b"WEBP");
    out.extend(chunks);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_what_is_not_an_image() {
        assert_eq!(clean(b"<svg></svg>"), None);
        assert_eq!(clean(b""), None);
        assert_eq!(clean(b"\xFF\xD8\xFF\xE0\x00\x10JFIF"), None, "cut short");
        assert_eq!(clean(b"\x89PNG\r\n\x1a\nnot really"), None);
        assert_eq!(clean(b"RIFF\x04\x00\x00\x00WEBP"), None);
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

    #[test]
    fn reads_orientation_in_either_byte_order() {
        let big = b"Exif\0\0MM\0*\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01\0\x06\0\0\0\0\0\0";
        assert_eq!(exif_orientation(big), Some(6));
        let little = b"Exif\0\0II*\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x03\0\0\0\0\0\0\0";
        assert_eq!(exif_orientation(little), Some(3));
        assert_eq!(exif_orientation(&orientation_segment(8)[4..]), Some(8));
        assert_eq!(exif_orientation(b"Exif\0\0MM\0*\0\0\0\xFF"), None);
    }
}
