use glam::{vec3, Quat, Vec3};

use crate::{controls::cameracontroller::CameraController, gpu::raytracer::compute_pipeline::CameraUniform};


pub struct Camera{
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,

}impl Camera{
    pub fn new(position: Vec3, forward: Vec3) -> Self {
        let yaw = forward.z.atan2(forward.x);    
        let pitch = forward.y.clamp(-1.0, 1.0).asin();  
    
        (yaw, pitch);
        Self {
            position,
            yaw,
            pitch,

        }
    }
}

pub struct Scene{
    pub camera: Camera,
}impl Scene{
    pub fn new()->Self{
        Self { camera: Camera::new(vec3(0.0,0.0,-80.0), vec3(0.0,3.0,9.5)) }
    }
    pub fn compile_camera(&mut self, controller: &mut CameraController, delta_time:f32, uniform: &mut CameraUniform){
        controller.update_camera(&mut self.camera, delta_time, uniform)
    }
    pub fn compile_objects(){}
    pub fn update(&mut self){}
}