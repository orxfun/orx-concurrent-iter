use alloc::vec::Vec;

fn con_drain<T: Send + Sync>(mut vec: Vec<T>) {
    use crate::ConcurrentDrainableOverSlice;
    let _con_iter = vec.con_drain(..);
}

#[test]
fn vec_drain_con_iter_trait_bounds() {
    con_drain(Vec::<String>::new());
}
