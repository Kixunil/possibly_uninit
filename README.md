Possibly uninitialized
======================

Traits and types helping with using uninitialized memory safely.

About 
-----

This crate provides several traits and types that make working with
uninitialized memory safer. They avoid memory bugs like accidentally
writing uninitialized value into initialized memory, reading uninitialized
memory, etc. They also provide strong guarantees for other safe code, which
is expressed as `unsafe` traits.

Since uninitialized values make most sense when it comes to large objects,
the main focus is on slices and arrays. For instance, you can initialize
`Box<[T]>` or `Box<[T; N]>` after it was allocated, avoiding copying.
Unfortunately that part isn't quite perfect right now, but it does seem to
work correctly.

The crate is `no_std`-compatible and `alloc`-compatible, of course.
