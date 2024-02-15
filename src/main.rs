fn main() {
    println!(
        "{:?}",
        map_parser::tokenizer(&std::fs::read_to_string("crates/map_parser/tests/220.map").unwrap())
    );
}
