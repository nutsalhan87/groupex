# Groupex

Syncronization primitive that allows acquire lock by index.

`RawGroupex` is implementation of this primitive. Its size the same as `usize`'s size. 
It is being locked by index and panics when index out of this range. Range depends on size of `usize`, i.e. depends on the platform.
It uses parking_lot internally for parking threads.

This crate provides `GroupexMap` and `GroupexVec` structs - hash table and dynamic array.

Let's talk about `GroupexMap` first. It includes `HashMap`. Its cells can be locked by keys.
For this `GroupexMap` computes hash of the key and calculates the remainder of the division this hash by `usize`'s size - 64 on amd64.
So it locks not only the element connected to the key but all other elements that collides to that key.
Thus **it's dangerous to lock another cell when one lock already acquired in the thread**.
So, this is some kind of Bloom filter.

`GroupexVec` works similar to the previous one but includes `Vec` and its cells can be locked just by index.
It's also **dangerous to lock another cell when one lock already acquired in the thread** because different indexes can be collided due to size of the `RawGroupex`.

Take into account that using just `HashMap<_, Mutex<_>>` is faster than `GroupexMap`.
The second one is your choice only if you need more space-efficient solution.