use gl::types::*;
use std::mem::size_of;
use glam::{Vec3, Mat4, EulerRot};

use crate::things::{Transform, Motive, Mesh, Model};

#[derive(Default)]
pub struct LegSeg {
    transform: Transform,
    length: f32,
}

pub struct LegAttach {
    offset: Vec3,
    base_yaw: f32,
}

#[derive(Default)]
pub struct Leg {
    coxa: LegSeg,
    femur: LegSeg,
    tibia: LegSeg,
    tarsus: LegSeg,

    rest_position: Vec3,
    current_target: Vec3,
    step_start: Vec3,
    step_end: Vec3,
    step_offset: f32,
    step_time: f32,
    step_cool: f32,
}

pub struct Spider {
    body_model: Model,
    legseg_model: Model,
    pub transform: Transform,
    pub motive: Motive,
    legs: [Leg; 8],
}

impl Spider {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let transform = Transform::new(x, y, z, 0.0, 0.0, 0.0);
        let motive = Motive::new();

        let body_matrix = transform.to_matrix();

        let attatchments: [LegAttach; 8] = Self::create_legattach();
        let leg_scale: [f32; 8] = Self::leg_scale();

        let legs: [Leg; 8] = std::array::from_fn(|i| {
            let att = &attatchments[i];
            let scale = leg_scale[i];
            let mut leg = Leg::default();

            let outward_dir = Vec3::new(att.offset.x, 0.0, att.offset.z).normalize();
            let aligned_yaw = outward_dir.x.atan2(-outward_dir.z);

            leg.coxa.transform.x = att.offset.x;
            leg.coxa.transform.y = att.offset.y;
            leg.coxa.transform.z = att.offset.z;
            leg.coxa.transform.yaw = aligned_yaw;
            leg.coxa.transform.pit = 20.0_f32.to_radians();
            leg.coxa.length = 6.0 * scale;

            leg.femur.transform.x = 0.0;
            leg.femur.transform.y = -leg.coxa.length;
            leg.femur.transform.z = 0.0;
            leg.femur.transform.pit = 40.0_f32.to_radians();
            leg.femur.length = 6.0 * scale;

            leg.tibia.transform.y = -leg.femur.length;
            leg.tibia.transform.pit = (-70.0_f32).to_radians();
            leg.tibia.length = 8.0 * scale;

            leg.tarsus.transform.y = -leg.tibia.length;
            leg.tarsus.length = 1.5 * scale;

            let leg_reach = leg.coxa.length + leg.femur.length
                + leg.tibia.length + leg.tarsus.length;
            let horiz_dist = leg_reach * 0.6;
            let drop_dist = leg_reach * 0.4;

            leg.rest_position = att.offset
                + outward_dir * horiz_dist
                + Vec3::new(0.0, -drop_dist, 0.0);

            let world_rest = body_matrix.transform_point3(leg.rest_position);
            leg.current_target = world_rest;
            leg.step_time = 1.0;
            leg.step_offset = if i % 2 == 0 { 0.0 } else { 0.15 };
            leg.step_start = world_rest;
            leg.step_end = world_rest;

            leg
        });

