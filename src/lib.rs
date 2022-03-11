use std::f32::consts::PI;

use glam::Vec3;
use image::{ImageBuffer, Rgb};

fn get_cubemap_face_normals() -> [[Vec3; 3]; 6] {
    [
        [
            // +x
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ],
        [
            // -x
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
        ],
        [
            // +y
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
        [
            // -y
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, -1.0, 0.0),
        ],
        [
            // +z
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
        [
            // -z
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
        ],
    ]
}

/// Returns spherical harmonics for input cube map.
/// Input should be 6 square images in the order: +x, -x, +y, -y, +z, -z
pub fn process(faces: &[ImageBuffer<Rgb<f32>, Vec<f32>>]) -> anyhow::Result<[Vec3; 9]> {
    if faces.len() != 6 {
        anyhow::bail!("Expected 6 faces")
    }
    let size = faces[0].width();
    let mut cube_map_vecs = Vec::new();
    let sizef = size as f32;
    let cubemap_face_normals = get_cubemap_face_normals();

    // Forsyth's weights
    let weight1 = 4.0 / 17.0;
    let weight2 = 8.0 / 17.0;
    let weight3 = 5.0 / 68.0;
    let weight4 = 15.0 / 17.0;
    let weight5 = 15.0 / 68.0;

    for (idx, face) in faces.iter().enumerate() {
        if face.width() != face.height() {
            anyhow::bail!("Expected face width and height to match")
        }
        let mut face_vecs = Vec::new();
        for v in 0..size {
            for u in 0..size {
                let fu = (2.0 * u as f32 / (sizef - 1.0)) - 1.0;
                let fv = (2.0 * v as f32 / (sizef - 1.0)) - 1.0;

                let vec_x = cubemap_face_normals[idx][0] * fu;
                let vec_y = cubemap_face_normals[idx][1] * fv;
                let vec_z = cubemap_face_normals[idx][2];

                face_vecs.push((vec_x + vec_y + vec_z).normalize())
            }
        }
        cube_map_vecs.push(face_vecs)
    }

    let mut sh = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];
    let mut weight_accum = 0.0;

    for (idx, face) in faces.iter().enumerate() {
        for y in 0..size {
            for x in 0..size {
                let mut color = Vec3::from(face.get_pixel(x, y).0);

                let tex_v = cube_map_vecs[idx][(y * size + x) as usize];

                let weight = solid_angle(x as f32, y as f32, sizef);

                color *= weight;

                sh[0] += color * weight1;

                sh[1] += color * weight2 * tex_v.y;
                sh[2] += color * weight2 * tex_v.z;
                sh[3] += color * weight2 * tex_v.x;

                sh[4] += color * weight3 * tex_v.y * tex_v.x;
                sh[5] += color * weight3 * tex_v.y * tex_v.z;
                sh[6] += color * weight3 * (3.0 * tex_v.z * tex_v.z - 1.0);
                sh[7] += color * weight4 * tex_v.z * tex_v.x;
                sh[8] += color * weight5 * (tex_v.x * tex_v.x - tex_v.y * tex_v.y);

                weight_accum += weight * 3.0;
            }
        }
    }

    for n in sh.iter_mut() {
        *n *= 4.0 * PI / weight_accum;
    }

    Ok(sh)
}

// Explanation: https://www.rorydriscoll.com/2012/01/15/cubemap-texel-solid-angle/
fn solid_angle(au: f32, av: f32, size: f32) -> f32 {
    //scale up to [-1, 1] range (inclusive), offset by 0.5 to point to texel center.
    let u = (2.0 * (au + 0.5) / size) - 1.0;
    let v = (2.0 * (av + 0.5) / size) - 1.0;

    let inv_size = 1.0 / size;

    // U and V are the -1..1 texture coordinate on the current face.
    // get projected area for this texel
    let x0 = u - inv_size;
    let y0 = v - inv_size;
    let x1 = u + inv_size;
    let y1 = v + inv_size;
    let angle =
        area_element(x0, y0) - area_element(x0, y1) - area_element(x1, y0) + area_element(x1, y1);

    return angle;
}

fn area_element(x: f32, y: f32) -> f32 {
    (x * y).atan2((x * x + y * y + 1.0).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    use image::io::Reader as ImageReader;

    #[test]
    fn test_sh() {
        let img = ImageReader::open("example_cube.png")
            .unwrap()
            .decode()
            .unwrap()
            .into_rgb32f();
        let width = img.width();
        let mut faces = Vec::new();
        for i in 0..6 {
            faces.push(
                image::imageops::crop_imm(&img, 0, i as u32 * width, width, width).to_image(),
            );
        }
        let sh = process(&faces).unwrap();
        assert_eq!(
            sh,
            [
                Vec3::new(0.48503733, 0.48316428, 0.32759333),
                Vec3::new(-0.26024902, -0.26002043, 0.26548815),
                Vec3::new(0.2476333, 0.24728885, 0.24757174),
                Vec3::new(-0.26511002, 0.2617326, -0.0001285886),
                Vec3::new(-3.3611246e-5, -3.8461396e-5, 1.5085657e-5),
                Vec3::new(8.057594e-5, 7.771898e-5, 1.5798581e-5),
                Vec3::new(0.0044354284, 0.0049572727, 0.030710248),
                Vec3::new(0.00032010113, 0.00019238573, 0.00054129824),
                Vec3::new(0.002222224, 0.000540525, -0.08147548)
            ]
        );
    }
}
