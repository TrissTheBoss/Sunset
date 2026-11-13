pub const MAX_LIGHT: u8 = 15;
pub const MAX_AO: u8 = 3;

pub fn light_brightness(light: u8) -> f32 {
    light.min(MAX_LIGHT) as f32 / MAX_LIGHT as f32
}

pub fn ao_factor(ao: u8) -> f32 {
    1.0 - ao.min(MAX_AO) as f32 / MAX_AO as f32
}

pub fn fog_factor(distance: f32, fog_start: f32, fog_end: f32) -> f32 {
    if fog_end <= fog_start {
        return 0.0;
    }
    ((distance - fog_start) / (fog_end - fog_start)).clamp(0.0, 1.0)
}

pub fn apply_tint(color: [f32; 3], tint: [f32; 3]) -> [f32; 3] {
    [color[0] * tint[0], color[1] * tint[1], color[2] * tint[2]]
}

#[derive(Clone, Copy, Debug)]
pub struct Fog {
    pub start: f32,
    pub end: f32,
    pub color: [f32; 3],
}

pub fn shade_color(
    texture: [f32; 3],
    light: u8,
    ao: u8,
    tint: [f32; 3],
    distance: f32,
    fog: Fog,
) -> [f32; 3] {
    let tinted = apply_tint(texture, tint);
    let brightness = light_brightness(light) * ao_factor(ao);
    let lit = [
        tinted[0] * brightness,
        tinted[1] * brightness,
        tinted[2] * brightness,
    ];
    let amount = fog_factor(distance, fog.start, fog.end);
    [
        lit[0] + (fog.color[0] - lit[0]) * amount,
        lit[1] + (fog.color[1] - lit[1]) * amount,
        lit[2] + (fog.color[2] - lit[2]) * amount,
    ]
}

pub fn face_uv(face: u8, position: [f32; 3]) -> [f32; 2] {
    match face {
        0 | 1 => [position[0], position[2]],
        2 | 3 => [position[0], position[1]],
        4 | 5 => [position[1], position[2]],
        _ => [0.0, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
    }

    #[test]
    fn light_brightness_is_linear() {
        close(light_brightness(0), 0.0);
        close(light_brightness(15), 1.0);
        close(light_brightness(7), 7.0 / 15.0);
    }

    #[test]
    fn ao_factor_covers_full_range() {
        close(ao_factor(0), 1.0);
        close(ao_factor(3), 0.0);
        close(ao_factor(1), 2.0 / 3.0);
    }

    #[test]
    fn fog_factor_is_linear_and_clamped() {
        close(fog_factor(10.0, 10.0, 100.0), 0.0);
        close(fog_factor(100.0, 10.0, 100.0), 1.0);
        close(fog_factor(55.0, 10.0, 100.0), 0.5);
        close(fog_factor(200.0, 10.0, 100.0), 1.0);
        close(fog_factor(0.0, 10.0, 100.0), 0.0);
    }

    #[test]
    fn fog_factor_is_zero_when_span_is_non_positive() {
        close(fog_factor(50.0, 100.0, 100.0), 0.0);
    }

    #[test]
    fn tint_is_multiplied_componentwise() {
        let color = [1.0, 0.5, 0.25];
        assert_eq!(apply_tint(color, [1.0, 1.0, 1.0]), color);
        assert_eq!(apply_tint(color, [0.5, 2.0, 1.0]), [0.5, 1.0, 0.25]);
    }

    #[test]
    fn shade_color_matches_analytic_endpoints() {
        let texture = [1.0, 1.0, 1.0];
        let tint = [1.0, 1.0, 1.0];
        let fog = Fog {
            start: 10.0,
            end: 100.0,
            color: [0.5, 0.5, 0.5],
        };
        assert_eq!(shade_color(texture, 15, 0, tint, 0.0, fog), [1.0, 1.0, 1.0]);
        assert_eq!(
            shade_color(texture, 15, 0, tint, 100.0, fog),
            [0.5, 0.5, 0.5]
        );
    }

    #[test]
    fn face_uv_selects_in_plane_axes() {
        let position = [1.0, 2.0, 3.0];
        assert_eq!(face_uv(0, position), [1.0, 3.0]);
        assert_eq!(face_uv(1, position), [1.0, 3.0]);
        assert_eq!(face_uv(2, position), [1.0, 2.0]);
        assert_eq!(face_uv(3, position), [1.0, 2.0]);
        assert_eq!(face_uv(4, position), [2.0, 3.0]);
        assert_eq!(face_uv(5, position), [2.0, 3.0]);
    }
}
