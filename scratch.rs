fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    println!("ARGC: {}", args.len());
    for (i, arg) in args.iter().enumerate() {
        println!("ARG {}: {:?}", i, arg);
    }
}
