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
    pub canvas_text_primary: RgbaColor,
    pub canvas_text_muted: RgbaColor,
    pub text_primary: RgbaColor,
    pub text_muted: RgbaColor,
    pub accent: RgbaColor,
    pub variant: u8,
}

const DAY_SCENES: [BackgroundScene; 4] = [
    BackgroundScene {
        canvas: rgba(24, 36, 50, 255),
        surface: rgba(215, 199, 166, 255),
        rule: rgba(96, 112, 131, 255),
        canvas_text_primary: rgba(240, 230, 210, 255),
        canvas_text_muted: rgba(184, 190, 196, 255),
        text_primary: rgba(31, 37, 40, 255),
        text_muted: rgba(93, 85, 72, 255),
        accent: rgba(196, 119, 46, 255),
        variant: 0,
    },
    BackgroundScene {
        canvas: rgba(24, 36, 50, 255),
        surface: rgba(215, 199, 166, 255),
        rule: rgba(96, 112, 131, 255),
        canvas_text_primary: rgba(240, 230, 210, 255),
        canvas_text_muted: rgba(184, 190, 196, 255),
        text_primary: rgba(31, 37, 40, 255),
        text_muted: rgba(93, 85, 72, 255),
        accent: rgba(196, 119, 46, 255),
        variant: 1,
    },
    BackgroundScene {
        canvas: rgba(24, 36, 50, 255),
        surface: rgba(215, 199, 166, 255),
        rule: rgba(96, 112, 131, 255),
        canvas_text_primary: rgba(240, 230, 210, 255),
        canvas_text_muted: rgba(184, 190, 196, 255),
        text_primary: rgba(31, 37, 40, 255),
        text_muted: rgba(93, 85, 72, 255),
        accent: rgba(196, 119, 46, 255),
        variant: 2,
    },
    BackgroundScene {
        canvas: rgba(24, 36, 50, 255),
        surface: rgba(215, 199, 166, 255),
        rule: rgba(96, 112, 131, 255),
        canvas_text_primary: rgba(240, 230, 210, 255),
        canvas_text_muted: rgba(184, 190, 196, 255),
        text_primary: rgba(31, 37, 40, 255),
        text_muted: rgba(93, 85, 72, 255),
        accent: rgba(196, 119, 46, 255),
        variant: 3,
    },
];

const NIGHT_SCENES: [BackgroundScene; 4] = [
    BackgroundScene {
        canvas: rgba(22, 24, 27, 255),
        surface: rgba(32, 34, 37, 255),
        rule: rgba(79, 88, 100, 255),
        canvas_text_primary: rgba(235, 228, 214, 255),
        canvas_text_muted: rgba(165, 158, 146, 255),
        text_primary: rgba(235, 228, 214, 255),
        text_muted: rgba(165, 158, 146, 255),
        accent: rgba(184, 143, 96, 255),
        variant: 0,
    },
    BackgroundScene {
        canvas: rgba(20, 27, 25, 255),
        surface: rgba(29, 37, 34, 255),
        rule: rgba(79, 88, 100, 255),
        canvas_text_primary: rgba(229, 232, 221, 255),
        canvas_text_muted: rgba(159, 168, 157, 255),
        text_primary: rgba(229, 232, 221, 255),
        text_muted: rgba(159, 168, 157, 255),
        accent: rgba(171, 142, 96, 255),
        variant: 1,
    },
    BackgroundScene {
        canvas: rgba(23, 24, 26, 255),
        surface: rgba(34, 35, 37, 255),
        rule: rgba(79, 88, 100, 255),
        canvas_text_primary: rgba(235, 229, 216, 255),
        canvas_text_muted: rgba(166, 159, 147, 255),
        text_primary: rgba(235, 229, 216, 255),
        text_muted: rgba(166, 159, 147, 255),
        accent: rgba(182, 141, 93, 255),
        variant: 2,
    },
    BackgroundScene {
        canvas: rgba(28, 22, 22, 255),
        surface: rgba(39, 31, 30, 255),
        rule: rgba(79, 88, 100, 255),
        canvas_text_primary: rgba(238, 227, 219, 255),
        canvas_text_muted: rgba(173, 158, 150, 255),
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
    fn day_dates_keep_one_panel_calibrated_palette() {
        let dates = ["2026-08-30", "2026-08-31", "2026-09-01", "2026-09-02"];
        let palettes: BTreeSet<_> = dates
            .into_iter()
            .map(|date| {
                let scene = background_for_date(date, false);
                (
                    scene.canvas,
                    scene.surface,
                    scene.rule,
                    scene.canvas_text_primary,
                    scene.canvas_text_muted,
                    scene.text_primary,
                    scene.text_muted,
                    scene.accent,
                )
            })
            .collect();
        let variants: BTreeSet<_> = dates
            .into_iter()
            .map(|date| background_for_date(date, false).variant)
            .collect();

        assert_eq!(
            palettes.len(),
            1,
            "physical-screen calibration must stay stable"
        );
        assert!(variants.len() > 1);
    }

    #[test]
    fn all_scene_colors_are_opaque_for_predictable_panel_output() {
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
                assert_eq!(scene.rule.a, 255, "{mode} scene {index} editorial rule");
                assert_eq!(
                    scene.canvas_text_primary.a, 255,
                    "{mode} scene {index} canvas primary text"
                );
                assert_eq!(
                    scene.canvas_text_muted.a, 255,
                    "{mode} scene {index} canvas muted text"
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
    fn day_palette_survives_a_low_dynamic_range_panel() {
        for (index, scene) in DAY_SCENES.iter().enumerate() {
            let canvas = luminance(scene.canvas);
            let surface = luminance(scene.surface);

            assert!(canvas <= 60_000, "day scene {index} canvas must be deep");
            assert!(
                (150_000..=225_000).contains(&surface),
                "day scene {index} paper must stay below near-white clipping"
            );
            assert!(
                surface.abs_diff(canvas) >= 110_000,
                "day scene {index} needs coarse luminance separation"
            );
        }
    }

    #[test]
    fn day_clock_and_paper_keep_independent_contrast() {
        for (index, scene) in DAY_SCENES.iter().enumerate() {
            assert!(
                luminance(scene.canvas_text_primary).abs_diff(luminance(scene.canvas)) >= 150_000,
                "day scene {index} clock must stay bright on the ink canvas"
            );
            assert!(
                luminance(scene.canvas_text_muted).abs_diff(luminance(scene.canvas)) >= 75_000,
                "day scene {index} canvas metadata must stay visible"
            );
            assert!(
                luminance(scene.text_primary).abs_diff(luminance(scene.surface)) >= 120_000,
                "day scene {index} reading text must stay dark on paper"
            );
            assert_ne!(scene.canvas_text_primary, scene.text_primary);
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
    fn editorial_rule_does_not_depend_on_panel_alpha_detail() {
        for (mode, scenes) in [("day", &DAY_SCENES), ("night", &NIGHT_SCENES)] {
            for (index, scene) in scenes.iter().enumerate() {
                assert_eq!(scene.rule.a, 255, "{mode} scene {index} editorial rule");
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
