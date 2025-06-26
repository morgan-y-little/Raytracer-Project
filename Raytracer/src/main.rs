pub mod controls;
pub mod gpu;
pub mod scene;

use controls::cameracontroller::{ CameraController};
use gpu::raytracer::compute_pipeline::{self, ComputeState};
use gpu::raytracer::fragment_pipeline::{self, RenderState};
use gpu::wgpu_init::Init;
use winit::event::{DeviceEvent, MouseButton, RawKeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use scene::scene::Scene;

struct Timer{
    last_render_time: std::time::Instant,
}impl Timer{
    fn new()->Self{
        Self { last_render_time: std::time::Instant::now() }
    }
}
#[derive(Default)]
struct App<'a> {
    window: Option<winit::window::Window>,
    init: Option<Init<'a>>, 
    compute_state: Option<ComputeState>,
    fragment_state: Option<RenderState>,
    scene: Option<Scene>,
    controller: Option<CameraController>,
    timer: Option<Timer>,
}

impl <'a>winit::application::ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = winit::window::Window::default_attributes().with_title("Raytracer").with_inner_size(winit::dpi::PhysicalSize::new(1200, 800));
        let window = event_loop.create_window(window_attributes).unwrap();
        self.init = Some(pollster::block_on(Init::new(&window))); 
        self.compute_state = Some(pollster::block_on(compute_pipeline::ComputeState::new(&self.init.as_ref().unwrap().device, &self.init.as_ref().unwrap().queue, &window.inner_size())));
        self.fragment_state = Some(pollster::block_on(fragment_pipeline::RenderState::new(&self.init.as_ref().unwrap().device, &self.compute_state.as_ref().unwrap().output_buffer)));
        self.controller = Some(CameraController::new(100.0, 0.6));
        self.scene = Some(Scene::new());
        self.timer = Some(Timer::new());
        self.window = Some(window);
        
    }

    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::Resized(physical_size) => {
                if let Some(init) = self.init.as_mut() {
                    init.resize(physical_size);
                    if let Some(compute_pipeline) = self.compute_state.as_mut() {
                        compute_pipeline.resize(physical_size, &init.queue);
                    }
                }

            },
            WindowEvent::RedrawRequested => {

                if let Some(compute_pipeline) = self.compute_state.as_mut() {
                    compute_pipeline.dispatch(self.init.as_ref().unwrap()); 

                    if let Some(fragment_pipeline) = self.fragment_state.as_mut() {
                        let _ = fragment_pipeline.render(self.init.as_ref().unwrap()); 
                    }

                    if let Some(timer) = self.timer.as_mut() {
                        let delta_time = (std::time::Instant::now() - timer.last_render_time).as_secs_f32();
                        timer.last_render_time = std::time::Instant::now();

                            if let Some(cameracontroller) = self.controller.as_mut() {

                                if let Some(scene) = self.scene.as_mut(){

                                    compute_pipeline.update(&self.init.as_ref().unwrap().queue, cameracontroller, scene, delta_time);

                                }
                            }
                    }
                }
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
    
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        let _ = (event_loop, cause);
    }
    
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }
    
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(cameracontroller) = self.controller.as_mut() {
        
        match event {
            DeviceEvent::Key(RawKeyEvent { physical_key, state }) => {
                cameracontroller.process_keyboard(physical_key, state);
            }
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                cameracontroller.process_mouse(dx, dy);
            }
            DeviceEvent::Button { button, state } => {
                cameracontroller.process_mouse_button(MouseButton::Left, state);
            }
            DeviceEvent::MouseWheel { delta } => {
                cameracontroller.process_scroll(&delta);
            }
            _ => {}
        }
        }
    }
    
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
    
    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}
fn main(){
let event_loop = EventLoop::new().unwrap();
event_loop.set_control_flow(ControlFlow::Poll);
event_loop.set_control_flow(ControlFlow::Wait);
let mut app = App::default();
let _ = event_loop.run_app(&mut app);
}