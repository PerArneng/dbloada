mod traits;
mod components;
mod component_assembler;

use component_assembler::ComponentAssembler;

fn main() {
    let assembler = ComponentAssembler::new();
    let engine = assembler.db_loada_engine();
    engine.init();
}
