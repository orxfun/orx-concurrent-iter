use orx_concurrent_iter::*;

#[test]
fn new() {
    let counter = AtomicCounter::new();
    assert_eq!(counter.current(), 0);

    let counter = AtomicCounter::default();
    assert_eq!(counter.current(), 0);

    let counter = counter.clone();
    assert_eq!(counter.current(), 0);
}

#[test]
fn fetch_and_add_increment() {
    let counter = AtomicCounter::new();

    let prior = counter.fetch_and_add(1);
    assert_eq!(prior, 0);
    assert_eq!(counter.current(), 1);

    let prior = counter.fetch_and_add(4);
    assert_eq!(prior, 1);
    assert_eq!(counter.current(), 5);

    let prior = counter.fetch_and_increment();
    assert_eq!(prior, 5);
    assert_eq!(counter.current(), 6);

    let counter = counter.clone();
    assert_eq!(counter.current(), 6);
}
