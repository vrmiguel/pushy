use std::sync::Arc;

use pushy::PushArray;

#[test]
fn drop() {
    let arc = Arc::new(0);

    {
        let mut arr: PushArray<_, 3> = PushArray::new();
        for _ in 0..3 {
            arr.push(arc.clone());
        }
        // There should now be 4 references to the
        // element of the Arc
        assert_eq!(Arc::strong_count(&arc), 4);
    }

    // The PushArray must've been dropped
    //
    // Therefore the reference count of the Arc
    // should now be 1.
    assert_eq!(Arc::strong_count(&arc), 1);
}

#[test]
fn clear() {
    let arc = Arc::new(0);

    let mut arr: PushArray<_, 4> = PushArray::new();
    for _ in 0..4 {
        arr.push(arc.clone());
    }

    let popped = arr.pop().unwrap();

    arr.clear();

    assert_eq!(Arc::strong_count(&arc), 2);
    assert_eq!(arr.len(), 0);
    assert_eq!(*popped, 0);
}

#[test]
fn pop_drop() {
    let arc = Arc::new(0);
    let mut arr: PushArray<_, 1> = PushArray::new();
    arr.push(arc.clone());

    let _dropped = arr.pop().unwrap();

    assert_eq!(Arc::strong_count(&arc), 2);
}

#[test]
fn pop_str() {
    let mut arr: PushArray<&str, 2> = PushArray::new();
    arr.push("Over");
    arr.push("There");
    assert_eq!(arr.len(), 2);

    let popped = arr.pop().unwrap();
    assert_eq!(arr.len(), 1);

    arr.push("Here");

    assert_eq!(arr.as_slice(), &["Over", "Here"]);
    assert_eq!(popped, "There");
}

#[test]
fn partial_eq() {
    let mut arr1: PushArray<u64, 2> = PushArray::new();
    arr1.push(5);
    arr1.push(10);

    let mut arr2: PushArray<u64, 2> = PushArray::new();
    arr2.push(5);
    arr2.push(10);

    assert_eq!(arr1, arr2);
}

#[test]
fn into_iter() {
    let mut arr: PushArray<u64, 2> = PushArray::new();
    arr.push(5);
    arr.push(10);

    let sum: u64 = arr.into_iter().sum();
    assert_eq!(sum, 15);
}

#[test]
fn deref_to_slice() {
    let mut arr: PushArray<u8, 5> = PushArray::new();
    arr.push_str("World").unwrap();

    let slice: &[u8] = &*arr;

    assert_eq!(slice, arr.as_slice());
}

#[test]
fn copy_from_slice_fails_when_not_enough_capacity() {
    let mut arr: PushArray<u8, 3> = PushArray::new();
    let zeroes = [0, 0, 0, 0];

    assert!(arr.copy_from_slice(&zeroes).is_err());
}

#[test]
fn push_array_fails_when_not_enough_capacity() {
    let mut arr: PushArray<u8, 3> = PushArray::new();
    let zeroes = [0, 0, 0, 0];

    assert!(arr.push_array(zeroes).is_err());
}

#[test]
fn push_checked() {
    let mut arr: PushArray<u8, 3> = PushArray::new();
    assert!(arr.push_checked(10).is_ok());
    assert!(arr.push_checked(20).is_ok());
    assert!(arr.push_checked(30).is_ok());

    // Not enough capacity!
    assert!(arr.push_checked(50).is_err());
    assert!(arr.push_checked(60).is_err());
}

#[test]
fn length() {
    let mut bytes: PushArray<u8, 9> = PushArray::new();
    assert_eq!(bytes.len(), 0);
    assert!(bytes.is_empty());

    bytes.push(b'H');
    assert_eq!(bytes.len(), 1);
    assert_eq!(bytes.is_empty(), false);

    bytes.push_str("ey ").unwrap();
    assert_eq!(bytes.len(), 4);
    assert_eq!(bytes.is_empty(), false);

    let hello = [b'H', b'e', b'l', b'l', b'o'];
    bytes.push_array(hello).unwrap();
    assert_eq!(bytes.len(), 9);

    bytes.clear();
    assert_eq!(bytes.len(), 0);
    assert!(bytes.is_empty());
}

