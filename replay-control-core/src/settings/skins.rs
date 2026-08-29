/// Color palette for a ReplayOS skin, mapping to CSS custom properties.
#[derive(Debug, Clone)]
pub struct SkinPalette {
    pub bg: &'static str,
    pub surface: &'static str,
    pub surface_hover: &'static str,
    pub border: &'static str,
    pub text: &'static str,
    pub text_secondary: &'static str,
    pub accent: &'static str,
    pub accent_hover: &'static str,
}

/// Stable RePlayOS skin-folder identity.
///
/// The value is intentionally open rather than an enum so a future custom
/// skin can flow through config and wire contracts without changing type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct SkinId(String);

impl Default for SkinId {
    fn default() -> Self {
        Self::new("replay")
    }
}

impl SkinId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().trim().to_ascii_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_supported(&self) -> bool {
        definition(self).is_some()
    }

    /// Parse current IDs and migrate RC's pre-1.8 numeric preference values.
    pub fn from_stored_value(value: &str) -> Option<Self> {
        if value.trim().is_empty() {
            return None;
        }
        if let Ok(index) = value.parse::<usize>() {
            return SKINS.get(index).map(|skin| Self::new(skin.id));
        }
        Some(Self::new(value))
    }
}

#[derive(Debug, Clone)]
pub struct SkinDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub palette: SkinPalette,
}

/// Bundled global skins in RePlayOS 1.8 display order.
pub const SKINS: [SkinDefinition; 11] = [
    SkinDefinition {
        id: "replay",
        name: "REPLAY",
        palette: SkinPalette {
            bg: "#101b32",
            surface: "#162541",
            surface_hover: "#1d3154",
            border: "#065ab5",
            text: "#edf4ff",
            text_secondary: "#94a8c7",
            accent: "#be1250",
            accent_hover: "#d52b68",
        },
    },
    SkinDefinition {
        id: "mega-tech",
        name: "MEGA TECH",
        palette: SkinPalette {
            bg: "#1b1e1c",
            surface: "#2d312d",
            surface_hover: "#3a3f3a",
            border: "#535d55",
            text: "#e8ebe8",
            text_secondary: "#a2aaa3",
            accent: "#ff004a",
            accent_hover: "#ff3370",
        },
    },
    SkinDefinition {
        id: "play-choice",
        name: "PLAY CHOICE",
        palette: SkinPalette {
            bg: "#003800",
            surface: "#005100",
            surface_hover: "#096809",
            border: "#4c864c",
            text: "#e7f0e5",
            text_secondary: "#a8c8a8",
            accent: "#ff4300",
            accent_hover: "#ff6b33",
        },
    },
    SkinDefinition {
        id: "astro",
        name: "ASTRO",
        palette: SkinPalette {
            bg: "#000000",
            surface: "#111111",
            surface_hover: "#1d1d1d",
            border: "#4e4e4e",
            text: "#e7eee9",
            text_secondary: "#97a79d",
            accent: "#00b543",
            accent_hover: "#19ca58",
        },
    },
    SkinDefinition {
        id: "super-video",
        name: "SUPER VIDEO",
        palette: SkinPalette {
            bg: "#020304",
            surface: "#111722",
            surface_hover: "#1c2636",
            border: "#2f54a4",
            text: "#e8edf5",
            text_secondary: "#9ba9bd",
            accent: "#892123",
            accent_hover: "#a92d30",
        },
    },
    SkinDefinition {
        id: "mvs",
        name: "MVS",
        palette: SkinPalette {
            bg: "#0f0f0f",
            surface: "#1a1a1a",
            surface_hover: "#292929",
            border: "#5e0a0a",
            text: "#eeeeee",
            text_secondary: "#a0a0a0",
            accent: "#e10202",
            accent_hover: "#ff2424",
        },
    },
    SkinDefinition {
        id: "rpg",
        name: "RPG",
        palette: SkinPalette {
            bg: "#2c292c",
            surface: "#4e4a4e",
            surface_hover: "#5d585d",
            border: "#8b542e",
            text: "#f2e9df",
            text_secondary: "#c5aa92",
            accent: "#6daa2c",
            accent_hover: "#83c43b",
        },
    },
    SkinDefinition {
        id: "fantasy",
        name: "FANTASY",
        palette: SkinPalette {
            bg: "#02023c",
            surface: "#07056d",
            surface_hover: "#08058b",
            border: "#909290",
            text: "#f0f1f3",
            text_secondary: "#b8b9c8",
            accent: "#be1250",
            accent_hover: "#d52b68",
        },
    },
    SkinDefinition {
        id: "simple-purple",
        name: "SIMPLE PURPLE",
        palette: SkinPalette {
            bg: "#0e0e0e",
            surface: "#171717",
            surface_hover: "#242424",
            border: "#474747",
            text: "#ededed",
            text_secondary: "#a0a0a0",
            accent: "#4c007f",
            accent_hover: "#6500a8",
        },
    },
    SkinDefinition {
        id: "metal",
        name: "METAL",
        palette: SkinPalette {
            bg: "#040404",
            surface: "#161616",
            surface_hover: "#252525",
            border: "#5d5c5c",
            text: "#dedede",
            text_secondary: "#929292",
            accent: "#7e2553",
            accent_hover: "#9e3a70",
        },
    },
    SkinDefinition {
        id: "unicolors",
        name: "UNICOLORS",
        palette: SkinPalette {
            bg: "#020001",
            surface: "#151313",
            surface_hover: "#242121",
            border: "#505050",
            text: "#eeeae0",
            text_secondary: "#b3a56b",
            accent: "#a49963",
            accent_hover: "#b9ad76",
        },
    },
];

