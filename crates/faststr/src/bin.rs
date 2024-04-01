use std::time::Instant;

use faststr::FastStr;

const TEST_AMOUNT: usize = 10000;

fn main() {
    for c in (0..255u8).map(|c| c as char) {
        let og_string = [c; TEST_AMOUNT].into_iter().collect::<String>();
        println!("Testing {c:?}");

        let mut veccy = Vec::new();
        let time = Instant::now();
        for _ in 0..TEST_AMOUNT {
            veccy.push(og_string.clone());
        }
        println!(
            "Cloning String of {TEST_AMOUNT} {c:?} took {}s",
            time.elapsed().as_secs_f32()
        );

        let mut veccy = Vec::new();
        let fstring = FastStr::from(og_string);
        let time = Instant::now();
        for _ in 0..TEST_AMOUNT {
            veccy.push(fstring.clone());
        }
        println!(
            "Cloning FastStr of {TEST_AMOUNT} {c:?} took {}s",
            time.elapsed().as_secs_f32()
        );

        println!()
    }
}
