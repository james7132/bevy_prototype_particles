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
  [[location(1)]] vertex_uv: vec4<f32>,
  [[location(2)]] particle_position: vec4<f32>,
  [[location(3)]] particle_size: vec4<f32>,
  [[location(4)]] particle_color: vec4<f32>,
}

struct VertexOutput {
  [[builtin(position)]] position: vec4<f32>,
  [[location(0)]] color: vec4<f32>,
}

[stage(vertex)]
fn vs_main(model: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.color = model.color;
  out.position = view.view_proj * vec4<f32>(model.position, 0.0);
  return out;
}

[stage(fragment)]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return in.color;
}