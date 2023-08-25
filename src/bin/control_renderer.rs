use bsp_explorer::run;

pub fn main() {
    pollster::block_on(run());
}
