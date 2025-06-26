// Define the Sphere struct
const MAX_RAY_DISTANCE: f32 = 1000.0; // Maximum ray travel distance
struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material,
}
// Define the Light struct
struct Star {
    color: vec3<f32>,
    intensity: f32,
    position: vec3<f32>,
    radius: f32,
};
struct Camera{
    position: vec3<f32>,
    aspect_ratio: f32,
    up: vec3<f32>,
    fov_y: f32, 
    forward: vec3<f32>, 
    _padding2: f32, 
};
struct EnvDimensions{
    width: u32,
    height: u32,
}
struct Material{
    refractive_index: f32,
    mirror_matte: f32,
    absorption: f32,
    specular: f32,
    color: vec4<f32>
}

// Binding the resources
@group(0) @binding(0) var<storage, read_write> output_buffer: array<vec4<f32>>;
@group(1) @binding(0) var<storage, read> sphere_data: array<Sphere>;
@group(1) @binding(1) var<storage, read> stars: array<Star>;
@group(2) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(2) var<uniform> rand_seed: u32;
@group(3) @binding(0) var<storage, read> env_buffer: array<u32>;
@group(3) @binding(1) var<uniform> env_dimensions: EnvDimensions;
var<workgroup> shared_accum: array<vec4<f32>, 16>;
 



@compute @workgroup_size(4, 4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(num_workgroups) dispatch_size: vec3<u32>, @builtin(workgroup_id) group_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>, @builtin(local_invocation_index) local_index: u32) {
    let camera_pos: vec3<f32> = camera.position;
    let max_x= f32(dispatch_size.x/2); 
    let max_y= f32(dispatch_size.y/2);

    
    let pixel_x = (f32(group_id.x) - max_x) / max_x; 
    let pixel_y = (f32(group_id.y) - max_y) / max_y;


    let scale = tan(camera.fov_y * 0.5 ); 
    let aspect_corrected_x = pixel_x * camera.aspect_ratio * scale;
    let aspect_corrected_y = pixel_y * scale;


    let right = normalize(cross(camera.forward, camera.up));
    let up = normalize(cross(right, camera.forward)); 

    let pix_ray_dir = normalize((aspect_corrected_x) * right + (aspect_corrected_y) * up + camera.forward);

    
    var ray_dir = conic_distribution(pix_ray_dir,0.001,global_id);
    var ray_color = vec3<f32>(0.0,0.5,0.0);
    var ray_origin = camera.position;
    let max_bounces = u32(8);
    var is_inside = false;
    var weight = 1.0;
    var ray_accumulated_color = vec3<f32>(0.0,0.0,0.0);
    let epsilon = 0.005;

    for(var b = u32(0); b < max_bounces;b++){

        var absorbed = false;
        var hit_point = vec3<f32>(0.0, 0.0, 0.0);
        var normal = vec3<f32>(0.0, 0.0, 0.0);
        var closest_t = 1000000.0;
        var closest_hit = false;
        var closest_sphere = Sphere();
        var inv_norm = false;

        for (var s = u32(0); s < arrayLength(&sphere_data); s++) {
            let sphere = sphere_data[s];
            inv_norm = false;
            let t = detect_hit(ray_origin, ray_dir, sphere);
            if t > 0.0 && t < closest_t { // hits in front of the ray origin and closer than previous hits
                closest_t = t;
                closest_hit = true;
                closest_sphere = sphere;
                hit_point = ray_origin + t * ray_dir;
                normal = normalize(hit_point - sphere.center);
                if length(ray_origin-hit_point) > length(ray_origin-sphere.center){ 
                    inv_norm = true;
                }
            }
        }

        if closest_hit{
            let closest_material = closest_sphere.material;

            if inv_norm{
                normal = -normal;
            }
            ray_origin = hit_point;

            var refracts = false;

            if closest_material.refractive_index > 0.01{
                refracts = true;
            }


            let refraction_dir = refract_dir(ray_dir,closest_material.refractive_index,normal, is_inside);

            if length(refraction_dir) < 0.01{ //total internal reflection 
                refracts = false;
            }

            if refracts{
                is_inside = !is_inside;
            }

            let reflection_dir = ray_dir - 2.0 * dot(ray_dir, normal) * normal;


            //Surface roughness
            if refracts{
                ray_dir = conic_distribution(refraction_dir,closest_material.mirror_matte,global_id);
            }else{
                ray_dir = conic_distribution(reflection_dir,closest_material.mirror_matte,global_id);
            }

            var diffuse_intensity = 1.0;
            let light = stars[0];
            let light_dir = normalize(light.position - hit_point);

            if !refracts {
                diffuse_intensity = max(dot(normal, light_dir), 0.5);
            } else {
                //ensure some 
                diffuse_intensity = max(abs(dot(normal, light_dir)), 0.5);
            }

            let highlight = pow(max(dot(normal, light_dir), 0.0), closest_material.specular * 32.0);

            let r_highlight = min(closest_material.color.r * light.color.r * highlight, light.color.r);
            let g_highlight = min(closest_material.color.g * light.color.g * highlight, light.color.g);
            let b_highlight = min(closest_material.color.b * light.color.b * highlight, light.color.b);
            
            let r_contribution = min(closest_material.color.r * light.color.r * diffuse_intensity, light.color.r);
            let g_contribution = min(closest_material.color.g * light.color.g * diffuse_intensity, light.color.g);
            let b_contribution = min(closest_material.color.b * light.color.b * diffuse_intensity, light.color.b);

            weight -= closest_material.absorption;
            ray_accumulated_color += vec3<f32>(
                r_contribution,
                g_contribution,
                b_contribution
            ) * closest_material.absorption * (1.0 / f32(b + 1));

            ray_accumulated_color += vec3<f32>(
                r_highlight,
                g_highlight,
                b_highlight,
            ) * closest_material.specular * (1.0 / f32(b + 1));

        } else{
            ray_accumulated_color += sample_spherical_background(ray_dir) * weight;
            break;
        }

        if weight < 0.01{
            break;
        }

        ray_origin += epsilon * ray_dir;
    }


    shared_accum[local_index] = vec4<f32>(ray_accumulated_color,1.0);
    workgroupBarrier();

    let index = group_id.y * dispatch_size.x + group_id.x;
    var accumulated_color = vec4<f32>(0.0,0.0,0.0,0.0);

    for(var i: u32 = 0; i< u32(16);i++){
        accumulated_color += shared_accum[i];
    }
    output_buffer[index] = accumulated_color/16.0;

}

