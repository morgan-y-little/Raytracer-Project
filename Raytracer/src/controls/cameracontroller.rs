
use winit::dpi::PhysicalPosition;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::event::{ ElementState, MouseButton, MouseScrollDelta,};
use glam::{ Vec3};
use crate::gpu::raytracer::compute_pipeline::CameraUniform;
use crate::scene::scene::Camera;

#[derive(Debug)]
pub struct CameraController {
    is_dragging: bool,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}impl CameraController{
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            is_dragging: false,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: PhysicalKey, state: ElementState) -> bool{
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyW)=> {
                self.amount_forward = amount;
                true
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyS) => {
                self.amount_backward = amount;
                true
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyA) => {
                self.amount_left = amount;
                true
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyD)=> {
                self.amount_right = amount;
                true
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyQ) => {
                self.amount_up = amount;
                true
            }
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyE) => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        if self.is_dragging {
            self.rotate_horizontal = mouse_dx as f32;
            self.rotate_vertical = mouse_dy as f32;
        }
    }
    pub fn process_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            self.is_dragging = state == ElementState::Pressed;
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
    }
    
    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32, uniform: &mut CameraUniform) {
        self.update_camera_rotation(camera, delta_time);
        let pitch = camera.pitch;
        let yaw = camera.yaw;
        let forward_direction = Vec3::new(
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin()
        ).normalize();
        
        let right_direction = Vec3::new(
            -yaw.sin(),
            0.0,
            yaw.cos()
        ).normalize();

        
        camera.position += forward_direction * (self.amount_forward - self.amount_backward) * self.speed * delta_time;
        camera.position += right_direction * (self.amount_right - self.amount_left) * self.speed * delta_time;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * delta_time;

        uniform.position = camera.position.into();
        uniform.forward = forward_direction.into();
    }

    pub fn update_camera_rotation(&mut self, camera: &mut Camera, delta_time: f32) {

        camera.yaw += self.rotate_horizontal * self.sensitivity * delta_time;
        camera.pitch += -self.rotate_vertical * self.sensitivity * delta_time;
        if camera.pitch > 1.50{
            camera.pitch = 1.49;
        }
        if camera.pitch < -1.50{
            camera.pitch = -1.49;
        }

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }

}