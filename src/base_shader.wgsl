struct InstanceInput {
    [[location(2)]] model_matrix_0: vec4<f32>;
    [[location(3)]] model_matrix_1: vec4<f32>;
    [[location(4)]] model_matrix_2: vec4<f32>;
    [[location(5)]] model_matrix_3: vec4<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] vertex_color: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] vertex_color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position.x, model.position.y, 0.0, 1.0) * model_matrix;
    out.vertex_color = vec4<f32>(model.vertex_color, 1.0);
    return out;
}

[[stage(fragment)]]
fn fragment_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.vertex_color;
}
 