#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub m: [f32; 16],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        m: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let mut out = [0.0f32; 16];
        for column in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[k * 4 + row] * other.m[column * 4 + k];
                }
                out[column * 4 + row] = sum;
            }
        }
        Mat4 { m: out }
    }

    pub fn transform_point(&self, point: [f32; 3]) -> [f32; 4] {
        let [x, y, z] = point;
        [
            self.m[0] * x + self.m[4] * y + self.m[8] * z + self.m[12],
            self.m[1] * x + self.m[5] * y + self.m[9] * z + self.m[13],
            self.m[2] * x + self.m[6] * y + self.m[10] * z + self.m[14],
            self.m[3] * x + self.m[7] * y + self.m[11] * z + self.m[15],
        ]
    }

    pub fn as_array(&self) -> [f32; 16] {
        self.m
    }
}

pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fovy * 0.5).tan();
    let mut m = [0.0f32; 16];
    m[0] = f / aspect;
    m[5] = f;
    m[10] = (far + near) / (near - far);
    m[11] = -1.0;
    m[14] = (2.0 * far * near) / (near - far);
    Mat4 { m }
}

pub fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> Mat4 {
    let forward = normalize(sub(center, eye));
    let side = normalize(cross(forward, up));
    let up = cross(side, forward);
    Mat4 {
        m: [
            side[0],
            up[0],
            -forward[0],
            0.0,
            side[1],
            up[1],
            -forward[1],
            0.0,
            side[2],
            up[2],
            -forward[2],
            0.0,
            -dot(side, eye),
            -dot(up, eye),
            dot(forward, eye),
            1.0,
        ],
    }
}

pub fn view_projection(
    eye: [f32; 3],
    center: [f32; 3],
    up: [f32; 3],
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    perspective(fovy, aspect, near, far).mul(&look_at(eye, center, up))
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(a: [f32; 3]) -> [f32; 3] {
    let length = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if length == 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [a[0] / length, a[1] / length, a[2] / length]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn look_at_origin_looking_negative_z_is_identity() {
        assert_eq!(
            look_at([0.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
            Mat4::IDENTITY
        );
    }

    #[test]
    fn perspective_projects_near_and_far() {
        let proj = perspective(std::f32::consts::FRAC_PI_2, 1.0, 1.0, 100.0);
        let near = proj.transform_point([0.0, 0.0, -1.0]);
        assert!((near[2] / near[3] + 1.0).abs() < 1e-5);
        let far = proj.transform_point([0.0, 0.0, -100.0]);
        assert!((far[2] / far[3] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn identity_is_multiplicative_identity() {
        let proj = perspective(std::f32::consts::FRAC_PI_2, 1.0, 1.0, 100.0);
        assert_eq!(Mat4::IDENTITY.mul(&proj), proj);
        assert_eq!(proj.mul(&Mat4::IDENTITY), proj);
    }

    #[test]
    fn view_projection_keeps_front_points_ahead() {
        let vp = view_projection(
            [0.0, 70.0, 0.0],
            [0.0, 70.0, -1.0],
            [0.0, 1.0, 0.0],
            std::f32::consts::FRAC_PI_2,
            1.0,
            0.1,
            300.0,
        );
        let p = vp.transform_point([0.0, 70.0, -10.0]);
        assert!(p[3] > 0.0);
    }
}
