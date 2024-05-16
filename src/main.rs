mod chunk;
mod compiler;
mod pipeline;
mod scanner;
mod value;
mod vm;

use std::{env, fs, io};

use crate::pipeline::Pipeline;

fn run_repl(pipeline: &mut Pipeline) {
    for line in io::stdin().lines() {
        if let Ok(line) = line {
            if let Err(e) = pipeline.interpret_source(&line) {
                println!("Error: {e:?}");
            }
        } else {
            break;
        }
    }
}

fn run_file(pipeline: &mut Pipeline, filepath: &str) -> Result<(), i32> {
    let source = fs::read_to_string(filepath).map_err(|e| {
        println!("Error: {e:?}");
        1
    })?;

    pipeline.interpret_source(&source).map_err(|e| {
        println!("Error: {e:?}");
        1
    })?;

    Ok(())
}

fn main() -> Result<(), i32> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        run_repl(&mut Pipeline {});
    } else if args.len() == 2 {
        run_file(&mut Pipeline {}, &args[1])?
    } else {
        println!("Usage: {} [path]", args[0]);
        return Err(1);
    }

    // let mut chunk = Chunk::default();

    // let idx = chunk.push_constant(1.2);
    // chunk.push(ByteCode::Constant(idx as u8), 0);

    // let idx = chunk.push_constant(3.4);
    // chunk.push(ByteCode::Constant(idx as u8), 0);

    // chunk.push(ByteCode::Add, 0);

    // let idx = chunk.push_constant(5.6);
    // chunk.push(ByteCode::Constant(idx as u8), 0);

    // chunk.push(ByteCode::Div, 0);
    // chunk.push(ByteCode::Negate, 0);

    // println!("Line of instruction 0: {:?}", chunk.get_line(0));

    // // TODO: Print line numbers for instructions
    // chunk.disassemble();

    // // Run the VM
    // let res = interpret(&chunk);
    // println!("{res:?}");

    Ok(())
}
