
pub fn render_particles(
    mut draw_context: DrawContext,
    msaa: Res<Msaa>,
    mut query: Query<(&Draw, &RenderPipelines, &Visible)>
) {
    query.for_each_mut(|(mut draw, mut render_pipelines, particles, visible)| {
        if !visible.is_visible {
            return;
        }

        let mut render_pipeline = RenderPipeline::specialized(
            WIREFRAME_PIPELINE_HANDLE.typed(),
            PipelineSpecialization {
                sample_count: msaa.samples,
                strip_index_format: None,
                shader_specialization: Default::default(),
                primitive_topology: mesh.primitive_topology(),
                dynamic_bindings: render_pipelines
                    .bindings
                    .iter_dynamic_bindings()
                    .map(|name| name.to_string())
                    .collect::<HashSet<String>>(),
                vertex_buffer_layout: mesh.get_vertex_buffer_layout(),
            },
        );
        render_pipeline.dynamic_bindings_generation =
            render_pipelines.bindings.dynamic_bindings_generation();

        draw_context
            .set_pipeline(
                &mut draw,
                &render_pipeline.pipeline,
                &render_pipeline.specialization,
            )
            .unwrap();
        draw_context
            .set_bind_groups_from_bindings(&mut draw, &mut [&mut render_pipelines.bindings])
            .unwrap();
        draw_context
            .set_vertex_buffer(&mut draw, &[&render_pipelines.bindings])
            .unwrap();

        match mesh.indices() {
            Some(Indices::U32(indices)) => draw.draw_indexed(0..indices.len() as u32, 0, 0..1),
            Some(Indices::U16(indices)) => draw.draw_indexed(0..indices.len() as u32, 0, 0..1),
            None => draw.draw(0..mesh.count_vertices() as u32, 0..1),
        };
    }
}
