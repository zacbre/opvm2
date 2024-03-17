use std::{fs::File, io::Write, path::Path, time::Instant};

use clap::Parser;
use lz4::{Decoder, EncoderBuilder};
use opvm2::parser::program::Program;
use opvm2_vm::{vm::Vm, CompiledProgram};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
    #[arg(short, long)]
    compile: bool,
    #[arg(short, long, required = true)]
    file: String,
    #[arg(short, long)]
    interpret: bool,
    #[arg(short, long)]
    plugin: Vec<String>,
}

fn compress(input: Vec<u8>, output_file: String) -> Result<(), String> {
    let output_file = File::create(output_file).map_err(|e| e.to_string())?;
    let mut encoder = EncoderBuilder::new().level(4).build(output_file).unwrap();
    encoder.write(&input).unwrap();
    let (_output, _result) = encoder.finish();
    Ok(())
}

fn run_interpreter(vm: &mut Vm, path: String, plugins: Vec<Vec<u8>>) -> Result<(), String> {
    let file_content = std::fs::read_to_string(path).unwrap();
    let program = Program::from(file_content.as_str());
    // load all plugins
    vm.plugin.load_all(plugins)?;
    vm.run(program)?;
    Ok(())
}

fn load_plugins(plugins: Vec<String>) -> Result<Vec<Vec<u8>>, String> {
    let mut loaded = Vec::new();
    for plugin in plugins {
        let content = std::fs::read(plugin).map_err(|e| e.to_string())?;
        loaded.push(content);
    }
    Ok(loaded)
}

fn run_compiled_program(vm: &mut Vm, path: String) -> Result<(), String> {
    let input_file = File::open(&path).map_err(|e| e.to_string())?;
    let mut decoder = Decoder::new(input_file).map_err(|e| e.to_string())?;
    let mut buffer: Vec<u8> = Vec::new();
    std::io::copy(&mut decoder, &mut buffer).map_err(|e| e.to_string())?;
    let compiled = CompiledProgram::from_compiled(buffer);
    // load any plugins
    vm.plugin.load_all(compiled.plugins)?;
    vm.run(compiled.program).unwrap();
    Ok(())
}

fn compile(path: String, plugins: Vec<Vec<u8>>) -> Result<(), String> {
    let file_content = std::fs::read_to_string(&path).unwrap();
    let program = Program::from(file_content.as_str());
    let to_compile = CompiledProgram::new(program, plugins);

    let compiled = to_compile.compile()?;

    compress(
        compiled,
        format!(
            "{}c",
            Path::new(&path).file_name().unwrap().to_str().unwrap()
        ),
    )?;
    Ok(())
}

fn main() -> Result<(), String> {
    let mut vm = Vm::new_e();
    let args = Args::parse();

    if args.debug {
        vm.plugin
            .load_from_path("target/wasm32-unknown-unknown/debug/debugger.wasm")
            .unwrap();
    }

    let plugins = load_plugins(args.plugin)?;

    if args.interpret {
        run_interpreter(&mut vm, args.file, plugins)?;
        return Ok(());
    }

    if args.compile {
        let now = Instant::now();
        compile(args.file, plugins)?;
        let end = now.elapsed();

        println!("Compiled in {} ms", end.as_millis());
        return Ok(());
    }

    run_compiled_program(&mut vm, args.file)?;

    Ok(())
}
