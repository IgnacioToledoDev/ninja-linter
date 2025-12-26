use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    name: String,
    count: u8,
}

fn main() {
    println!("Hello, world!");
    if shadow_rs::branch().is_empty() {
        eprintln!("No branch founded!")
    }

    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello, {}!", args.name);
    }
}
