#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockState(pub u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_state_is_a_dense_handle() {
        assert_eq!(BlockState(0).0, 0);
        assert_eq!(BlockState(u16::MAX).0, u16::MAX);
        assert_eq!(BlockState(42), BlockState(42));
        assert_ne!(BlockState(1), BlockState(2));
    }
}
