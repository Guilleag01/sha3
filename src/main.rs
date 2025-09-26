use std::{env, fs::File, io::Read, time};

use sha3::sha3::Sha3_256;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let mut file = File::open(filename).unwrap();

    let mut file_data = Vec::new();

    file.read_to_end(&mut file_data).unwrap();

    // println!("{:?}", (0x01 as u64).to_ne_bytes());

    // let text = "hola";

    let mut time = 0_f32;

    let mut res: [u8; 32] = [0_u8; 32];

    let times = 1;

    for _ in 0..times {
        let mut sha = Sha3_256::default();
        let now = time::Instant::now();

        sha.absorb(&file_data);
        res = sha.squeeze();

        let elapsed = now.elapsed().as_micros() as f32;
        time += elapsed;
    }

    // let expected_res: [u8; 32] = [
    //     0x8a, 0xf1, 0x3d, 0x92, 0x44, 0x61, 0x8e, 0xee, 0x87, 0x6d, 0x04, 0x31, 0xf3, 0x44, 0x9a,
    //     0xa4, 0xff, 0x95, 0x27, 0x4c, 0xa3, 0xe7, 0xe5, 0xc6, 0x54, 0x19, 0x79, 0x49, 0x9f, 0x5b,
    //     0x85, 0xde,
    // ];

    print!("SHA3-256: ");
    for i in 0..32 {
        print!("{:x}", res[i]);
    }
    println!();

    println!("Avg Time taken: {} ms", (time / times as f32) / 1000_f32);

    // assert!(res == expected_res);
}
