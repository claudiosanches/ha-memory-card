//! Value lists shared by the API, the database and the CSV import. Stored as
//! their snake_case names.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Lowercase and keep only letters, digits and `%`, so "NTSC-U", "ntsc u"
/// and "Ntsc_U" compare equal.
pub fn loose(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphanumeric() || *c == '%')
        .collect()
}

macro_rules! value_list {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $text:literal [$($alias:literal),*]),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, sqlx::Type)]
        #[serde(rename_all = "snake_case")]
        #[sqlx(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &[Self] = &[$(Self::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            /// Accepts the stored name or a common spelling, ignoring case,
            /// spaces and punctuation.
            pub fn parse_loose(value: &str) -> Option<Self> {
                let key = loose(value);
                $(
                    if key == loose($text) $(|| key == $alias)* {
                        return Some(Self::$variant);
                    }
                )+
                None
            }
        }
    };
}

value_list! {
    /// Release region, matching IGDB's release regions. `ww` also covers region-free.
    Region {
        Na = "na" ["ntscu", "us", "usa", "northamerica"],
        Eu = "eu" ["pal", "eur", "europe", "uk"],
        Au = "au" ["aus", "australia"],
        Nz = "nz" ["newzealand"],
        Jp = "jp" ["ntscj", "jpn", "japan"],
        Cn = "cn" ["chn", "china"],
        Asia = "asia" [],
        Kr = "kr" ["kor", "korea"],
        Br = "br" ["bra", "brazil", "brasil"],
        Ww = "ww" ["world", "worldwide", "regionfree", "free", "all"],
    }
}

value_list! {
    Format {
        Physical = "physical" ["disc", "cartridge", "cart"],
        Digital = "digital" ["download"],
    }
}

value_list! {
    /// Physical copies only.
    Condition {
        Sealed = "sealed" ["new"],
        Cib = "cib" ["complete", "completeinbox"],
        NoManual = "no_manual" [],
        Loose = "loose" ["cartonly", "disconly"],
    }
}

value_list! {
    CopyStatus {
        Owned = "owned" ["have", "own", "incollection", "collection"],
        Wanted = "wanted" ["wishlist", "want", "wanttobuy", "onorder", "preordered"],
    }
}

value_list! {
    /// What a piece of hardware is.
    DeviceKind {
        Console = "console" [],
        Handheld = "handheld" ["portable"],
        Pc = "pc" ["computer"],
        Controller = "controller" ["gamepad"],
        Accessory = "accessory" [],
        Toy = "toy" ["amiibo", "figure"],
    }
}

impl DeviceKind {
    /// Consoles, handhelds and PCs hold copies; the rest is hardware for a
    /// platform.
    pub fn holds_games(self) -> bool {
        matches!(self, Self::Console | Self::Handheld | Self::Pc)
    }
}

value_list! {
    /// Where a copy came from: added in the app, or by a CSV import.
    CopySource {
        Manual = "manual" [],
        Csv = "csv" [],
    }
}

value_list! {
    /// The play statuses every install has. Users can add more; see `play_statuses`.
    BuiltinStatus {
        Backlog = "backlog" ["unplayed", "notstarted", "notplayed", "toplay", "wanttoplay"],
        Playing = "playing" ["nowplaying", "currentlyplaying", "inprogress", "started", "endless"],
        Paused = "paused" ["onhold", "unfinished", "shelved"],
        Completed = "completed" ["beaten", "finished", "complete", "done"],
        Dropped = "dropped" ["abandoned", "retired", "quit", "gaveup"],
        Mastered = "mastered" ["100", "100%", "platinum", "perfected"],
    }
}

value_list! {
    /// How the backlog page and suggestions treat a play status.
    StatusGroup {
        Backlog = "backlog" [],
        Playing = "playing" [],
        Paused = "paused" [],
        Finished = "finished" [],
        NotCounted = "not_counted" [],
    }
}

value_list! {
    /// Mantine palette names.
    StatusColor {
        Gray = "gray" [],
        Red = "red" [],
        Pink = "pink" [],
        Grape = "grape" [],
        Violet = "violet" [],
        Indigo = "indigo" [],
        Blue = "blue" [],
        Cyan = "cyan" [],
        Teal = "teal" [],
        Green = "green" [],
        Lime = "lime" [],
        Yellow = "yellow" [],
        Orange = "orange" [],
    }
}

/// A UPC or EAN (GTIN-8, -12, -13 or -14) as digits, if its check digit is right.
/// Spaces and hyphens are ignored.
pub fn barcode(text: &str) -> Option<String> {
    let digits: String = text.chars().filter(|c| !matches!(c, ' ' | '-')).collect();
    if !matches!(digits.len(), 8 | 12 | 13 | 14) || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let values: Vec<u32> = digits.bytes().map(|b| u32::from(b - b'0')).collect();
    let (check, body) = values.split_last()?;
    // From the right, digits alternate weights 3 and 1.
    let sum: u32 = body
        .iter()
        .rev()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { d * 3 } else { *d })
        .sum();
    ((10 - sum % 10) % 10 == *check).then_some(digits)
}

