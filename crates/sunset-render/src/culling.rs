#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    pub fn corners(self) -> [[f32; 3]; 8] {
        let [x0, y0, z0] = self.min;
        let [x1, y1, z1] = self.max;
        [
            [x0, y0, z0],
            [x1, y0, z0],
            [x0, y1, z0],
            [x1, y1, z0],
            [x0, y0, z1],
            [x1, y0, z1],
            [x0, y1, z1],
            [x1, y1, z1],
        ]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub normal: [f32; 3],
    pub distance: f32,
}

impl Plane {
    pub fn signed_distance(self, point: [f32; 3]) -> f32 {
        self.normal[0] * point[0]
            + self.normal[1] * point[1]
            + self.normal[2] * point[2]
            + self.distance
    }

    pub fn positive_vertex(self, aabb: &Aabb) -> [f32; 3] {
        let mut vertex = [0.0f32; 3];
        for (axis, slot) in vertex.iter_mut().enumerate() {
            *slot = if self.normal[axis] >= 0.0 {
                aabb.max[axis]
            } else {
                aabb.min[axis]
            };
        }
        vertex
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn cull(&self, aabb: &Aabb) -> bool {
        self.planes
            .iter()
            .any(|plane| plane.signed_distance(plane.positive_vertex(aabb)) < 0.0)
    }

    pub fn from_view_projection(m: &[f32; 16]) -> Self {
        let row0 = [m[0], m[4], m[8], m[12]];
        let row1 = [m[1], m[5], m[9], m[13]];
        let row2 = [m[2], m[6], m[10], m[14]];
        let row3 = [m[3], m[7], m[11], m[15]];
        let sources = [
            (row0, 1.0f32),
            (row0, -1.0),
            (row1, 1.0),
            (row1, -1.0),
            (row2, 1.0),
            (row2, -1.0),
        ];
        let mut planes = [Plane {
            normal: [0.0; 3],
            distance: 0.0,
        }; 6];
        for (index, (row, sign)) in sources.into_iter().enumerate() {
            let coefficients = [
                row3[0] + sign * row[0],
                row3[1] + sign * row[1],
                row3[2] + sign * row[2],
                row3[3] + sign * row[3],
            ];
            let length = (coefficients[0] * coefficients[0]
                + coefficients[1] * coefficients[1]
                + coefficients[2] * coefficients[2])
                .sqrt();
            planes[index] = Plane {
                normal: [
                    coefficients[0] / length,
                    coefficients[1] / length,
                    coefficients[2] / length,
                ],
                distance: coefficients[3] / length,
            };
        }
        Self { planes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_view_projection_yields_clip_cube() {
        let frustum = Frustum::from_view_projection(&[
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]);
        let inside = Aabb::new([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
        assert!(!frustum.cull(&inside));
        assert!(frustum.cull(&Aabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0])));
        assert!(frustum.cull(&Aabb::new([-3.0, -1.0, -1.0], [-2.0, 0.0, 0.0])));
    }

    #[test]
    fn positive_vertex_matches_brute_force() {
        let plane = Plane {
            normal: [1.0, -2.0, 0.5],
            distance: 3.0,
        };
        let aabb = Aabb::new([-4.0, -1.0, 2.0], [5.0, 6.0, 7.0]);
        let vertex = plane.positive_vertex(&aabb);
        let expected = aabb
            .corners()
            .iter()
            .map(|corner| {
                plane.normal[0] * corner[0]
                    + plane.normal[1] * corner[1]
                    + plane.normal[2] * corner[2]
            })
            .fold(f32::MIN, f32::max);
        assert!(
            (plane.normal[0] * vertex[0]
                + plane.normal[1] * vertex[1]
                + plane.normal[2] * vertex[2]
                - expected)
                .abs()
                < 1e-4
        );
    }

    #[test]
    fn visibility_suite_has_no_false_culling() {
        let frustum = box_frustum(8.0);
        let section = |x: f32, y: f32, z: f32| {
            Aabb::new(
                [x * 16.0, y * 16.0, z * 16.0],
                [x * 16.0 + 16.0, y * 16.0 + 16.0, z * 16.0 + 16.0],
            )
        };
        assert!(!frustum.cull(&section(0.0, 0.0, 0.0)));
        assert!(frustum.cull(&section(10.0, 0.0, 0.0)));
        assert!(frustum.cull(&section(-11.0, 0.0, 0.0)));
        assert!(frustum.cull(&section(0.0, 10.0, 0.0)));
        assert!(frustum.cull(&section(0.0, 0.0, 10.0)));
        assert!(!frustum.cull(&Aabb::new([0.0, 0.0, 0.0], [24.0, 24.0, 24.0])));
    }

    #[test]
    fn property_culling_never_rejects_intersecting_aabb() {
        let frustum = box_frustum(8.0);
        let mut state = 0x0ddc_0ffe_e15e_beefu64;
        for _ in 0..10_000 {
            let min = [next_f32(&mut state) * 40.0 - 20.0; 3];
            let max = [
                min[0] + next_f32(&mut state) * 32.0,
                min[1] + next_f32(&mut state) * 32.0,
                min[2] + next_f32(&mut state) * 32.0,
            ];
            let aabb = Aabb::new(min, max);
            let culled = frustum.cull(&aabb);
            let intersects = aabb_intersects_box(&aabb, 8.0);
            assert!(!culled || !intersects, "false culling of {aabb:?}");
        }
    }

    fn box_frustum(half: f32) -> Frustum {
        Frustum {
            planes: [
                Plane {
                    normal: [1.0, 0.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [-1.0, 0.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, 1.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, -1.0, 0.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, 0.0, 1.0],
                    distance: half,
                },
                Plane {
                    normal: [0.0, 0.0, -1.0],
                    distance: half,
                },
            ],
        }
    }

    fn aabb_intersects_box(aabb: &Aabb, half: f32) -> bool {
        aabb.min[0] <= half
            && aabb.max[0] >= -half
            && aabb.min[1] <= half
            && aabb.max[1] >= -half
            && aabb.min[2] <= half
            && aabb.max[2] >= -half
    }

    fn next_f32(state: &mut u64) -> f32 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        (x as u32) as f32 / u32::MAX as f32
    }
}
