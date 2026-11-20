//! Value lists shared by the API, the database and the CSV import. Stored as
//! their snake_case names.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Lowercase and keep only letters, digits and `%`, so "NTSC-U", "ntsc u"
/// and "Ntsc_U" compare equal.
fn loose(value: &str) -> String {
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
        Owned = "owned" [],
        Wanted = "wanted" ["wishlist"],
    }
}

value_list! {
    /// `mastered` means 100% completion.
    PlayStatus {
        Backlog = "backlog" [],
        Playing = "playing" [],
        Paused = "paused" ["onhold"],
        Completed = "completed" ["beaten", "finished"],
        Dropped = "dropped" ["abandoned"],
        Mastered = "mastered" ["100", "100%"],
    }
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
        assert_eq!(PlayStatus::parse_loose("100%"), Some(PlayStatus::Mastered));
        assert_eq!(Format::parse_loose("Cartridge"), Some(Format::Physical));
    }

    #[test]
    fn names_round_trip() {
        for region in Region::ALL {
            assert_eq!(Region::parse_loose(region.as_str()), Some(*region));
        }
        for status in PlayStatus::ALL {
            assert_eq!(PlayStatus::parse_loose(status.as_str()), Some(*status));
        }
    }

    #[test]
    fn sort_title_moves_articles() {
        assert_eq!(sort_title("The Legend of Zelda"), "legend of zelda");
        assert_eq!(sort_title("A Hat in Time"), "hat in time");
        assert_eq!(sort_title("Super Mario Odyssey"), "super mario odyssey");
        assert_eq!(sort_title("The "), "the");
    }
}
