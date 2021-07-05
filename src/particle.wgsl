// TODO: try merging this block with the binding?
[[block]]
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var view: View;

struct VertexInput {
  [[location(0)]] vertex_position: vec3<f32>;
  [[location(1)]] vertex_uv: vec2<f32>;
  [[location(2)]] particle_position: vec4<f32>;
  [[location(3)]] particle_size: f32;
  [[location(4)]] particle_color: vec4<f32>;
};

struct VertexOutput {
  [[builtin(position)]] position: vec4<f32>;
  [[location(0)]] color: vec4<f32>;
  [[location(1)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(model: VertexInput) -> VertexOutput {
  // Uses the view projection matrix to compute the world-space movement directions
  // TODO: This is actually constant for all billboards and is best done CPU-side.
  var camera_right: vec3<f32> = 
    normalize(vec3<f32>(view.view_proj.x.x, view.view_proj.y.x, view.view_proj.z.x));
  var camera_up: vec3<f32> = 
    normalize(vec3<f32>(view.view_proj.x.y, view.view_proj.y.y, view.view_proj.z.y));

  var theta: f32 = model.particle_position.w;
  var sin_cos: vec2<f32> = vec2<f32>(cos(theta), sin(theta));
  
  var rotation: mat2x2<f32> = mat2x2<f32>(
    vec2<f32>(sin_cos.x, -sin_cos.y),
    vec2<f32>(sin_cos.y, sin_cos.x),
  );

  var position: vec2<f32> = rotation * model.vertex_position.xy;

  var world_space: vec3<f32> = 
    model.particle_position.xyz + 
    (camera_right * position.x * model.particle_size) + 
    (camera_up * position.y * model.particle_size);

  var out: VertexOutput;
  out.position = view.view_proj * vec4<f32>(world_space, 1.0);
  out.color = model.particle_color;
  out.uv = model.vertex_uv;
  return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return in.color;
}
