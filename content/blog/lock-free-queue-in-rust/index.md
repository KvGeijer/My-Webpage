+++
title="Implementing a Lock-Free Queue in Rust"
date=2024-10-17

[taxonomies]
categories = ["Programming"]
tags = ["lock-free", "rust"]

[extra]
toc = true
+++

Classical concurrent data structures such as FIFO queues are foundational in many different applications. It is very simple to implement such a queue by wrapping a sequential implementation in a mutex. However, lock-free designs are often preferred, as they can give better scalability and progress guarantees than ones relying on locks. The classic lock-free FIFO queue is the [Michael-Scott queue](https://doi.org/10.1145/248052.248106) (MS queue), which is a neat lock-free linked list. Lock-free designs are inherently tricky to implement, and linked lists are known to be troublesome in Rust, so this post details my journey of implementing this MS queue in Rust.

<!-- more -->

# The Algorithm Idea

Before covering the Rust implementation, it can be helpful to understand the language-agnostic algorithm design of the [MS queue](https://doi.org/10.1145/248052.248106), which we will cover here with rusty pseudocode. However, if you already know it, or don't care for the algorithm part, feel free to skip ahead to the next section.

The MS queue implements a linked list with lock-free *enqueue* and *dequeue* methods, using the atomic primitive [compare-and-swap](https://en.wikipedia.org/wiki/Compare-and-swap) (CAS) to achieve lock-freedom. The code consists of two main structs, the queue and its nodes:

``` rust
struct MSQueue<T> {
    head: &Node<T>,
    tail: &Node<T>,
}

struct Node<T> {
    data: T,
    next: Option<&Node<T>>,
}
```

As you can see, the pseudocode here disregards memory management to just convey the idea of the algorithm. The `head` and `tail` pointers always point to a node, and the `next` pointer in a node is `None` as long as the node is the oldest in the queue (it is the tail), after which it changes to the new tail. To guarantee that `head` and `tail` always point to a node, they are initialized to point at a dummy node (in general, the queue is empty when `tail == head`).

``` rust
impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            next: None,
        }
    }
}

impl<T> MSQueue<T> {
    pub fn new() -> Self {
        let dummy = Node::new(<anything>);
        Self {
            head: &dummy,
            tail: &dummy,
        }
    }
}
```

Then, when a new items are enqueued, the `tail` will point to the logical tail node. However, the `head` will always point to the node before the logical head (the node whose `next` is the logical head). We will look at these functions now.

The enqueue is the most complicated, and logically performs two steps:
1. Update the `next` pointer of the current `tail` to point to the new node,
2. Update the `tail` to point at the newly enqueued node,

after which new enqueues can race to perform (1). The tricky part is to preserve lock-freedom. If the thread stalls between 1 and 2, we could get an issue of no other thread being able to make progress. To combat this, we let all threads help with step 2 , even for nodes enqueued by another thread. The pseudocode is shown below.

``` rust
impl <T> MSQueue<T> {
    pub fn enqueue(&mut self, item: T) {
        let new_node = Node::new(item);
        // Repeat until completing step 1
        loop {
            // Atomic reads
            let tail = self.tail;
            if let Some(next) = tail.next {
                // Try to help the enqueue of next
                // Step 2 (Helping another thread)
                CAS(&self.tail, tail, next);
            } else {
                // Try to enqueue the node by updating the tail.next
                // Step 1
                if CAS(&tail.next, None, &new_node) {
                    // Try to bump up self.tail to new_node
                    // Step 2
                    CAS(&self.tail, tail, &new_node);
                    break;
                }
            }
        }
    }
}
```

As we can see, the thread loops until it completes step 1. Until achieving that, it helps other threads complete their step 2. Finally, it tries to complete its own step 2.

The dequeue function is slightly easier, as it mainly consists of a repeatedly trying to complete a single step:
1. Update `head` to point at its `next` node.

The main complication is when the queue is almost empty, with only a single partially enqueued item (which has only passed enqueue step 1), which the dequeue must help finish before trying to enqueue it. The pseudocode is shown below.

``` rust
impl <T> MSQueue<T> {
    pub fn dequeue(&mut self) -> Option<T> {
        loop {
            // Atomic reads
            let head = self.head;
            let tail = self.tail;
            if head != tail {
                // Queue is not empty!
                // Dequeue step 1
                let next_head = head.next;
                if CAS(&self.head, next_head, next) {
                    // Successful dequeue
                    return Some(next_head.data);
                }
            } else if let Some(next_tail) == tail.next {
                // Help partial enqueue
                // Enqueue step 2
                CAS(&self.tail, tail, next_tail)
            } else {
                // Empty queue
                return None;
            }
        }
    }
}
```

This is the core of the algorithm, which at its core is rather simple. The part which is often hard to grasp is that all operations have to help partial enqueues at different points, to guarantee lock-freedom.

## Garbage Collection

Although the main algorithm is outlined above, real implementations contain some extra logic to handle memory management and to avoid the [ABA problem](https://en.wikipedia.org/wiki/ABA_problem). This is a whole research field in itself, but one of the most common and versatile solutions is using [Hazard pointers](https://en.wikipedia.org/wiki/Hazard_pointer). These have some practical issues, and are not the fastest, but are very versatile and give excellent correctness guarantees.

The main problem which hazard pointers try to solve is that no node should be freed when a thread has a reference to it. For example, assume we would freed the previous head just before returning from a successful dequeue. This would make sense, as it is then unlinked from the data structure. However, some slow thread could still have a reference to it, and try to do some CAS with it at this point. If the node is freed like this, the thread which is still trying to work with it will end up with undefined behavior problems, for example running into issues of the same memory being re-used for another node, running into the [ABA problem](https://en.wikipedia.org/wiki/ABA_problem). Hazard pointers solve the use-after-free and ABA problems in one fell sweep by ensuring no node is freed while another thread has a live reference to it.

The idea of hazard pointers is that each thread has a set of so called _hazard pointers_, which can be read by any thread, but only written to by the owner thread. When a thread wants to use a pointer, it protects it by writing it into one of its hazard pointers, only clearing the hazard pointer when it does not need the pointer anymore. Then, instead of directly freeing a node after dequeueing it, we _retire_ the node. Periodically, a thread calling _retire_ iterates over all retired nodes, freeing the ones which are not protected by the hazard pointer of any thread.

There are of course some further details of how these hazard pointers work, and there are many optimizations to make. But the base idea is that you protect a pointer while using it, which prevents other threads from freeing it.

# Rust Implementation

To actually implement the MS queue in Rust, we have to replace references with atomic pointers, integrate hazard pointers for safe memory management (using the [Haphazard](https://docs.rs/haphazard/latest/haphazard/) library), and in general ensure sound use of memory and ownership. Lets follow the same path as the pseudocode, starting with the structs:

## Structs

``` rust
struct Node<T> {
    next: AtomicPtr<Node<T>>,
    data: MaybeUninit<T>,
}

pub struct MSQueue<T> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}
```

First, we see that all references have now been replaced with `AtomicPtr`, which is essentially a wrapper around a raw pointer, that is updated and read atomically (furthermore, this is not `std::sync::atomic::AtomicPtr`, but rather `haphazard::AtomicPtr` which wraps the one in std to also integrates with hazard pointers nicely). As a minor optimization, `next` is just an `AtomicPtr<Node<T>>` instead of an `Option<AtomicPtr<Node<T>>>` as we can essentially use `std::ptr::null` instead of `None`.

The most interesting change from the pseudocode, in my opinion, is that `data` now has the type `MaybeUninit<T>` instead of just `T`. Why is this? Well, the root cause is that the oldest item in a MS queue is not stored in `queue.head.data`, but rather `queue.head.next.data`, meaning that the `queue.head` node is logically not part of the queue. This can be seen in that when we initialize the queue, we add a dummy node without any item. By using `MaybeUninit`, we can initialize this node without initializing its `data`. Furthermore, `MaybeUninit` does not automatically drop contained values when dropped, which is in line with the algorithm, as we want to return ownership of the item after a `dequeue`, in turn handing over the responsibility of dropping the item after its dequeue. Thus, it also avoids double-free and undefined behavior.

Furthermore, the initialization functions are adapted to handle these types:

``` rust
impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            next: unsafe { AtomicPtr::new(core::ptr::null_mut()) },
            data: MaybeUninit::new(data),
        }
    }

    fn empty() -> Self {
        Self {
            next: unsafe { AtomicPtr::new(core::ptr::null_mut()) },
            data: MaybeUninit::uninit(),
        }
    }
}

impl<T> MSQueue<T> {
    pub fn new() -> Self {
        let dummy = Box::new(Node::empty()).into_raw();
        Self {
            head: unsafe { AtomicPtr::new(dummy) },
            tail: unsafe { AtomicPtr::new(dummy) },
        }
    }
}
```

For a new node, we simply wrap the data in a new `MaybeUninit`, and initialize `next` as a null pointer. In the queue, similarly to the pseudocode, we create a dummy node that both `head` and `tail` point to. This dummy node is required for the algorithm (if we had let them both be `null`, we would need to update them both when enqueueing an item, rather than just `dummy.next`), and as its `data` is never read, we safely set it to an uninitialized `MaybeUninit`. Note also that we use `Box::into_raw`, which returns the raw pointer of a `Box`, and is very useful for us to get around ownership in the linked list.

## Enqueue

The logic of the `enqueue` is quite similar to the pseudocode, and except being a bit more verbose and taking a `HazardPointer` as input, you can see the exact same flow. We again have the following two logical steps:
1. Update the `next` pointer of the current `tail` to point to the new node,
2. Update the `tail` to point at the newly enqueued node.

The loop repeats until the thread has completed step 1, and attempted step 2 (other threads can help with step 2):

``` rust
impl<T: Sync + Send> MSQueue<T> {
    pub fn enqueue(&self, hp: &mut HazardPointer, data: T) {
        let new_node: *mut Node<T> = Box::new(Node::new(data)).into_raw();
        // Repeat until completing step 1
        loop {
            // Atomic reads
            let tail = self.tail.safe_load(hp).unwrap();
            let next = tail.next.load_ptr();
            if !next.is_null() {
                // Try to help the enqueue of next
                // Step 2 (Helping another thread)
                unsafe {
                    let _ = self.tail.compare_exchange_ptr(
                        tail as *const Node<T> as *mut Node<T>,
                        next,
                    );
                };
            } else {
                // Try to enqueue the node by updating the tail.next
                // Step 1
                if unsafe {
                    tail.next
                        .compare_exchange_ptr(std::ptr::null_mut(), new_node)
                }
                .is_ok() // Did the CAS succeed?
                {
                    // Try to bump up self.tail to new_node
                    // Step 2
                    unsafe {
                        let _ = self
                            .tail
                            .compare_exchange_ptr(tail as *const Node<T> as *mut Node<T>, new_node);
                    };
                    return;
                }
            }
        }
    }
}
```

First, the new node is allocated, and the `Box` is converted into a raw pointer. In the loop, instead of just reading `self.tail` with `self.tail.load_ptr()`, we use `self.tail.safe_load(hp)` which uses the hazard pointers to make sure the load is safe, returning a safe normal reference to the node. However, when we read `tail.next`, we don't actually need a safe reference to it, just a raw pointer which we can use CAS on and check if it is null. Thus we just use `load_ptr`, which is just a normal read.

As you see, it is a bit more verbose than the pseudocode. Specifically where we use `unsafe`, such as around CAS. We also use `compare_exchange_ptr` as CAS, which essentially is CAS for an `AtomicPtr`, operating on its wrapped raw pointer.

#### Memory safety question

Have you kept up so far? Or did I pull a fast one on you? We don't protect `next` with a hazard pointer, so it is not guaranteed that the memory that `next` points to is valid when we try to update `self.tail` to it with CAS (Step 2 - Helping another thread). So, have a think about if this is really safe.

> Is it safe to just use `let next = tail.next.load_ptr()`, and then use `next` in a CAS? Can't `next` have been freed, instead pointing to arbitrary memory?

So what do you think? This is actually safe, but it is not immediately obvious, and one must consider such scenarios very precisely. The reason is that we at that point are protecting `tail` with a hazard pointer. If the CAS succeeds, setting `self.tail = next`, that must mean that `self.tail == tail` at that instant, which in turn (as `tail.next == next`) must mean that `next` points to a node which cannot have been dequeued at that point. If the CAS fails, `next` can point to arbitrary memory, but in that case we don't use it.

## Dequeue

Translating the `dequeue` method is similar as the `enqueue`, but requires two hazard pointers. The idea is still to retry until achieving the single step:
1. Update `head` to point at its `next` node.

``` rust
impl<T: Sync + Send> MSQueue<T> {
    pub fn dequeue(&self, hp_head: &mut HazardPointer, hp_next: &mut HazardPointer) -> Option<T> {
        loop {
            // Atomic reads
            let head = self
                .head
                .safe_load(hp_head)
                .expect("MS queue should never be empty");
            let head_ptr = head as *const Node<T> as *mut Node<T>;
            let next_ptr = head.next.load_ptr();
            let tail_ptr = self.tail.load_ptr();

            if head_ptr != tail_ptr {
                // Queue is not empty!
                // Dequeue step 1
                let next = head.next.safe_load(hp_next).unwrap();
                if let Ok(unlinked_head_ptr) = unsafe {
                    self.head
                        .compare_exchange_ptr(head_ptr, next_ptr)
                } {
                    // Successful dequeue
                    unsafe {
                        unlinked_head_ptr.unwrap().retire();
                    }

                    // Take and return ownership of the data.
                    return Some(unsafe {
                        std::ptr::read(next.data.assume_init_ref() as *const _)
                    });
                }
            } else if !next_ptr.is_null() {
                // Help partial enqueue
                // Enqueue step 2
                unsafe {
                    let _ = self
                        .tail
                        .compare_exchange_ptr(tail_ptr as *mut Node<T>, next_ptr);
                }
            } else {
                // Empty queue
                return None;
            }
        }
    }
}
```

The code for when the queue is empty, or close to empty, (when the first `if` fails) follows quite simply from the pseudocode, and is mainly interesting algorithmically. However, when we attempt the dequeue within the first `if`, we safely load the reference `next` with our second hazard pointer. We need this safety to ensure that the node is not freed when we read out its `data`. It would be nice to be able to re-use the first hazard pointer here instead, but we need that during the safe load of `next`, as it reads from `head`, requiring the use of two hazard pointers.

Another interesting part is how we read out the `data`. First, we unsafely assume it is the initialized version of `MaybeUninit`, and then use  `std::ptr::read` to convert `*const T` to `T`, taking ownership of the data. Here you have to be careful, as we then have two owned values containing the data. However, due to using `MaybeUninit`, and algorithmic guarantees, our code will never read (or free) the `data` in this node again, making the ownership transfer safe.

Furthermore, we here see that we use `retire` to free the unlinked node from the queue. This free is then deferred until no hazard pointer has a reference to it. It requires `unsafe` as it can have severe consequences if the node is unlinked before it becomes completely unreachable from _all_ parts of the data structure.

## Using Miri

When dabbling in unsafe Rust, we can easily shoot ourselves in the foot. This is extra true when you (like me when I wrote this code) are not that familiar with using unsafe. However, [Miri](https://github.com/rust-lang/miri) is a great tool to regain some of that safety. It can run tests for you, while at the same time keeping track of metadata such as borrowings, which results in it finding a lot of bugs in unsafe code. This is similar to [valgrind](https://valgrind.org/) and [Google's sanitizers](https://github.com/google/sanitizers).

Miri is only available for nightly Rust, so you can install it as
``` sh
rustup +nightly component add miri
```

and run your tests with it as

```sh
cargo +nightly miri test
```

There are different configurations and so on, but this is the core usage. You should note that Miri interprets your test and code, which leads to a rather slow execution. So make sure you don't run it with too long tests.

This helped find one bug with CAS in the development of the code above. But if you use all the code in its current state, and add some simple test, you will still get an error: A memory leak! In hindsight, this should be obvious, as we never drop the remaining nodes in a queue when it is dropped, so let's fix that!

## Dropping the queue

As the `MSQueue` struct only contains atomic pointers, its default drop implementation will not drop what these pointers point to. Therefore, we have to do it ourselves by implementing the `Drop` trait.

There are two things we need to ensure we drop to not leak any memory:
1. The `Node<T>` nodes remaining in the queue, allocated with `Box::new` in `enqueue`.
2. The remaining `data` corresponding to items still in the queue at the end.

Here we have the working code for dropping these two parts:
``` rust
impl<T> Drop for MSQueue<T> {
    fn drop(&mut self) {
        // Don't drop the data on self.head
        let head = unsafe { Box::from_raw(self.head.load_ptr()) };
        let mut next = head.next;

        while !next.load_ptr().is_null() {
            let node = unsafe { Box::from_raw(next.load_ptr()) };

            // Drop the initialized data
            unsafe { node.data.assume_init() };

            // Move on to the next node
            next = node.next;
        }
    }
}
```

If you've never implemented `Drop` before, you get a mutable borrow to the struct, and can drop any final members before the default `drop` functionality uses the default `drop` on all the struct fields. Note that you don't have to worry about concurrency, as we have a mutable antialiased reference to the queue.

In this code, we iterate over all nodes in the queue, converting their pointers to `Box<Node<T>>`, which in turn drops the box when it goes out of scope. Furthermore, we load the `data` field on the nodes with `assume_init()`, which takes ownership of the data, dropping it when going out of scope. It feels a bit strange that we don't explicitly drop anything here, but the drops are explicit side effects after taking ownerships of the respective structs.

Finally, we don't read (drop) the `data` on the `head` node. As we have discussed before, this is due to the fact that that `data` has already been returned by a `dequeue`, and the responsibility of freeing it has been handed over to the one calling `dequeue`.

# Final thoughts

Implementing a lock-free linked list, in the form of the [MS queue](https://doi.org/10.1145/248052.248106), in Rust was a bit tricky. But after getting the hang of what types and functions to use, it's actually not too bad. You do have to think about more things than in an unchecked language like C when writing the implementation, but we also did not have to hunt down any hidden concurrency bugs. Furthermore, now that we have done this implementation, it can be used very easily in new safe code, encapsulating the tricky reasoning to this file.

This was a very fun side project, and there is much more to do. For one thing, it would be nice to compare its performance with other concurrent queues in Rust (it should not beat the fast ones). Then it would be good fun to implement a more modern and efficient design, based on fetch-and-add rather than compare-and-swap in its bottle-neck. Finally, I would like to integrate it with the [research](@/research/index.md) I am doing in my PhD, implementing a few relaxed queues.