fn refract_dir(ray_dir: vec3<f32>, refractive_index: f32, normal: vec3<f32>, exit_cond: bool) -> vec3<f32> {

    var eta_ratio = 1.0 / refractive_index;
    if exit_cond{
        eta_ratio = refractive_index / 1.0;
    }
    let cos_theta = dot(-ray_dir, normal);
    let sin_theta_sq = 1.0 - cos_theta * cos_theta;

    if eta_ratio * eta_ratio * sin_theta_sq > 1.0 {
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    //snells law
    let r_out_perpendicular = eta_ratio * (ray_dir + cos_theta * normal);
    let r_out_parallel = -sqrt(abs(1.0 - dot(r_out_perpendicular, r_out_perpendicular))) * normal;

    return r_out_perpendicular + r_out_parallel;
}


fn detect_hit(origin: vec3<f32>, ray_dir: vec3<f32>, sphere: Sphere) -> f32 {
    let oc = origin - sphere.center;
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(oc, ray_dir);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant > 0.0 {
        let t = (-b - sqrt(discriminant)) / (2.0 * a);
        return t;
    }
    //no valid hit - flag value of -1 (does not consider hits behind camera)
    return -1.0;
}

fn init_color(global_id: vec3<u32>) -> vec3<f32> {
    let random_r = rand_float(global_id, u32(0));
    let random_g = rand_float(global_id, u32(1)); 
    let random_b = rand_float(global_id, u32(2));

    let total_random = random_r + random_g + random_b;

    let lightcolor = vec3<f32>(1.0,1.0,1.0);

    let color = vec3<f32>(
        (random_r / total_random) * lightcolor.x,
        (random_g / total_random) * lightcolor.y,
        (random_b / total_random) * lightcolor.z
    );
    
    return color;
}


fn conic_distribution(a: vec3<f32>, stdev: f32, global_id: vec3<u32>) -> vec3<f32> {

    //given a vector, a, returns a vector that is *close* to a but has a random angle difference according to a distribution
    //random angles in spherical coordinates
    let u = rand_float(global_id,u32(0)); 
    let o = rand_float(global_id,u32(1));
    let theta = acos(1.0 - u * (1.0 - cos(stdev)));
    let phi = o * 2.0 * 3.141592653589793;

    let sin_theta = sin(theta);
    let x = sin_theta * cos(phi);
    let y = sin_theta * sin(phi);
    let z = cos(theta);

    var dir = vec3<f32>(x, y, z);

    let up = vec3<f32>(0.0, 0.0, 1.0); 
    let axis = -normalize(cross(up, a));
    let angle = acos(dot(up, normalize(a)));
    let rotation_matrix = rotation_matrix_around_axis(axis, angle);

    // apply rotation to the direction vector
    dir = (rotation_matrix * vec4<f32>(dir, 0.0)).xyz;
    return normalize(dir); 
}

fn rotation_matrix_around_axis(axis: vec3<f32>, angle: f32) -> mat4x4<f32> {
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    let one_minus_cos_a = 1.0 - cos_a;

    let x = axis.x;
    let y = axis.y;
    let z = axis.z;

    return mat4x4<f32>(
        cos_a + x * x * one_minus_cos_a,
        x * y * one_minus_cos_a - z * sin_a,
        x * z * one_minus_cos_a + y * sin_a,
        0.0,

        y * x * one_minus_cos_a + z * sin_a,
        cos_a + y * y * one_minus_cos_a,
        y * z * one_minus_cos_a - x * sin_a,
        0.0,

        z * x * one_minus_cos_a - y * sin_a,
        z * y * one_minus_cos_a + x * sin_a,
        cos_a + z * z * one_minus_cos_a,
        0.0,

        0.0, 0.0, 0.0, 1.0
    );
}
//////////////////
//RNG
fn rand(global_id: vec3<u32>, offset: u32) -> u32 {
    var state = rand_seed ^ (global_id.x * 374761393u) ^ (global_id.y * 668265263u) ^ offset;
    state = state * 1664525u + 1013904223u;  // LCG parameters for 32-bit values
    return state;
}
fn rand_float(global_id: vec3<u32>, offset: u32) -> f32 {
    let random_int = rand(global_id, offset);
    return f32(random_int) / f32(0xFFFFFFFFu);  // normalize to 0-1
}
fn rand_normal(global_id: vec3<u32>, offset: u32, stdev: f32, average: f32) -> f32 {
    let u1 = f32(rand(global_id, offset)) / f32(0xFFFFFFFFu);
    let u2 = f32(rand(global_id, offset + 1u)) / f32(0xFFFFFFFFu);

    // Box-Muller transform
    let z0 = sqrt(-2.0 * log(u1)) * cos(2.0 * 3.14159265359 * u2);

    let random_value = z0 * stdev + average;

    return random_value;
}

fn sample_spherical_background(ray_dir: vec3<f32>) -> vec3<f32> {

    let relative_dir = normalize(ray_dir);
    let theta = atan2(relative_dir.z, relative_dir.x);  
    let phi = acos(relative_dir.y);                    


    let u = (theta / (2.0 * 3.141592653589793)) + 0.5;
    let v = phi / 3.141592653589793;

    let x = u * f32(env_dimensions.width - 1);
    let y = v * f32(env_dimensions.height - 1);

    let x0 = u32(floor(x));
    let x1 = min(x0 + 1, env_dimensions.width - 1);
    let y0 = u32(floor(y));
    let y1 = min(y0 + 1, env_dimensions.height - 1);

    let tx = x - f32(x0);
    let ty = y - f32(y0);

    let color00 = unpack_color(env_buffer[y0 * env_dimensions.width + x0]);
    let color10 = unpack_color(env_buffer[y0 * env_dimensions.width + x1]);
    let color01 = unpack_color(env_buffer[y1 * env_dimensions.width + x0]);
    let color11 = unpack_color(env_buffer[y1 * env_dimensions.width + x1]);

    let color0 = mix(color00, color10, tx);
    let color1 = mix(color01, color11, tx);
    let final_color = mix(color0, color1, ty);

    return final_color.rgb;
}

fn unpack_color(packed_color: u32) -> vec4<f32> {
    let r = f32((packed_color >> 0) & 0xFF) / 255.0;
    let g = f32((packed_color >> 8) & 0xFF) / 255.0;
    let b = f32((packed_color >> 16) & 0xFF) / 255.0;
    let a = f32((packed_color >> 24) & 0xFF) / 255.0;
    return vec4<f32>(r, g, b, a);
}




