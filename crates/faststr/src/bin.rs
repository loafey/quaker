use std::time::Instant;

use faststr::FastStr;

fn main() {
    let max = 10000;
    let chars = 255u8;

    println!("Testing string of length {max}");
    let mut string_total_time = 0.0;
    let mut fstring_total_time = 0.0;

    for c in (0..chars).map(|c| c as char) {
        let og_string = vec![c; max].into_iter().collect::<String>();

        let mut veccy = Vec::new();
        for _ in 0..max {
            let time = Instant::now();
            veccy.push(og_string.clone());
            string_total_time += time.elapsed().as_secs_f32();
        }

        let mut veccy = Vec::new();
        let fstring = FastStr::from(og_string);
        for _ in 0..max {
            let time = Instant::now();
            veccy.push(fstring.clone());
            fstring_total_time += time.elapsed().as_secs_f32();
        }
    }

    let avg_string = string_total_time / chars as f32;
    let avg_fstring = fstring_total_time / chars as f32;
    println!("Copying String took {avg_string:.10}s on average",);
    println!("Copying FastStr took {avg_fstring:.10}s on average",);
    println!("A {:.2}% speedup", (avg_string / avg_fstring) * 100.0);
}