        Spider {
            transform: transform,
            motive: motive,
            body_model: Model::new(Self::create_body_mesh()),
            legseg_model: Model::new(Self::create_legseg_mesh()),
            legs,
        }
    }

    pub fn draw(&self, model_loc: GLint) {
        let body_mat: Mat4 = self.transform.to_matrix();
        unsafe { gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, body_mat.as_ref().as_ptr()) };
        self.body_model.draw();

        for leg in &self.legs {
            let coxa_mat = body_mat * leg.coxa.transform.to_matrix();
            let femur_mat = coxa_mat * leg.femur.transform.to_matrix();
            let tibia_mat = femur_mat * leg.tibia.transform.to_matrix();
            let tarsus_mat = tibia_mat * leg.tarsus.transform.to_matrix();

            struct SegDraw {
                mat: *const Mat4,
                length: f32,
            }
            let segs: [SegDraw; 4] = [
                SegDraw {mat: &coxa_mat, length: leg.coxa.length},
                SegDraw {mat: &femur_mat, length: leg.femur.length},
                SegDraw {mat: &tibia_mat, length: leg.tibia.length},
                SegDraw {mat: &tarsus_mat, length: leg.tarsus.length},
            ];

            for seg in &segs {
                unsafe {
                    let final_mat = *seg.mat * Mat4::from_scale(Vec3::new(1.0, seg.length, 1.0));
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, final_mat.as_ref().as_ptr());
                }
                self.legseg_model.draw();
            }
        }
    }

    pub fn update() {

    }

    pub fn behave() {

    }

    fn create_legattach() -> [LegAttach; 8] {
        [
            LegAttach { offset: Vec3::new(0.25, -0.5, 0.3), base_yaw: (-60.0_f32).to_radians() },
            LegAttach { offset: Vec3::new(0.25, -0.5, 0.0), base_yaw: (-30.0_f32).to_radians() },
            LegAttach { offset: Vec3::new(0.25, -0.5, -0.2), base_yaw: (30.0_f32).to_radians() },
            LegAttach { offset: Vec3::new(0.25, -0.5, -0.75), base_yaw: (60.0_f32).to_radians() },
            LegAttach { offset: Vec3::new(-0.25, -0.5, 0.3), base_yaw: (-120.0_f32).to_radians() },
            LegAttach { offset: Vec3::new(-0.25, -0.5, 0.0), base_yaw: (-150.0_f32).to_radians() },
            LegAttach { offset: Vec3::new(-0.25, -0.5, -0.2), base_yaw: (150.0_f32).to_radians() },
            LegAttach { offset: Vec3::new(-0.25, -0.5, -0.75), base_yaw: (120.0_f32).to_radians() },
        ]
    }

    fn leg_scale() -> [f32; 8] {
        [
            0.8, 1.0, 1.1, 1.0,
            0.8, 1.0, 1.1, 1.0,
        ]
    }

    fn create_body_mesh() -> Mesh {
        let vertices = vec![
            -0.1875, -0.2025, -1.25,  // front square
            0.1875, -0.2025, -1.25,
            -0.1875, -0.6975, -1.25,
            0.1875, -0.6975, -1.25,
            -0.1875, -0.2025,  0.25,  // back square
            0.1875, -0.2025,  0.25,
            -0.1875, -0.6975,  0.25,
            0.1875, -0.6975,  0.25,
            -0.375,  -0.2025, -0.875, // left square
            -0.375,  -0.2025, -0.125,
            -0.375,  -0.6975, -0.875,
            -0.375,  -0.6975, -0.125,
            0.375,  -0.2025, -0.875, // right square
            0.375,  -0.2025, -0.125,
            0.375,  -0.6975, -0.875,
            0.375,  -0.6975, -0.125,
            -0.1875,  0.045,  -0.875, // top square
            0.1875,  0.045,  -0.875,
            -0.1875,  0.045,  -0.125,
            0.1875,  0.045,  -0.125,
            -0.1875, -0.945,  -0.875, // bottom square
            0.1875, -0.945,  -0.875,
            -0.1875, -0.945,  -0.125,
            0.1875, -0.945,  -0.125,

            -0.25,  0.25, -0.3125,  // front square
            0.25,  0.25, -0.3125,
            -0.25, -0.25, -0.3125,
            0.25, -0.25, -0.3125,
            -0.125, 0.125, 2.1875,  // back square
            0.125, 0.125, 2.1875,
            -0.125,-0.125, 2.1875,
            0.125,-0.125, 2.1875,
            -0.5,   0.25,  0.3125,  // left square
            -0.4,   0.2,   1.5625,
            -0.5,  -0.25,  0.3125,
            -0.5,  -0.25,  1.5625,
            0.5,   0.25,  0.3125,  // right square
            0.4,   0.2,   1.5625,
            0.5,  -0.25,  0.3125,
            0.5,  -0.25,  1.5625,
            -0.25,  0.5,   0.3125,  // top square
            0.25,  0.5,   0.3125,
            -0.2,   0.4,   1.5625,
            0.2,   0.4,   1.5625,
            -0.25, -0.5,   0.3125,  // bottom square
            0.25, -0.5,   0.3125,
            -0.25, -0.5,   1.5625,
            0.25, -0.5,   1.5625,
        ];

        let indices = vec![
            0, 1, 2, // squares
            2, 3, 1,
            4, 5, 6,
            6, 7, 5,
            8, 9, 10,
            10, 11, 9,
            12, 13, 14,
            14, 15, 13,
            16, 17, 18,
            18, 19, 17,
            20, 21, 22,
            22, 23, 21,
            0, 2, 8, // connecting squares
            2, 8, 10,
            1, 3, 12,
            3, 12, 14,
            9, 11, 6,
            4, 6, 9,
            13, 15, 5,
            15, 5, 7,
            16, 17, 0,
            0, 1, 17,
            18, 19, 4,
            19, 4, 5,
            16, 18, 8,
            18, 8, 9,
            17, 19, 12,
            19, 12, 13,
            20, 21, 2,
            2, 3, 21,
            22, 23, 6,
            6, 7, 23,
            20, 22, 10,
            10, 11, 22,
            21, 23, 14,
            14, 15, 23,
            0, 8, 16, // triangles front
            1, 12, 17,
            2, 10, 20,
            3, 14, 21,
            4, 9, 18, // triangles back
            5, 13, 19,
            6, 11, 22,
            7, 15, 23,

            24, 25, 26, // squares
            26, 27, 25,
            28, 29, 30,
            30, 31, 29,
            32, 33, 34,
            34, 35, 33,
            36, 37, 38,
            38, 39, 37,
            40, 41, 42,
            42, 43, 41,
            44, 45, 46,
            46, 47, 45,
            24, 26, 32, // connecting squares
            26, 32, 34,
            25, 27, 36,
            27, 36, 38,
            33, 35, 30,
            28, 30, 33,
            37, 39, 29,
            39, 29, 31,
            40, 41, 24,
            24, 25, 41,
            42, 43, 28,
            43, 28, 29,
            40, 42, 32,
            42, 32, 33,
            41, 43, 36,
            43, 36, 37,
            44, 45, 26,
            26, 27, 45,
            46, 47, 30,
            30, 31, 47,
            44, 46, 34,
            34, 35, 46,
            45, 47, 38,
            38, 39, 47,
            24, 32, 40, // triangles front
            25, 36, 41,
            26, 34, 44,
            27, 38, 45,
            28, 33, 42, // triangles back
            29, 37, 43,
            30, 35, 46,
            31, 39, 47
        ];

        Mesh { vertices, indices }
    }

    fn create_legseg_mesh() -> Mesh {
        let vertices = vec![
            -0.025, -0.25, -0.05, // front square
            0.025, -0.25, -0.05,
            -0.025, -0.75, -0.05,
            0.025, -0.75, -0.05,
            -0.025, -0.25, 0.05, // back square
            0.025, -0.25, 0.05,
            -0.025, -0.75, 0.05,
            0.025, -0.75, 0.05,
            -0.05, -0.25, -0.025, // left square
            -0.05, -0.25, 0.025,
            -0.05, -0.75, -0.025,
            -0.05, -0.75, 0.025,
            0.05, -0.25, -0.025, // right square
            0.05, -0.25, 0.025,
            0.05, -0.75, -0.025,
            0.05, -0.75, 0.025,
            -0.025, 0.0, -0.025, // top square
            0.025, 0.0, -0.025,
            -0.025, 0.0, 0.025,
            0.025, 0.0, 0.025,
            -0.025, -1.0, -0.025, // bottom square
            0.025, -1.0, -0.025,
            -0.025, -1.0, 0.025,
            0.025, -1.0, 0.025
        ];

        let indices = vec![
            0, 1, 2, // squares
            2, 3, 1,
            4, 5, 6,
            6, 7, 5,
            8, 9, 10,
            10, 11, 9,
            12, 13, 14,
            14, 15, 13,
            16, 17, 18,
            18, 19, 17,
            20, 21, 22,
            22, 23, 21,
            0, 2, 8, // connecting squares
            2, 8, 10,
            1, 3, 12,
            3, 12, 14,
            9, 11, 6,
            4, 6, 9,
            13, 15, 5,
            15, 5, 7,
            16, 17, 0,
            0, 1, 17,
            18, 19, 4,
            19, 4, 5,
            16, 18, 8,
            18, 8, 9,
            17, 19, 12,
            19, 12, 13,
            20, 21, 2,
            2, 3, 21,
            22, 23, 6,
            6, 7, 23,
            20, 22, 10,
            10, 11, 22,
            21, 23, 14,
            14, 15, 23,
            0, 8, 16, // triangles front
            1, 12, 17,
            2, 10, 20,
            3, 14, 21,
            4, 9, 18, // triangles back
            5, 13, 19,
            6, 11, 22,
            7, 15, 23
        ];

        Mesh { vertices, indices }
    }
}
