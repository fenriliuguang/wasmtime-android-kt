//! Implemented wasi:webgpu `[method]` smokes in one integration binary.
//!
//! Keep each slice in its own module; do not add empty ignore stubs.

mod begin_compute_pass;
mod begin_render_pass;
mod buffer_map_async;
mod buffer_unmap;
mod command_encoder_finish;
mod compute_pass_dispatch_workgroups;
mod compute_pass_end;
mod compute_pass_set_bind_group;
mod compute_pass_set_pipeline;
mod copy_buffer_to_buffer;
mod create_bind_group;
mod create_bind_group_layout;
mod create_buffer;
mod create_command_encoder;
mod create_compute_pipeline;
mod create_pipeline_layout;
mod create_render_pipeline;
mod create_sampler;
mod create_shader_module;
mod create_texture;
mod device_queue;
mod queue_submit;
mod render_pass_draw;
mod render_pass_end;
mod render_pass_set_bind_group;
mod render_pass_set_pipeline;
mod render_pass_set_vertex_buffer;
mod request_adapter;
mod request_device;
mod texture_create_view;
mod write_buffer;
mod write_texture;
