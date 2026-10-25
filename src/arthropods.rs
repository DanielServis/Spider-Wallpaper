use gl::types::*;
use std::{f32::consts::PI, mem::size_of};
use glam::{Vec3, Vec4, Quat, Mat3, Mat4};

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

    last_group_step_time: f32,
    anim_time: f32,
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
            last_group_step_time: 0.0,
            anim_time: 0.0,
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

    pub fn update(&mut self, delta_time: f32) {
        let STEP_TRIGGER_DIST = 10.0;
        let step_speed = 3.0 * (self.motive.speed + 1.0);
        let STEP_HEIGHT = 5.0;
        let group_stagger = 0.1 * (10.0 / self.motive.speed);

        self.anim_time += delta_time;

        let model = self.transform.to_matrix();
        let inv_body = model.inverse();

        for (i, leg) in self.legs.iter_mut().enumerate() {
            let group_a = i % 2 == 0;

            let ideal_world_position: Vec3 = (model
                * Vec4::from((leg.rest_position, 1.0))).truncate();
            
            if leg.step_time >= 1.0 {
                leg.step_cool -= delta_time;
                let drift = (ideal_world_position - leg.current_target).length();

                let stagger_ok = group_a 
                    || (self.anim_time - self.last_group_step_time > group_stagger);

                if drift > STEP_TRIGGER_DIST && leg.step_cool <= 0.0 && stagger_ok {
                    leg.step_start = leg.current_target;
                    leg.step_end = ideal_world_position;
                    leg.step_time = 0.0;
                    leg.step_cool = 0.25;

                    if group_a {
                        self.last_group_step_time = self.anim_time;
                    }
                }
            }
            else {
                leg.step_time = (leg.step_time + delta_time * step_speed).min(1.0);

                let flat = leg.step_start.lerp(leg.step_end, leg.step_time);
                let lift = (leg.step_time * PI).sin() * STEP_HEIGHT;

                leg.current_target = flat + Vec3::new(0.0, lift, 0.0);
            }

            let target = leg.current_target;
            Self::solve_leg_ik(leg, target, inv_body, delta_time);
        }
    }

    fn solve_leg_ik(leg: &mut Leg, world_target: Vec3, inv_body: Mat4, delta_time: f32) {
        let local_target: Vec3 = (inv_body * Vec4::from((world_target, 1.0))).truncate();

        let hip_pos = Vec3::new(
            leg.coxa.transform.x,
            leg.coxa.transform.y,
            leg.coxa.transform.z,
        );

        let l0 = leg.coxa.length;
        let l1 = leg.femur.length;
        let l2 = leg.tibia.length;
        let l3 = leg.tarsus.length;

        let mut positions = [Vec3::ZERO; 5];
        positions[0] = hip_pos;

        // Coxa
        let to_target_full = local_target - hip_pos;
        let coxa_pitch = (-80.0_f32).to_radians();
        let coxa_yaw = to_target_full.x.atan2(-to_target_full.z);

        let coxa_dir = Vec3::new(
            coxa_yaw.sin() * coxa_pitch.cos(),
            -coxa_pitch.sin(),
            -coxa_yaw.cos() * coxa_pitch.cos(),
        );

        positions[1] = hip_pos + coxa_dir * l0;

        // Femur
        let knee_base = positions[1];
        let to_target = local_target - knee_base;
        let mut dist_to_target = to_target.length();

        let l_eff = l2 + l3;
        let max_reach = l1 + l_eff;
        let min_reach = (l1 - l_eff).abs();
        dist_to_target = dist_to_target.clamp(min_reach + 0.01, max_reach - 0.01);

        let to_target_dir = if to_target.length() > 1e-4 {
            to_target.normalize()
        } else {
            Vec3::new(0.0, -1.0, 0.0)
        };

        let cos_knee = ((l1 * l1 + dist_to_target * dist_to_target - l_eff * l_eff)
            / (2.0 * l1 * dist_to_target))
            .clamp(-1.0, 1.0);
        let knee_angle = cos_knee.acos();

        let coxa_horizontal = Vec3::new(coxa_dir.x, 0.0, coxa_dir.z).normalize();
        let up = Vec3::Y;

        let bend_axis_raw = coxa_horizontal.cross(up);
        let bend_axis_len = bend_axis_raw.length();
        let bend_axis = if bend_axis_len < 1e-3 {
            if coxa_horizontal.x.abs() < coxa_horizontal.z.abs() {
                Vec3::X
            } else {
                Vec3::Z
            }
        } else {
            bend_axis_raw / bend_axis_len
        };

        let rot = Quat::from_axis_angle(bend_axis, knee_angle);
        let mut femur_dir = rot * to_target_dir;
        femur_dir = Self::clamp_dir_pitch(femur_dir, coxa_horizontal, 5.0_f32.to_radians(), 45.0_f32.to_radians());
        positions[2] = knee_base + femur_dir * l1;

        // Tibia
        let knee_to_target = local_target - positions[2];
        let knee_to_target_dist = knee_to_target.length();
        let knee_to_target_dir = if knee_to_target_dist > 1e-4 {
            knee_to_target / knee_to_target_dist
        } else {
            Vec3::new(0.0, -1.0, 0.0)
        };

        positions[3] = positions[2] + knee_to_target_dir * l2;
        positions[4] = positions[3] + knee_to_target_dir * l3;

        Self::set_seg_angles(&mut leg.coxa.transform, Mat3::IDENTITY, positions[0], positions[1], delta_time);

        let coxa_rot = Mat3::from_mat4(leg.coxa.transform.to_matrix());
        Self::set_seg_angles(&mut leg.femur.transform, coxa_rot, positions[1], positions[2], delta_time);

        let femur_rot = coxa_rot * Mat3::from_mat4(leg.femur.transform.to_matrix());
        Self::set_seg_angles(&mut leg.tibia.transform, femur_rot, positions[2], positions[3], delta_time);

        let tibia_rot = femur_rot * Mat3::from_mat4(leg.tibia.transform.to_matrix());
        Self::set_seg_angles(&mut leg.tarsus.transform, tibia_rot, positions[3], positions[4], delta_time);
    }

    fn clamp_dir_pitch(dir: Vec3, coxa_horizontal_fallback: Vec3, min_pit: f32, max_pit: f32) -> Vec3 {
        let d = dir.normalize();
        let horiz = (d.x * d.x + d.z * d.z).sqrt();
        let pitch = (-d.y).atan2(horiz).clamp(min_pit, max_pit);

        let h = if horiz > 1e-4 {
            Vec3::new(d.x, 0.0, d.z) / horiz
        } else {
            coxa_horizontal_fallback
        };

        Vec3::new(h.x * pitch.cos(), -pitch.sin(), h.z * pitch.cos())
    }

    fn solve_ik() {

    }

    fn set_seg_angles(transform: &mut Transform, parent_world_rot: Mat3, from_pos: Vec3, to_pos: Vec3, delta_time: f32) {
        const DEADZONE: f32 = 0.1;
        const SMOOTH_RATE: f32 = 15.0;

        let diff = to_pos - from_pos;
        if diff.length() < 1e-5 {
            return;
        }

        let world_dir = diff.normalize();
        let local_dir = parent_world_rot.inverse() * world_dir;

        let horiz = (local_dir.x * local_dir.x + local_dir.z * local_dir.z).sqrt();
        let smoothing = 1.0 - (-SMOOTH_RATE * delta_time).exp();

        if horiz > DEADZONE {
            let target_yaw = (-local_dir.x).atan2(-local_dir.z);

            let mut diff_angle = target_yaw - transform.yaw;
            while diff_angle > PI {
                diff_angle -= 2.0 * PI;
            }
            while diff_angle < -PI {
                diff_angle += 2.0 * PI;
            }

            transform.yaw += diff_angle * smoothing;
        }

        let target_pitch = horiz.atan2(-local_dir.y);
        transform.pit += (target_pitch - transform.pit) * smoothing;

        transform.rol = 0.0;
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

pub struct Centipede {
    body_model: Model,
    legseg_model: Model,
    pub transform: Transform,
    pub motive: Motive,
    legs: [Leg; 30],
}
