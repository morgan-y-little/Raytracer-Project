#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material{
    refractive_index: f32,
    mirror_matte: f32,
    absorption: f32,
    specular: f32,
    color: [f32;4],
}


pub fn glass_material() -> Material {
    Material {
        refractive_index: 1.5,
        mirror_matte: 0.000,
        absorption: 0.01,
        specular: 0.9,
        color: [0.8, 0.8, 1.0, 1.0],
    }
}

pub fn metal_material() -> Material {
    Material {
        refractive_index: 0.0,
        mirror_matte: 0.01,
        absorption: 0.3,
        specular: 0.95,
        color: [0.8, 0.8, 0.85, 1.0],
    }
}

pub fn colored_glass() -> Material {
    Material {
        refractive_index: 1.3,
        mirror_matte: 0.1,
        absorption: 0.5,
        specular: 0.85,
        color: [0.7, 0.1, 0.2, 1.0],
    }
}

pub fn dark_mirror() -> Material {
    Material {
        refractive_index: 0.0,
        mirror_matte: 0.0,
        absorption: 0.9,
        specular: 1.0,
        color: [0.1, 0.1, 0.1, 1.0],
    }
}

pub fn polished_gold() -> Material {
    Material {
        refractive_index: 0.0,
        mirror_matte: 0.05,
        absorption: 0.2,
        specular: 0.9,
        color: [1.0, 0.843, 0.0, 1.0],
    }
}


pub fn pearlescent() -> Material {
    Material {
        refractive_index: 0.0,
        mirror_matte: 0.01,
        absorption: 0.55,
        specular: 0.8,
        color: [0.98, 0.92, 0.9, 1.0],
    }
}

pub fn emerald_crystal() -> Material {
    Material {
        refractive_index: 1.6,
        mirror_matte: 0.1,
        absorption: 0.05,
        specular: 0.8,
        color: [0.0, 0.8, 0.3, 0.7],
    }
}

pub fn rusty_metal() -> Material {
    Material {
        refractive_index: 0.0,
        mirror_matte: 1.0,
        absorption: 0.5,
        specular: 0.2,
        color: [0.6, 0.3, 0.2, 1.0],
    }
}

pub fn obsidian() -> Material {
    Material {
        refractive_index: 1.2,
        mirror_matte: 0.05,
        absorption: 0.7,
        specular: 0.85,
        color: [0.05, 0.05, 0.1, 0.9],
    }
}