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
    pub surface: RgbaColor,
    pub rule: RgbaColor,
    pub text_primary: RgbaColor,
    pub text_muted: RgbaColor,
    pub accent: RgbaColor,
    pub variant: u8,
}

const DAY_SCENES: [BackgroundScene; 4] = [
    BackgroundScene {
        canvas: rgba(232, 228, 219, 255),
        surface: rgba(247, 244, 238, 255),
        rule: rgba(145, 118, 88, 80),
        text_primary: rgba(42, 41, 38, 255),
        text_muted: rgba(105, 101, 94, 255),
        accent: rgba(145, 108, 70, 255),
        variant: 0,
    },
    BackgroundScene {
        canvas: rgba(229, 232, 225, 255),
        surface: rgba(245, 246, 241, 255),
        rule: rgba(125, 130, 108, 72),
        text_primary: rgba(39, 44, 40, 255),
        text_muted: rgba(96, 104, 97, 255),
        accent: rgba(130, 104, 72, 255),
        variant: 1,
    },
    BackgroundScene {
        canvas: rgba(235, 231, 222, 255),
        surface: rgba(249, 246, 239, 255),
        rule: rgba(134, 117, 88, 72),
        text_primary: rgba(42, 41, 38, 255),
        text_muted: rgba(107, 101, 91, 255),
        accent: rgba(143, 105, 68, 255),
        variant: 2,
    },
    BackgroundScene {
        canvas: rgba(235, 228, 224, 255),
        surface: rgba(249, 244, 240, 255),
        rule: rgba(155, 109, 90, 72),
        text_primary: rgba(48, 40, 37, 255),
        text_muted: rgba(112, 96, 90, 255),
        accent: rgba(151, 96, 74, 255),
        variant: 3,
    },
];

const NIGHT_SCENES: [BackgroundScene; 4] = [
    BackgroundScene {
        canvas: rgba(22, 24, 27, 255),
        surface: rgba(32, 34, 37, 255),
        rule: rgba(172, 139, 99, 72),
        text_primary: rgba(235, 228, 214, 255),
        text_muted: rgba(165, 158, 146, 255),
        accent: rgba(184, 143, 96, 255),
        variant: 0,
    },
    BackgroundScene {
        canvas: rgba(20, 27, 25, 255),
        surface: rgba(29, 37, 34, 255),
        rule: rgba(146, 145, 110, 64),
        text_primary: rgba(229, 232, 221, 255),
        text_muted: rgba(159, 168, 157, 255),
        accent: rgba(171, 142, 96, 255),
        variant: 1,
    },
    BackgroundScene {
        canvas: rgba(23, 24, 26, 255),
        surface: rgba(34, 35, 37, 255),
        rule: rgba(168, 139, 96, 64),
        text_primary: rgba(235, 229, 216, 255),
        text_muted: rgba(166, 159, 147, 255),
        accent: rgba(182, 141, 93, 255),
        variant: 2,
    },
    BackgroundScene {
        canvas: rgba(28, 22, 22, 255),
        surface: rgba(39, 31, 30, 255),
        rule: rgba(177, 125, 105, 64),
        text_primary: rgba(238, 227, 219, 255),
        text_muted: rgba(173, 158, 150, 255),
        accent: rgba(192, 137, 107, 255),
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
                assert_eq!(scene.surface.a, 255, "{mode} scene {index} surface");
                assert!(
                    (48..=112).contains(&scene.rule.a),
                    "{mode} scene {index} editorial rule"
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
    fn editorial_surface_is_opaque_and_visibly_separate_from_canvas() {
        for (mode, scenes) in [("day", &DAY_SCENES), ("night", &NIGHT_SCENES)] {
            for (index, scene) in scenes.iter().enumerate() {
                assert_eq!(scene.surface.a, 255, "{mode} scene {index} surface");
                assert!(
                    luminance(scene.surface) > luminance(scene.canvas),
                    "{mode} scene {index} surface must lift above the canvas"
                );
            }
        }
    }

    #[test]
    fn editorial_surface_keeps_high_reading_contrast() {
        for (mode, scenes) in [("day", &DAY_SCENES), ("night", &NIGHT_SCENES)] {
            for (index, scene) in scenes.iter().enumerate() {
                let difference = luminance(scene.text_primary).abs_diff(luminance(scene.surface));
                assert!(
                    difference >= 110_000,
                    "{mode} scene {index} text/surface contrast"
                );
            }
        }
    }

    #[test]
    fn editorial_rule_stays_translucent() {
        for (mode, scenes) in [("day", &DAY_SCENES), ("night", &NIGHT_SCENES)] {
            for (index, scene) in scenes.iter().enumerate() {
                assert!(
                    (48..=112).contains(&scene.rule.a),
                    "{mode} scene {index} editorial rule"
                );
            }
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
