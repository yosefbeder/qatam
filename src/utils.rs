pub fn combine(a: u8, b: u8) -> u16 {
    (b as u16) << 8 | (a as u16)
}

pub fn split(bytes: u16) -> [u8; 2] {
    [bytes as u8, (bytes >> 8) as u8]
}
