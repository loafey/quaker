fn main() {
    match map_parser::parse(
        &std::fs::read_to_string("crates/map_parser/tests/combined.map").unwrap(),
    ) {
        Ok(tree) => println!("{tree:#?}"),
        Err(e) => eprintln!("{e}"),
    }
}
