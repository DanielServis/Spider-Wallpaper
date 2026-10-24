use gl::types::*;
use std::mem::size_of;
use glam::{Vec3, Mat4, EulerRot};

use crate::things::{Transform, Motive, Mesh, Model};

pub struct LegSeg {
    transform: Transform,
    length: f32,
}

pub struct LegAttach {
    offset: Vec3,
    base_yaw: f32,
}

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
    /*pub fn new(x: f32, y: f32, z: f32) -> Self {
        for i in 0..8 {
            
        }

        Spider {
            transform: Transform::new(x, y, z, 0.0, 0.0, 0.0),
            motive: Motive::new(),
        }
    }*/

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

    fn create_body_mesh() -> Mesh {
        let mut mesh: Mesh;

        mesh.vertices = [

        ];

        mesh.indices = [

        ];

        mesh
    }

    fn create_legseg_mesh() -> Mesh {
        let mut mesh: Mesh;

        mesh.vertices = [

        ];

        mesh.indices = [

        ];

        mesh
    }
}

impl Drop for Spider {
    fn drop(&mut self) {
        unsafe {

        }
    } 
}
