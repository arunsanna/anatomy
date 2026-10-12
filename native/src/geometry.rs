use crate::model::Part;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4],
    pub material: [f32; 4],
}
fn rotate(mut p: [f64; 3], r: [f64; 3]) -> [f64; 3] {
    let (s, c) = r[0].to_radians().sin_cos();
    p = [p[0], c * p[1] - s * p[2], s * p[1] + c * p[2]];
    let (s, c) = r[1].to_radians().sin_cos();
    p = [c * p[0] + s * p[2], p[1], -s * p[0] + c * p[2]];
    let (s, c) = r[2].to_radians().sin_cos();
    [c * p[0] - s * p[1], s * p[0] + c * p[1], p[2]]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = v.iter().map(|v| v * v).sum::<f64>().sqrt();
    v.map(|v| v / len)
}
fn emit(out: &mut Vec<Vertex>, p: &Part, pos: [f64; 3], normal: [f64; 3]) {
    let world = rotate(std::array::from_fn(|i| pos[i] * p.size[i]), p.rotation);
    let normal = normalize(rotate(
        std::array::from_fn(|i| normal[i] / p.size[i]),
        p.rotation,
    ));
    out.push(Vertex {
        position: [
            (world[0] + p.position[0]) as f32,
            (world[1] + p.position[1]) as f32,
            (world[2] + p.position[2]) as f32,
            1.,
        ],
        normal: [normal[0] as f32, normal[1] as f32, normal[2] as f32, 0.],
        color: p.color.map(|v| v as f32),
        material: [p.glow as f32, 0., 0., 0.],
    });
}
pub fn vertices(parts: &[Part], out: &mut Vec<Vertex>) {
    out.clear();
    for p in parts {
        match p.kind.as_str() {
            "box" => {
                for (axis, sign) in [(0, -1.), (0, 1.), (1, -1.), (1, 1.), (2, -1.), (2, 1.)] {
                    let a = (axis + 1) % 3;
                    let b = (axis + 2) % 3;
                    let corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
                    for j in [0, 1, 2, 0, 2, 3] {
                        let mut pos = [0.; 3];
                        pos[axis] = sign * 0.5;
                        pos[a] = corners[j][0];
                        pos[b] = corners[j][1];
                        let mut n = [0.; 3];
                        n[axis] = sign;
                        emit(out, p, pos, n);
                    }
                }
            }
            "sphere" => {
                let at = |lat: usize, lon: usize| {
                    let theta = std::f64::consts::PI * lat as f64 / 16.;
                    let phi = std::f64::consts::TAU * lon as f64 / 24.;
                    [
                        theta.sin() * phi.cos(),
                        theta.cos(),
                        theta.sin() * phi.sin(),
                    ]
                };
                for lat in 0..16 {
                    for lon in 0..24 {
                        for (a, b) in [
                            (lat, lon),
                            (lat + 1, lon),
                            (lat + 1, lon + 1),
                            (lat, lon),
                            (lat + 1, lon + 1),
                            (lat, lon + 1),
                        ] {
                            let n = at(a, b);
                            emit(out, p, n.map(|v| v * 0.5), n);
                        }
                    }
                }
            }
            "cylinder" => {
                for i in 0..32 {
                    let a = std::f64::consts::TAU * i as f64 / 32.;
                    let b = std::f64::consts::TAU * (i + 1) as f64 / 32.;
                    let na = [a.cos(), 0., a.sin()];
                    let nb = [b.cos(), 0., b.sin()];
                    for (n, y) in [
                        (na, -0.5),
                        (nb, -0.5),
                        (nb, 0.5),
                        (na, -0.5),
                        (nb, 0.5),
                        (na, 0.5),
                    ] {
                        emit(out, p, [n[0] * 0.5, y, n[2] * 0.5], n);
                    }
                    for y in [-0.5, 0.5] {
                        for pos in [
                            [0., y, 0.],
                            [na[0] * 0.5, y, na[2] * 0.5],
                            [nb[0] * 0.5, y, nb[2] * 0.5],
                        ] {
                            emit(out, p, pos, [0., y.signum(), 0.]);
                        }
                    }
                }
            }
            _ => unreachable!("validated part"),
        }
    }
}
#[repr(C)]
pub struct CameraUniform {
    pub eye: [f32; 4],
    pub right: [f32; 4],
    pub up: [f32; 4],
    pub forward: [f32; 4],
    pub projection: [f32; 4],
}
pub fn camera(yaw: f64, pitch: f64, distance: f64, aspect: f64) -> CameraUniform {
    let (sy, cy) = yaw.to_radians().sin_cos();
    let (sp, cp) = pitch.to_radians().sin_cos();
    CameraUniform {
        eye: [
            (sy * cp * distance) as f32,
            (sp * distance) as f32,
            (cy * cp * distance) as f32,
            0.,
        ],
        right: [cy as f32, 0., -sy as f32, 0.],
        up: [(-sy * sp) as f32, cp as f32, (-cy * sp) as f32, 0.],
        forward: [(-sy * cp) as f32, -sp as f32, (-cy * cp) as f32, 0.],
        projection: [
            (1. / (45_f64.to_radians() / 2.).tan()) as f32,
            aspect as f32,
            0.1,
            100.,
        ],
    }
}
