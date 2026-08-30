#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    #[allow(dead_code)]
    pub fn to_slint_color(self) -> slint::Color {
        slint::Color::from_argb_u8(self.a, self.r, self.g, self.b)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundScene {
    pub canvas: RgbaColor,
    pub wash_primary: RgbaColor,
    pub wash_secondary: RgbaColor,
    pub text_primary: RgbaColor,
    pub text_muted: RgbaColor,
    pub accent: RgbaColor,
    pub variant: u8,
}

const DAY_SCENES: [BackgroundScene; 4] = [
    BackgroundScene {
        canvas: rgba(226, 219, 204, 255),
        wash_primary: rgba(173, 154, 123, 56),
        wash_secondary: rgba(121, 142, 131, 32),
        text_primary: rgba(70, 60, 47, 255),
        text_muted: rgba(111, 102, 87, 255),
        accent: rgba(153, 124, 83, 255),
        variant: 0,
    },
    BackgroundScene {
        canvas: rgba(213, 223, 214, 255),
        wash_primary: rgba(124, 154, 138, 44),
        wash_secondary: rgba(186, 162, 130, 28),
        text_primary: rgba(56, 67, 58, 255),
        text_muted: rgba(88, 100, 92, 255),
        accent: rgba(132, 118, 89, 255),
        variant: 1,
    },
    BackgroundScene {
        canvas: rgba(219, 216, 228, 255),
        wash_primary: rgba(142, 133, 162, 48),
        wash_secondary: rgba(186, 171, 137, 30),
        text_primary: rgba(62, 58, 76, 255),
        text_muted: rgba(98, 92, 111, 255),
        accent: rgba(120, 111, 151, 255),
        variant: 2,
    },
    BackgroundScene {
        canvas: rgba(232, 221, 214, 255),
        wash_primary: rgba(171, 132, 117, 52),
        wash_secondary: rgba(128, 147, 155, 34),
        text_primary: rgba(77, 58, 49, 255),
        text_muted: rgba(118, 96, 88, 255),
        accent: rgba(156, 110, 89, 255),
        variant: 3,
    },
];

const NIGHT_SCENES: [BackgroundScene; 4] = [
    BackgroundScene {
        canvas: rgba(24, 27, 31, 255),
        wash_primary: rgba(148, 121, 82, 44),
        wash_secondary: rgba(86, 104, 112, 26),
        text_primary: rgba(227, 216, 194, 255),
        text_muted: rgba(180, 169, 152, 255),
        accent: rgba(179, 141, 96, 255),
        variant: 0,
    },
    BackgroundScene {
        canvas: rgba(22, 31, 28, 255),
        wash_primary: rgba(118, 142, 121, 38),
        wash_secondary: rgba(138, 118, 88, 24),
        text_primary: rgba(220, 226, 214, 255),
        text_muted: rgba(172, 182, 169, 255),
        accent: rgba(151, 136, 102, 255),
        variant: 1,
    },
    BackgroundScene {
        canvas: rgba(25, 25, 34, 255),
        wash_primary: rgba(120, 109, 147, 42),
        wash_secondary: rgba(153, 138, 94, 24),
        text_primary: rgba(224, 219, 235, 255),
        text_muted: rgba(174, 168, 190, 255),
        accent: rgba(142, 133, 173, 255),
        variant: 2,
    },
    BackgroundScene {
        canvas: rgba(32, 24, 24, 255),
        wash_primary: rgba(154, 109, 96, 40),
        wash_secondary: rgba(95, 112, 121, 28),
        text_primary: rgba(232, 219, 211, 255),
        text_muted: rgba(189, 172, 164, 255),
        accent: rgba(182, 129, 103, 255),
        variant: 3,
    },
];

pub fn background_for_date(local_date: &str, night_mode: bool) -> BackgroundScene {
    let index = (fnv1a_hash(local_date.as_bytes()) % DAY_SCENES.len() as u32) as usize;
    if night_mode {
        NIGHT_SCENES[index]
    } else {
        DAY_SCENES[index]
    }
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> RgbaColor {
    RgbaColor { r, g, b, a }
}

fn fnv1a_hash(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c9dc5_u32;
    for &byte in bytes {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn same_date_and_mode_produce_same_scene() {
        let first = background_for_date("2026-08-30", false);
        let second = background_for_date("2026-08-30", false);

        assert_eq!(first, second);
    }

    #[test]
    fn fixed_dates_cover_more_than_one_day_palette() {
        let dates = ["2026-08-30", "2026-08-31", "2026-09-01", "2026-09-02"];
        let canvases: BTreeSet<_> = dates
            .into_iter()
            .map(|date| background_for_date(date, false).canvas)
            .collect();
        let variants: BTreeSet<_> = dates
            .into_iter()
            .map(|date| background_for_date(date, false).variant)
            .collect();

        assert!(canvases.len() > 1);
        assert!(variants.len() > 1);
    }

    #[test]
    fn all_scene_colors_are_opaque_or_intentionally_translucent() {
        for (mode, scenes) in [("day", &DAY_SCENES), ("night", &NIGHT_SCENES)] {
            for (index, scene) in scenes.iter().enumerate() {
                assert_eq!(scene.canvas.a, 255, "{mode} scene {index} canvas");
                assert_eq!(
                    scene.text_primary.a, 255,
                    "{mode} scene {index} primary text"
                );
                assert_eq!(scene.text_muted.a, 255, "{mode} scene {index} muted text");
                assert_eq!(scene.accent.a, 255, "{mode} scene {index} accent");
                assert!(
                    (20..=96).contains(&scene.wash_primary.a),
                    "{mode} scene {index} primary wash"
                );
                assert!(
                    (20..=96).contains(&scene.wash_secondary.a),
                    "{mode} scene {index} secondary wash"
                );
            }
        }
    }

    #[test]
    fn night_canvas_is_darker_than_day_canvas() {
        for (index, (day, night)) in DAY_SCENES.iter().zip(NIGHT_SCENES.iter()).enumerate() {
            assert!(
                luminance(night.canvas) < luminance(day.canvas),
                "night scene {index} must be darker than its day pair"
            );
        }
    }

    #[test]
    fn variant_is_always_zero_through_three() {
        for date in ["2026-08-30", "2026-08-31", "2026-09-01", "2026-09-02"] {
            let scene = background_for_date(date, false);
            assert!((0..=3).contains(&scene.variant));
        }
    }

    fn luminance(color: RgbaColor) -> u32 {
        299 * color.r as u32 + 587 * color.g as u32 + 114 * color.b as u32
    }
}
