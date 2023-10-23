pub(crate) fn sizeof(number: usize) -> u32 {
    let mut size = 0;
    let mut n = number;
    while n > 0 {
        size += 1;
        n >>= 1;
    }
    size.max(1)
}
