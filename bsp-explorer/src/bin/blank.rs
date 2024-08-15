use bsp_explorer::{vinit, vrun};

pub fn main() {
    println!("Starting...");

    let (state, event_loop) = pollster::block_on(vinit());

    vrun(state, event_loop);
}
