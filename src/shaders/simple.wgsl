@group(0) @binding(0) var<uniform> uGlobals: GlobalsUniform;
@group(1) @binding(0) var<uniform> uCamera: CameraUniform;
@group(2) @binding(0) var<uniform> uModel: ModelUniform;
@group(3) @binding(0) var text: texture_2d<f32>;
@group(3) @binding(1) var norm: texture_2d<f32>;
@group(3) @binding(2) var sampl: sampler;

struct GlobalsUniform {
    time: f32
}

struct CameraUniform {
    projection: mat4x4f,
    view: mat4x4f,
    camera_pos: vec3f,
}

struct ModelUniform {
    model: mat4x4f,
    normal: mat4x4f,
}

struct VertexInput {
    @location(0) pos: vec3f,
    @location(1) tangent: vec3f,
    @location(2) bitangent: vec3f,
    @location(3) normal: vec3f,
    @location(4) uv: vec2f,
};

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) tangent: vec3f,
    @location(1) bitangent: vec3f,
    @location(2) normal: vec3f,
    @location(3) view_direction: vec3f,
    @location(4) uv: vec2f,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let world_pos = uModel.model * vec4f(in.pos, 1.0);
    let out_pos = uCamera.projection * uCamera.view * world_pos;
    let normal = (uModel.normal * vec4f(in.normal, 0.0)).xyz;
    let view_direction = normalize(uCamera.camera_pos - world_pos.xyz);

    let tangent = (uModel.normal * vec4f(in.tangent, 0.0)).xyz;
    let bitangent = (uModel.normal * vec4f(in.bitangent, 0.0)).xyz;

    return VertexOutput(out_pos, tangent, bitangent, normal, view_direction, in.uv);
}

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) face: bool) -> @location(0) vec4f {
    let light = vec3f(10.0, 0.0, 0.0); 
    let texture_sample = textureSample(text, sampl, in.uv);
    let normal_sample = textureSample(norm, sampl, in.uv);
    let local_normal = normal_sample.rgb * 2.0 - 1.0;
    let local_to_world = mat3x3f(
        normalize(in.tangent),
        normalize(in.bitangent),
        normalize(in.normal)
    );
    let world_normal = local_to_world * local_normal;
    let strength = 0.5;
    let normal = mix(in.normal, world_normal, strength);
    
    let diffuse = max(0.1, dot(-light, normal)) * texture_sample;
    
    let half_dir = normalize(normalize(in.view_direction) + normalize(light));
    let angle = max(0.0, dot(normal, half_dir));
    let hardness = 32.0;
    let specular = 0.4*vec4f(vec3f(pow(angle, hardness)), 1.0);
    
    return diffuse + specular;
}
