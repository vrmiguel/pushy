# `pushy`: Vec-like stack-allocated buffer [![codecov](https://codecov.io/gh/vrmiguel/pushy/branch/main/graph/badge.svg?token=S0H9YIOGAQ)](https://codecov.io/gh/vrmiguel/pushy)

`pushy::PushArray` is a safe abstraction over uninitialized Rust arrays.

## A buffer you can push elements to

```rust
// Fixed capacity of 3
let mut arr: PushArray<_, 3> = PushArray::new();

while let Some(elem) = rx.next() {
   // `push` panics if a buffer overflow would happen
   arr.push(elem);
}

if let Some(elem) = other_rx.next() {
   // Non-panicking version of `push
   arr.push_checked(elem)?;
}
```

## The length is the amount of initialized elements

```rust
let mut arr: PushArray<u8, 5> = PushArray::new();

// Nothing was initialized yet
assert_eq!(arr.len(), 0);

arr.push_str("World")?;
assert_eq!(arr.len(), 5);
```

## Byte-specific methods

```rust
// `as_str` and `push_str` are implemented for `PushArray<u8>`
let mut arr: PushArray<u8, 10> = PushArray::new();
arr.push_str("Hey")?;

// Converts to &str if the contents of the array are valid UTF-8
assert_eq!(arr.as_str(), Some("Hey"));
``` 

## You can only access elements that were initialized

```rust
let mut arr: PushArray<u8, 10> = PushArray::new();
arr.push_str("Hey")?;

assert_eq!(arr.get(0), Some(&b'H'));
assert_eq!(arr.get(1), Some(&b'e'));
assert_eq!(arr.get(2), Some(&b'y'));

// Even though the capacity is 10, only three elements were initialized, so `get(3)` returns None
assert_eq!(arr.get(3), None);

// Access through the Index trait
assert_eq!(arr[2], b'y');
```

## Pushing many elements at once

```rust
let mut bytes: PushArray<u8, 10> = PushArray::new();

let hello = [b'H', b'e', b'l', b'l', b'o'];
// You can copy from a slice (currently only for Copy types)
bytes.copy_from_slice(&hello)?;

assert_eq!(bytes.as_str(), Some("Hello"));

// Push an array onto the PushArray taking ownership of these elements (works for !Copy elements)
bytes.push_array(hello)?;

assert_eq!(bytes.as_str(), Some("HelloHello"));
```

## Get all initialized elements

```rust
let mut numbers: PushArray<u32, 50> = PushArray::new();
for number in [2, 5, 7, 2, 3, 4] {
    numbers.push(number);
}

// Get all initialized elements with `initialized`
assert_eq!(numbers.initialized(), &[2, 5, 7, 2, 3, 4]);
// `as_slice` is an alias to `initialized`
assert_eq!(numbers.as_slice(), &[2, 5, 7, 2, 3, 4]);
```
