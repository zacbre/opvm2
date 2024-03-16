use clap::Parser;
use opvm2::parser::program::Program;
use opvm2_vm::vm::Vm;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let mut vm = Vm::new_e();
    let args = Args::parse();
    if args.debug {
        vm.plugin
            .load("target/wasm32-unknown-unknown/debug/debugger.wasm");
    }
    vm.plugin
        .load("target/wasm32-unknown-unknown/debug/plugin_test.wasm");
    vm.run(Program::from(
        r"
        _main:
            mov ra, 26          ; length of the alphabet
            mov rb, 97          ; start position of ASCII table
            add ra, rb
        _loop:
            ascii rb
            inc rb
            test rb, ra
            jl loop
        _end:
    ",
    ))
    .unwrap();
}
