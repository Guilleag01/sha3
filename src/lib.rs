pub mod consts;
pub mod sha3;

#[cfg(test)]
mod tests {
    // use super::*;

    use crate::sha3::Sha3_256;

    #[test]
    fn it_works() {
        let mut sha = Sha3_256::default();

        let text = "";

        let arr = text.as_bytes();

        let mut data = [0_u8; 136];
        data.clone_from_slice(arr);

        // sha.absorb([0, 1, 2, 3, 4, 5, 6, 7]);
        sha.absorb(&data);
        let res = sha.squeeze();
        // a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a
        let expected_res: [u8; 32] = [
            0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61,
            0xd6, 0x62, 0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b,
            0x80, 0xf8, 0x43, 0x4a,
        ];

        for i in 0..32 {
            print!("{:#001x}", res[i]);
        }
        println!();

        assert!(res == expected_res);
    }
}