#[test]
fn push_array() {
    let mut bytes: PushArray<u8, 10> = PushArray::new();
    let hello = [b'H', b'e', b'l', b'l', b'o'];
    bytes.copy_from_slice(&hello).unwrap();
    assert_eq!(bytes.as_str(), Some("Hello"));

    bytes.push_array(hello).unwrap();
    assert_eq!(bytes.as_str(), Some("HelloHello"));
}

#[test]
fn as_str_and_push_str() {
    let mut bytes: PushArray<u8, 11> = PushArray::new();
    bytes.push_str("Hello").unwrap();
    assert_eq!(bytes.as_str(), Some("Hello"));

    bytes.push(b' ');
    assert_eq!(bytes.as_str(), Some("Hello "));

    bytes.push_str("World").unwrap();
    assert_eq!(bytes.as_str(), Some("Hello World"));
}

#[test]
fn copy_from_slice() {
    let mut arr: PushArray<_, 10usize> = PushArray::new();
    let byte_slice = b"rogue-like";

    arr.copy_from_slice(byte_slice).unwrap();

    assert_eq!(arr.as_slice(), byte_slice)
}

#[test]
fn get() {
    let mut arr: PushArray<u8, 10> = PushArray::new();
    arr.push_str("Hey").unwrap();

    assert_eq!(arr.get(0), Some(&b'H'));
    assert_eq!(arr.get(1), Some(&b'e'));
    assert_eq!(arr.get(2), Some(&b'y'));
    assert_eq!(arr.get(3), None);
}

#[test]
fn get_mut() {
    let mut arr: PushArray<u8, 3> = PushArray::new();
    arr.push_str("Hey").unwrap();

    assert_eq!(arr.as_str().unwrap(), "Hey");

    let t = arr.get_mut(1).unwrap();
    *t = b'a';

    assert_eq!(arr.as_str().unwrap(), "Hay");
}

#[test]
fn index_impl() {
    let mut arr: PushArray<u8, 3> = PushArray::new();

    arr.push_str("Hey").unwrap();

    assert_eq!(arr[0], b'H');
    assert_eq!(arr[1], b'e');
    assert_eq!(arr[2], b'y');
}

#[test]
#[should_panic]
fn index_panics_when_out_of_bounds() {
    let mut arr: PushArray<u8, 3> = PushArray::new();

    arr.push_str("Hey").unwrap();

    assert_eq!(arr[0], b'H');
    assert_eq!(arr[1], b'e');
    assert_eq!(arr[2], b'y');
    arr[3]; // uh-oh
}

#[test]
#[should_panic]
fn panics_when_overflows() {
    let mut numbers: PushArray<u32, 1> = PushArray::new();
    numbers.push(2); // ok
    numbers.push(3); // uh-oh!
}

#[test]
fn initialized_i32() {
    let mut numbers: PushArray<u32, 50> = PushArray::new();
    for number in [2, 5, 7, 2, 3, 4] {
        numbers.push(number);
    }

    assert_eq!(numbers.initialized(), &[2, 5, 7, 2, 3, 4]);
    assert_eq!(numbers.as_slice(), &[2, 5, 7, 2, 3, 4]);
}

#[test]
fn initialized_str() {
    let mut words: PushArray<&str, 50> = PushArray::new();
    for word in ["hey", "there", "friend"] {
        words.push(word);
    }

    assert_eq!(words.initialized(), &["hey", "there", "friend"]);

    words.push("miss ya");
    assert_eq!(words.initialized(), &["hey", "there", "friend", "miss ya"]);
}

#[test]
fn initiliazed_when_uninitialized() {
    let numbers: PushArray<u8, 20> = PushArray::new();
    assert_eq!(numbers.initialized(), &[])
}

#[test]
fn collect_iterator() {
    let array = [1, 2, 3, 4];
    let numbers: PushArray<u8, 20> = array.iter().copied().collect();

    assert_eq!(numbers.as_slice(), array.as_slice());
}

#[test]
#[should_panic]
fn collect_iterator_capacity_error() {
    let array = [1, 2, 3, 4];
    let numbers: PushArray<u8, 3> = array.iter().copied().collect();

    assert_eq!(numbers.as_ref(), array.as_slice());
}

#[test]
fn collect_iterator_empty_without_capacity_dont_panic() {
    let array = [];
    let numbers: PushArray<u8, 0> = array.iter().copied().collect();

    assert_eq!(numbers.as_slice(), array.as_slice());
}
