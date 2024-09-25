+++
title="Implementing a Lock-Free Queue in Rust"
date=2024-09-25

[taxonomies]
categories = ["Programming"]
tags = ["lock-free", "rust"]

[extra]
toc = true
+++

Classical concurrent data structures such as FIFO queues are foundational in many different applications. It is very simple to implement such a concurrent queue by wrapping a sequential implementation in a mutex. However, lock-free data structures are often used as they can give better scalability and progress guarantees than ones relying on locks, and the most classic lock-free FIFO queue is the [Michael-Scott queue](https://doi.org/10.1145/248052.248106) (MS queue). Lock-free designs are inherently tricky to implement, and linked lists are known to be troublesome in Rust, so in this post we will explore how to implement this MS queue is Rust.

<!-- more -->

# The Algorithm Idea

The MS queue implements a linked list with lock-free *enqueue* and *dequeue* methods, using the atomic primitive [compare-and-swap](https://en.wikipedia.org/wiki/Compare-and-swap) (CAS) to achieve lock-freedom. The data structure consists of two main structs, the queue and its nodes:

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

As you can see, I here use pseudo-code which disregards memory management to just convey the idea of the algorithm. The `head` and `tail` pointers always point to a node, and the `next` pointer in a node is `null` as long as the node is the oldest in the queue (it is the tail), after which it changes to the new tail. To guarantee that `head` and `tail` always point to a node, they are initialized to point at a dummy node.

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
1. Update the `next` pointer of the current `tail` to point at the new node,
2. Update the `tail` to point at the newly enqueued node,

after which new enqueues can race to perform (1). The tricky part is to preserve lock-freedom. If the thread stalls between 1 and 2, we could get an issue of no other thread being able to make progress. To combat this, we let all threads help with step 2 , even for nodes enqueued by another thread. The pseudo-code is shown below.

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

The main complication is when the queue is almost empty, with only a single partially enqueued item (which has only passed enqueue step 1), which the dequeue must help finish before trying to enqueue it. The pseudo-code is shown below.

``` rust
impl <T> MSQueue<T> {
    pub fn dequeue(&mut self) -> Option<T> {
        loop {
            // Atomic reads
            let head = self.head;
            let next_head = head.next;
            let tail = self.tail;
            if head != tail {
                // Queue is not empty!
                // Dequeue step 1
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

Although the main algorithm is outlined above, real implementations also have some extra logic to handle memory management and to avoid the [ABA problem](https://en.wikipedia.org/wiki/ABA_problem). Memory management is a whole research field in itself, but one of the most common and versatile solutions is using [Hazard pointers](https://en.wikipedia.org/wiki/Hazard_pointer). These have some practical issues, and are not the fastest, but are very versatile and give better correctness guarantees than its alternatives.

The main problem which hazard pointers try to solve is that no node should be deallocated as long as a thread has a reference to it. For example, assume we would deallocate the previous head just before returning from a successful dequeue. This would make sense, as it is then unlinked from the data structure. However, some slow thread could still have access to it, and try to do some CAS with it at this point. If the node is freed like this, the thread which still is trying to work with it will end up with undefined behavior problems, for example running into issues of the same memory being re-used, running into the [ABA problem](https://en.wikipedia.org/wiki/ABA_problem). Hazard pointers solve the use-after-free and ABA problems in one fell sweep by ensuring no node is deallocated while another thread has a live reference to it.

The idea of hazard pointers is that each thread has a set of so called hazard pointers, which can be read by any thread, but only written to by the owner thread. When a thread wants to use a pointer, it protects it by writing it into one of its hazard pointers, only clearing the hazard pointer when it does not need the pointer anymore. Then, instead of directly deallocating a node after dequeueing it, we _retire_ the node. Periodically, together with one such _retire_, the thread iterates over all retired nodes, deallocating the ones which are not protected by the hazard pointer of any thread.

There are of course some further details of how hazard pointers work. But the base idea is that you protect a pointer while using it, which prevents other threads from deallocating it.

# Rust Implementation

# Conclusions