//! Azusa CLI — repository drift intelligence.
//!
//! Measures, classifies, explains and plans reconciliation of drift between
//! repository states. The bare `azusa` invocation is the fast path and is
//! equivalent to `azusa drift`.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("azusa {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    eprintln!("azusa: CLI lands with milestone 1 (phase 1 foundation)");
    std::process::exit(5);
}