/// Search key for a title: compatibility forms folded (full-width letters,
/// ligatures), accents dropped, lowercase, runs of spaces made one.
/// Punctuation stays.
pub fn search_key(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    use unicode_normalization::char::is_combining_mark;
    let folded: String = text
        .nfkd()
        // Kana voicing marks change the word, unlike Latin accents.
        .filter(|c| !is_combining_mark(*c) || matches!(c, '\u{3099}' | '\u{309A}'))
        .flat_map(char::to_lowercase)
        .collect();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Sort key for a title: lowercase, leading English article moved out.
pub fn sort_title(title: &str) -> String {
    let lower = title.trim().to_lowercase();
    for article in ["the ", "a ", "an "] {
        if let Some(rest) = lower.strip_prefix(article) {
            let rest = rest.trim_start();
            if !rest.is_empty() {
                return rest.to_owned();
            }
        }
    }
    lower
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_statuses_other_trackers_use() {
        for (text, status) in [
            ("Beaten", BuiltinStatus::Completed),
            ("Unfinished", BuiltinStatus::Paused),
            ("Unplayed", BuiltinStatus::Backlog),
            ("Retired", BuiltinStatus::Dropped),
            ("Now Playing", BuiltinStatus::Playing),
        ] {
            assert_eq!(BuiltinStatus::parse_loose(text), Some(status), "{text}");
        }
        assert_eq!(
            CopyStatus::parse_loose("In Collection"),
            Some(CopyStatus::Owned)
        );
        assert_eq!(
            CopyStatus::parse_loose("Wish List"),
            Some(CopyStatus::Wanted)
        );
    }

    #[test]
    fn parses_names_and_aliases() {
        assert_eq!(Region::parse_loose("NTSC-U"), Some(Region::Na));
        assert_eq!(Region::parse_loose(" pal "), Some(Region::Eu));
        assert_eq!(Region::parse_loose("Region free"), Some(Region::Ww));
        assert_eq!(Region::parse_loose("BR"), Some(Region::Br));
        assert_eq!(Region::parse_loose("mars"), None);
        assert_eq!(
            Condition::parse_loose("No manual"),
            Some(Condition::NoManual)
        );
        assert_eq!(
            Condition::parse_loose("no_manual"),
            Some(Condition::NoManual)
        );
        assert_eq!(Condition::parse_loose("CIB"), Some(Condition::Cib));
        assert_eq!(
            BuiltinStatus::parse_loose("100%"),
            Some(BuiltinStatus::Mastered)
        );
        assert_eq!(Format::parse_loose("Cartridge"), Some(Format::Physical));
    }

    #[test]
    fn names_round_trip() {
        for region in Region::ALL {
            assert_eq!(Region::parse_loose(region.as_str()), Some(*region));
        }
        for status in BuiltinStatus::ALL {
            assert_eq!(BuiltinStatus::parse_loose(status.as_str()), Some(*status));
        }
    }

    #[test]
    fn search_key_folds_accents_case_and_width() {
        assert_eq!(
            search_key("Pokémon  Mystery Dungeon"),
            "pokemon mystery dungeon"
        );
        assert_eq!(search_key("ＦＩＮＡＬ ＦＡＮＴＡＳＹ"), "final fantasy");
        assert_eq!(search_key("Ōkami"), "okami");
        assert_eq!(
            search_key("Pilot's Log; 100%_Complete"),
            "pilot's log; 100%_complete"
        );
        assert_ne!(search_key("ゼルダ"), search_key("セルダ"));
    }

    #[test]
    fn barcodes_need_a_right_check_digit() {
        assert_eq!(barcode("0 45496 59002 4").as_deref(), Some("045496590024"));
        assert_eq!(barcode("4902370536485").as_deref(), Some("4902370536485"));
        assert_eq!(barcode("96385074").as_deref(), Some("96385074"));
        assert_eq!(barcode("4902370536484"), None);
        assert_eq!(barcode("12345"), None);
        assert_eq!(barcode("ABCDEFGHIJKL"), None);
    }

    #[test]
    fn sort_title_moves_articles() {
        assert_eq!(sort_title("The Legend of Zelda"), "legend of zelda");
        assert_eq!(sort_title("A Hat in Time"), "hat in time");
        assert_eq!(sort_title("Super Mario Odyssey"), "super mario odyssey");
        assert_eq!(sort_title("The "), "the");
    }
}
