use std::time::{SystemTime, UNIX_EPOCH};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use image::GenericImageView;


const RESOLUTION_X: u32 = 2400;//MUST BE DIVISIBLE BY 8
const RESOLUTION_Y: u32 = 1600;
const PIXEL_SIZE: u64 = 16; // 16 bytes per pixel for vec3 format
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Sphere {
    center: [f32; 3],  
    radius: f32,      
    material: materials::Material,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Star { //light struct
    color: [f32; 3],
    intensity: f32,  
    position: [f32; 3],
    radius: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct EnvDimensions {
    width: u32,
    height: u32,
}

use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec3};

use crate::{controls::cameracontroller::{CameraController}, gpu::{raytracer::materials, wgpu_init::Init}, scene::{self, scene::Scene}};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub position: [f32; 3],
    pub aspect_ratio: f32, 
    pub up:  [f32; 3],
    pub fov_y: f32, 
    pub forward: [f32; 3],
    pub _padding2: f32, // For alignment
}

impl CameraUniform {
    fn new(position: Vec3, forward: Vec3, up: Vec3, fov_y: f32, aspect_ratio: f32) -> Self {
        Self {
            position: position.to_array(),
            up: up.normalize().to_array(),
            forward: forward.normalize().to_array(),
            aspect_ratio,
            fov_y,
            _padding2:  0.0,
        }
    }
    fn update(){

    }
    fn resize(&mut self, height: f32, width: f32){
        self.aspect_ratio = height/width;
    }
}

pub struct ComputeState {
    pub camera_uniform: CameraUniform,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub object_bind_group: wgpu::BindGroup,
    pub output_buffer_bind_group: wgpu::BindGroup,
    pub output_buffer_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    pub output_buffer: wgpu::Buffer,            
    pub camera_buffer: wgpu::Buffer,
    pub env_bind_group: wgpu::BindGroup,
    pub rand_buffer: wgpu::Buffer,
}

impl ComputeState {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: &PhysicalSize<u32>) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_shader.wgsl").into()), 
        });

        let buffer_size = (RESOLUTION_X as u64 * RESOLUTION_Y as u64 * PIXEL_SIZE) as u64;

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let output_buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Buffer Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false }, 
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let output_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Buffer Bind Group"),
            layout: &output_buffer_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &output_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let (env_pixels, env_width, env_height) = load_image_as_rgba(r"src\assets\dock_texture.jpg");

        let env_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Environment Pixel Buffer"),
            contents: &env_pixels, 
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let env_dimensions_data = EnvDimensions {
            width: env_width,
            height: env_height,
        };

        let env_dimensions = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Environment Dimensions Buffer"),
            contents: bytemuck::cast_slice(&[env_dimensions_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let env_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Environment Bind Group Layout"),
        });

        let env_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &env_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: env_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: env_dimensions.as_entire_binding(),
                },
            ],
            label: Some("Environment Bind Group"),
        });

        let camera_position = Vec3::from_array([0.0,0.0,-80.0]);
        let target = Vec3::from_array([0.0,10.0,9.5]);
        let camera_forward = (target-camera_position).normalize();
        let up = Vec3::Y;
        let fov_y = 1.05;
        let aspect_ratio = (size.height as f32/size.width as f32) as f32;


        let camera_uniform = CameraUniform::new(camera_position, camera_forward,up,fov_y, aspect_ratio);


        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Camera Bind Group Layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });


        let rand_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });




        // Sphere buffer
        let sphere_1 = Sphere {
            center: [0.0,0.0,0.0],  
            radius: 30.0,              
            material: materials::glass_material(),
        };
        let sphere_2 = Sphere {
            center: [100.0,-0.0,0.0],  
            radius: 30.0,            
            material: materials::metal_material(),
        };
        let sphere_3 = Sphere {
            center: [50.0,-0.0,-60.0],  
            radius: 10.0,            
            material: materials::colored_glass(),
        };
        let sphere_4 = Sphere {
            center: [50.0,-0.0,-30.0],  
            radius: 10.0,            
            material: materials::dark_mirror(),
        };
        let sphere_5 = Sphere {
            center: [50.0,-0.0,0.0],  
            radius: 10.0,           
            material: materials::emerald_crystal(),
        };
        let sphere_7 = Sphere {
            center: [50.0, -0.0, 30.0],
            radius: 10.0,
            material: materials::polished_gold(),
        };
        
        let sphere_8 = Sphere {
            center: [50.0, -0.0, 60.0],
            radius: 10.0,
            material: materials::pearlescent(),
        };
        


        let sphere_data = vec![sphere_1,sphere_2,sphere_3,sphere_4,sphere_5,sphere_7,sphere_8];
        let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Storage Buffer"),
            contents: bytemuck::cast_slice(&sphere_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST 
        });
        let light_position = vec3(0.676,0.041,-0.735);

        let light_data = Star {
            color: [1.0,0.55,0.0],
            intensity: 1.0,
            position: (light_position * 2000.0).into(),
            radius: 0.0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_data]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });


        let object_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Object Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let object_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &object_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(sphere_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(light_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(rand_buffer.as_entire_buffer_binding()),
                }
            ],
            label: Some("Object Bind Group"),
        });


        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[
                &output_buffer_bind_group_layout,  // Bind group 0 for texture
                &object_bind_group_layout,   // Bind group 1 for sphere and light
                &camera_bind_group_layout,  
                &env_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });


        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            compute_pipeline,
            object_bind_group,
            camera_bind_group,
            camera_uniform,
            camera_buffer,
            env_bind_group,
            output_buffer_bind_group,
            output_buffer,
            output_buffer_bind_group_layout,
            rand_buffer,
        }
    }


    pub fn dispatch(&self, init: &Init) {
        let mut encoder = init.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.output_buffer_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.object_bind_group, &[]);
            compute_pass.set_bind_group(2, &self.camera_bind_group, &[]);
            compute_pass.set_bind_group(3, &self.env_bind_group, &[]);


            compute_pass.dispatch_workgroups(RESOLUTION_X, RESOLUTION_Y, 1);
        }

        init.queue.submit(Some(encoder.finish()));
    }
    pub fn resize(&mut self, size: PhysicalSize<u32>, queue: &wgpu::Queue){
        self.camera_uniform.resize(size.width as f32, size.height as f32);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }
    pub fn update(&mut self, queue: &wgpu::Queue, cameracontroller: &mut CameraController, scene: &mut Scene, delta_time: f32){
        scene.compile_camera(cameracontroller, delta_time, &mut self.camera_uniform );

        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        queue.write_buffer(&self.rand_buffer, 0, bytemuck::cast_slice(&[random_seed()]));
    }

}

fn random_seed() -> u32 {
    let start = SystemTime::now();
    let since_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    (since_epoch.as_millis() % u32::MAX as u128) as u32
}

fn load_image_as_rgba(path: &str) -> (Vec<u8>, u32, u32) {

    let img = image::open(path).expect("Failed to load image");

    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    (rgba.to_vec(), width, height)
}