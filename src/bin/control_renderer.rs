use bsp_explorer::{bsp::header::dheader_t, state::State};
use std::thread;

use bsp_explorer::{run, state_mesh::StateMesh};

pub fn main() {
    pollster::block_on(run(|state| {
        let instance = state.renderer().instance();

        let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::TriangleList);

        let (header,mut buffer) = dheader_t::load(
			"D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_02.bsp").unwrap();

        header.validate();

        mesh.load_glb_mesh(instance.clone());

        state.add_mesh(mesh);

        let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::LineList);

        mesh.load_debug_edges(instance.clone(), &header, &mut buffer);

        state.add_mesh(mesh);

        let mut mesh = StateMesh::new(state.renderer(), wgpu::PrimitiveTopology::TriangleList);

        mesh.load_debug_faces(instance, &header, &mut buffer);

        state.add_mesh(mesh);
    }));
}
