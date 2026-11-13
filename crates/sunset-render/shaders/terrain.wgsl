struct Camera {
    view_proj: mat4x4<f32>,
    fog_start: f32,
    fog_end: f32,
    fog_color: vec4<f32>,
    tint_color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) layer: u32,
    @location(2) @interpolate(flat) ao: u32,
    @location(3) @interpolate(flat) light: u32,
    @location(4) @interpolate(flat) tint_index: u32,
    @location(5) fog_distance: f32,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var terrain_textures: texture_2d_array<f32>;
@group(0) @binding(2) var terrain_sampler: sampler;
@group(0) @binding(3) var<storage, read> section_origins: array<vec4<f32>>;

fn face_uv(face: u32, position: vec3<f32>) -> vec2<f32> {
    if face == 0u || face == 1u {
        return vec2<f32>(position.x, position.z);
    }
    if face == 2u || face == 3u {
        return vec2<f32>(position.x, position.y);
    }
    return vec2<f32>(position.y, position.z);
}

@vertex
fn vs_main(@location(0) packed: vec2<u32>, @builtin(instance_index) instance: u32) -> VertexOut {
    let lo = packed.x;
    let hi = packed.y;
    let local = vec3<u32>((lo & 0x1Fu), ((lo >> 5u) & 0x1Fu), ((lo >> 10u) & 0x1Fu));
    let face = (lo >> 15u) & 0x7u;
    let ao = (lo >> 18u) & 0x3u;
    let light = (lo >> 20u) & 0xFu;
    let layer = ((lo >> 24u) & 0xFFu) | ((hi & 0xFFu) << 8u);
    let tint_index = (hi >> 8u) & 0xFFu;

    let world = section_origins[instance].xyz + vec3<f32>(local);
    let clip = camera.view_proj * vec4<f32>(world, 1.0);

    var out: VertexOut;
    out.clip_position = clip;
    out.uv = face_uv(face, world);
    out.layer = layer;
    out.ao = ao;
    out.light = light;
    out.tint_index = tint_index;
    out.fog_distance = length(world);
    return out;
}

fn light_brightness(light: u32) -> f32 {
    return f32(light) / 15.0;
}

fn ao_factor(ao: u32) -> f32 {
    return 1.0 - f32(ao) / 3.0;
}

fn apply_fog(color: vec3<f32>, distance: f32) -> vec3<f32> {
    let span = camera.fog_end - camera.fog_start;
    let factor = clamp((distance - camera.fog_start) / span, 0.0, 1.0);
    return mix(color, camera.fog_color.rgb, factor);
}

fn shade_texel(texel: vec4<f32>, ao: u32, light: u32, tint_index: u32, distance: f32) -> vec3<f32> {
    var color = texel.rgb;
    if tint_index > 0u {
        color = color * camera.tint_color.rgb;
    }
    color = color * (light_brightness(light) * ao_factor(ao));
    return apply_fog(color, distance);
}

@fragment
fn fs_opaque(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(terrain_textures, terrain_sampler, in.uv, i32(in.layer));
    return vec4<f32>(shade_texel(texel, in.ao, in.light, in.tint_index, in.fog_distance), 1.0);
}

@fragment
fn fs_cutout(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(terrain_textures, terrain_sampler, in.uv, i32(in.layer));
    if texel.a < 0.5 {
        discard;
    }
    return vec4<f32>(shade_texel(texel, in.ao, in.light, in.tint_index, in.fog_distance), 1.0);
}

@fragment
fn fs_translucent(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(terrain_textures, terrain_sampler, in.uv, i32(in.layer));
    let rgb = shade_texel(texel, in.ao, in.light, in.tint_index, in.fog_distance);
    return vec4<f32>(rgb, texel.a);
}