pub fn definition(skin_id: &SkinId) -> Option<&'static SkinDefinition> {
    SKINS.iter().find(|skin| skin.id == skin_id.as_str())
}

pub fn palette(skin_id: &SkinId) -> &'static SkinPalette {
    definition(skin_id)
        .map(|skin| &skin.palette)
        .unwrap_or(&SKINS[0].palette)
}

/// Generate a CSS `<style>` block that overrides `:root` custom properties
/// for the given supported global skin ID.
///
/// Returns `None` for RePlay (which matches the static CSS) and unsupported
/// IDs, which currently use the RePlay fallback palette.
pub fn theme_css(skin_id: &SkinId) -> Option<String> {
    if skin_id.as_str() == "replay" || !skin_id.is_supported() {
        return None;
    }
    let p = palette(skin_id);
    Some(format!(
        ":root{{\
--bg:{bg};\
--surface:{surface};\
--surface-hover:{surface_hover};\
--border:{border};\
--text:{text};\
--text-secondary:{text_secondary};\
--accent:{accent};\
--accent-hover:{accent_hover};\
}}",
        bg = p.bg,
        surface = p.surface,
        surface_hover = p.surface_hover,
        border = p.border,
        text = p.text,
        text_secondary = p.text_secondary,
        accent = p.accent,
        accent_hover = p.accent_hover,
    ))
}

/// Return the `--bg` color for a skin ID (used for `<meta name="theme-color">`).
pub fn theme_color(skin_id: &SkinId) -> &'static str {
    palette(skin_id).bg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_skin_returns_no_css() {
        assert!(theme_css(&SkinId::new("replay")).is_none());
    }

    #[test]
    fn valid_skin_returns_css() {
        let css = theme_css(&SkinId::new("mega-tech")).unwrap();
        assert!(css.contains("--bg:"));
        assert!(css.contains("#ff004a")); // MEGA TECH accent
    }

    #[test]
    fn replayos_ids_round_trip() {
        for skin in &SKINS {
            let skin_id = SkinId::new(skin.id);
            assert_eq!(definition(&skin_id).map(|known| known.id), Some(skin.id));
        }
        assert_eq!(SkinId::new("ASTRO").as_str(), "astro");
        assert!(!SkinId::new("midnight-arcade").is_supported());
    }

    #[test]
    fn skin_id_serializes_as_replayos_value() {
        let skin_id = SkinId::new("astro");
        assert_eq!(serde_json::to_string(&skin_id).unwrap(), "\"astro\"");
        assert_eq!(
            serde_json::from_str::<SkinId>("\"midnight-arcade\"").unwrap(),
            SkinId::new("midnight-arcade")
        );
    }

    #[test]
    fn stored_numeric_values_migrate_to_ids() {
        assert_eq!(SkinId::from_stored_value("0"), Some(SkinId::new("replay")));
        assert_eq!(SkinId::from_stored_value("3"), Some(SkinId::new("astro")));
        assert_eq!(
            SkinId::from_stored_value("10"),
            Some(SkinId::new("unicolors"))
        );
        assert_eq!(SkinId::from_stored_value("11"), None);
        assert_eq!(
            SkinId::from_stored_value("midnight-arcade"),
            Some(SkinId::new("midnight-arcade"))
        );
    }

    #[test]
    fn unsupported_skin_uses_replay_palette() {
        let custom = SkinId::new("midnight-arcade");
        assert!(!custom.is_supported());
        assert_eq!(palette(&custom).bg, palette(&SkinId::default()).bg);
        assert_eq!(theme_color(&custom), theme_color(&SkinId::default()));
        assert!(theme_css(&custom).is_none());
    }

    #[test]
    fn all_palettes_have_valid_hex_colors() {
        for skin in &SKINS {
            let palette = &skin.palette;
            for (name, color) in [
                ("bg", palette.bg),
                ("surface", palette.surface),
                ("surface_hover", palette.surface_hover),
                ("border", palette.border),
                ("text", palette.text),
                ("text_secondary", palette.text_secondary),
                ("accent", palette.accent),
                ("accent_hover", palette.accent_hover),
            ] {
                assert!(
                    color.starts_with('#') && (color.len() == 7 || color.len() == 4),
                    "Skin {} has invalid {name} color: {color}",
                    skin.id,
                );
            }
        }
    }
}
