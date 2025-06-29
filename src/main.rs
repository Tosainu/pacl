fn main() {
    if let Err(e) = pacl::cli::run() {
        eprintln!("{e}");
        std::process::exit(3);
    }
}
