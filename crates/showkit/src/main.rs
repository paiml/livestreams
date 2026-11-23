fn main() {
    std::process::exit(showkit::run(&std::env::args().skip(1).collect::<Vec<_>>()));
}
