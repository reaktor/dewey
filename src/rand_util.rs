extern crate rand;

use self::rand::random;

fn get_rand_alphanum() -> char {
    // >>> !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~<<<
    // lowercase 97...122 = 26
    // uppercase 65...90 = 26
    // numerals 48...57 = 10
    // lowercase 0...25, + 97
    // uppercase 26...51, + 39
    // numerals 52...61, - 4
    let mut val = (random::<f32>() * 62.0) as u8;
    if val >= 52 {
        val = val - 4;
    } else if val >= 26 {
        val = val + 39;
    } else {
        val = val + 97;
    }
    val as char
}

pub fn random_string(len: usize) -> String {
    (0..len).map(|_| get_rand_alphanum()).collect()
}
