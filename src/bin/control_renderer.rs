use bsp_explorer::state::State;
use std::thread;

use bsp_explorer::{run, state_mesh::StateMesh};

pub fn main() {
    pollster::block_on(run(|state| {
        let instance = state.renderer().instance();

        let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::TriangleList);

        mesh.load_glb_mesh(instance.clone());

        state.add_mesh(mesh);

        let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::LineList);

        mesh.load_debug_edges(instance.clone());

        state.add_mesh(mesh);

        let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::TriangleList);

        mesh.load_debug_faces(instance);

        state.add_mesh(mesh);
    }));
}
