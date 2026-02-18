use nexum::ports::PortAllocator;

#[test]
fn allocates_unique_ports_across_multiple_capsules() {
    let mut allocator = PortAllocator::new(4300, 4310);
    allocator.reserve(4300);
    allocator.reserve(4301);

    let allocated = ["cap-a", "cap-b", "cap-c", "cap-d", "cap-e"]
        .into_iter()
        .map(|id| allocator.allocate(id).expect("port expected"))
        .collect::<Vec<_>>();

    assert_eq!(allocated, vec![4302, 4303, 4304, 4305, 4306]);
}

#[test]
fn allocation_is_stable_for_same_capsule_and_reusable_after_release() {
    let mut allocator = PortAllocator::new(5000, 5003);

    let first = allocator.allocate("cap-a").unwrap();
    let second = allocator.allocate("cap-a").unwrap();
    assert_eq!(first, second);

    allocator.release("cap-a");
    let reused = allocator.allocate("cap-b").unwrap();
    assert_eq!(reused, first);
}

#[test]
fn returns_none_when_range_is_exhausted() {
    let mut allocator = PortAllocator::new(7000, 7001);
    assert_eq!(allocator.allocate("cap-a"), Some(7000));
    assert_eq!(allocator.allocate("cap-b"), Some(7001));
    assert_eq!(allocator.allocate("cap-c"), None);
}

#[test]
#[should_panic(expected = "port range start must be <= end")]
fn rejects_invalid_port_range_configuration() {
    let _ = PortAllocator::new(7100, 7099);
}

#[test]
#[should_panic(expected = "reserved port out of range")]
fn rejects_reserving_out_of_range_port() {
    let mut allocator = PortAllocator::new(7200, 7201);
    allocator.reserve(7300);
}
