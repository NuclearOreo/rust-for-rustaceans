#[allow(dead_code)]
fn addition(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn running_print() {
        assert_eq!(addition(3, 6), 9);
    }
    
}