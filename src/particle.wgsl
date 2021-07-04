// TODO: try merging this block with the binding?
[[block]]
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var view: View;

struct VertexInput {
  [[location(0)]] vertex_position: vec4<f32>,
  [[location(1)]] vertex_uv: vec2<f32>,
  [[location(2)]] particle_position: vec4<f32>,
  [[location(3)]] particle_size: vec4<f32>,
  [[location(4)]] particle_color: vec4<f32>,
}

struct VertexOutput {
  [[builtin(position)]] position: vec4<f32>,
  [[location(0)]] color: vec4<f32>,
  [[location(1)]] uv: vec2<f32>,
}

[stage(vertex)]
fn vs_main(model: VertexInput) -> VertexOutput {
  var pos = model.particle_position.xyz;
  var rot = model.particle_position.w;
  var size = model.particle_size;
  var translation: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(pos, 1.0);
  );

  var rotation: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(cos(rot), sin(rot), 0.0, 0.0),
    vec4<f32>(-sin(rot), cos(rot), 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0),
  );

  var scale: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(size, 0, 0.0, 0.0),
    vec4<f32>(0, size, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, size, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0),
  );

  var transform = translation * rotation * scale;

  var out: VertexOutput;
  out.color = model.color;
  out.uv = model.vertex_uv;
  out.position = view.view_proj * transform * vec4<f32>(model.position, 0.0);
  return out;
}

[stage(fragment)]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return in.color;
}
