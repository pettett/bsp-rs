use bevy_ecs::system::{Commands, ResMut, SystemState};
use std::path::Path;

use bsp_explorer::{run, v::vrenderer::VRenderer};

pub fn main() {
    println!("Starting...");
    pollster::block_on(run(|state| {
        // Construct a `SystemState` struct, passing in a tuple of `SystemParam`
        // as if you were writing an ordinary system.
        let mut system_state: SystemState<(Commands, ResMut<VRenderer>)> =
            SystemState::new(state.world_mut());

        // Use system_state.get_mut(&mut world) and unpack your system parameters into variables!
        // system_state.get(&world) provides read-only versions of your system parameters instead.
        let (mut commands, mut renderer) = system_state.get_mut(state.world_mut());

        bsp_explorer::bsp::loader::load_bsp(Path::new("D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\maps\\d1_trainstation_02.bsp"), &mut commands, &mut renderer);

        system_state.apply(state.world_mut());
    }));
}
