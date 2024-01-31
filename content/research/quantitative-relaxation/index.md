+++
title="Quantitative Relaxation of Concurrent Data Structures"
date=2023-05-16

[taxonomies]
categories = ["Research summary"]
tags = ["relaxed semantics", "lock-free"]

[extra]
toc = true
+++

In the middle of a flurry of work on relaxed semantics, Henzinger et al. published the paper [Quantitative Relaxation of Concurrent Data Structures](https://dl.acm.org/doi/10.1145/2480359.2429109) in 2013. Its primary contribution is ironing out a framework for the theory of relaxed semantics. The idea is to allow incorrect transitions between states in the data structure specification while being able to associate different paths or transitions with different costs. Furthermore, they create a relaxed out-of-order stack. Here we will concisely but informally present the theory, and in the end, quickly outline the stack.

<!-- more -->

If you want a broader introduction to the motivation of semantic relaxation, and multicore programming, I recommend reading my earlier post [Data Structures in the Multicore Age](../data-structures-in-the-multicore-age). But the main idea with relaxation is that sequential data structures like stacks cannot scale forever, as there will be contention on the first element. By relaxing the stack we can create several access points and alleviate contention, at the cost of the sequential specification.

# Specifying sequential data structures
To specify relaxed data structures we need to be able to first specify sequential ones. This builds up to specifying a labeled transition system (LTS), which defines possible states and allowed transitions for each data structure.

A *sequential history* **s** is a sequence over the labeled methods of a data structure (including input and output values). So for a stack, an example could be `push(1), push(2), pop(2), push(3)`. A data structure is then *specified* by the set of all allowed sequential histories, such that the semantics are upheld.

A core concept is that two linear histories can be considered *S-equivalent* (S comes from sequential specification, but I intuitively think of state) if they correspond to the same state. For example, `push(1), push(2), pop(2), push(3)` and `push(1), push(3)` are equivalent under this relation. They define this by saying that two histories are equivalent if any valid continuation to one of them is valid for the other one, which I find quite elegant.

This can be used to define *kernel histories* for each such equivalence class as the shortest history in the class. For a stack, this would be a history that only pushes the items in the stacks, without any pops which would just add extra calls.

Finally, they define the Labeled transition system (LTS) of a data structure as (Q, Σ, →, {% katex(block=false) %}q_0{% end %}), made up by the following:
* The set of allowed states Q, which is the set of all allowed kernels.
* A set of labels Σ, which are the possible labled methods.
* A transition relation →, which encodes all allowed transitions between states caused by a method.
* An initial state {% katex(block=false) %} q_0 {% end %}, which is most often the empty kernel.


# Theory of quantitative relaxation

Now we are ready to incorporate relaxation into this theory, being able to specify relaxed versions of the sequential structures. At a high level, we will allow the transition between any two valid states, but associate each transition (or more generally, sequence of transitions) with a cost.

To relax a data structure we start by extending the transition relation → to {% katex(block=false) %} Q \times \Sigma \times Q {% end %}, which contains the transition from any state to any other state, with any labeled method (including input/output values). This is called the *completed labeled transition system*.

From that, we create the *quantitative labeled transition system* (QLTS), which has an added _cost_ function that maps the transition domain to a (well-ordered) cost domain *C* (think nonnegative numbers). If the transition is allowed in the normal LTS the cost should always be 0.

A *quantitative path* {% katex(block=false) %} \kappa {% end %} is a sequence of states, transitions and their corresponding costs from the QLTS. The *quantitative trace* {% katex(block=false) %} \tau \in (\Sigma \times C) {% end %} for a path (also notated qtr(κ)) is the sequence of labeled methods and their costs. Furthermore, the *trace* tr(qtr(κ)) = tr(κ) = **u** is the sequence of labeled method calls in the qtr(κ) (so just disregarding the costs). Finally, they denote qtr(**u**) as the set of all quantitative traces with trace **u**, and similarly qtr(*S*) as all possible traces for *S*.

Furthermore, they define the *path cost function* pcost(τ), as a monotone mapping from a quantitative trace qtr(S) to a cost. If path A is a prefix to B, then A cannot cost more than B. This way we can map whole paths to costs, instead of just one transition. For example, a common mapping might be the max of all transition costs, but a more advanced one might be the minimum over paths of a specific length.

Finally, they define a *k-relaxed* data structure for some cost k, as the specification containing all traces **u** where the pcost(qtr(**u**)) is less than k. <!-- Note that there are many quantative traces in qtr(**u**) for each **u**, but the cheapest one is used (meaning that we choose the minimal costs at each steps given the behavior) -->

With this definition, we can create a quantitatively relaxed data structure for any sequential data structure, as long as the relaxation has a hard bound. This means probabilistic relaxations like the [MultiQueue](https://dl.acm.org/doi/10.1145/2755573.2755616) cannot be covered by this. For that, I recommend reading the paper [Distibutionally Lenarizable Data Structures](https://dl.acm.org/doi/10.1145/3210377.3210411) which extends this definition for these randomized relaxations.

# Out-of-order relaxation

The out-of-order relaxation is the most widely used one in the literature, and now we have all the tools to define it quantitatively.

Intuitively this relaxation says that the cost of a transition is the shortest number of methods to apply to make the transition valid in the sequential LTS.

Formally they define this by looking at the transition from *q* to *q'* with the method *m*. The transition cost is given by the length of **v**, where **v** is a kernel (of minimal length) for which one of the following must hold:
1. **Removing v**. The trace **uvw** is a kernel of the initial state q, **uw** is also a kernel sequence and m sequentially transitions either ([**s**] is the equivalence class of the sequence **s**)
    * [**uw**] to [**u'w**] where *q'* = [**u'vw**], or
    * [**uw**] to [**uw'**] where *q'* = [**uvw'**].

    So we transition **uvw** to **u'vw** or **uvw'**, but sequentially we can only go from **uw** to **u'w** or **uw'**. Meaning, **v** is the shortest sub-trace we can remove from the kernel trace to make the transition valid.
2. **Adding v**. The trace **uw** is a kernel of the initial state q, **uvw** is also a kernel sequence and m sequentially transitions either
    * [**uvw**] to [**u'vw**] where *q'* = [**u'w**], or
    * [**uvw**] to [**uvw'**] where *q'* = [**uw'**].

    So we transition **uw** to **u'w** or **uw'**, but sequentially we can only go from **uvw** to **u'vw** or **uvw'**. Meaning, **v** is the shortest sub-trace we can insert into the kernel trace to make the transition valid.

I think this is a rather nice relaxation. Firstly, it is quite simple and only bounds individual transitions instead of creating a more complex path cost function. Secondly, as with all relaxations in this framework, it can be applied to only a subset of labeled methods. In the simplest case, this means you can apply it only to pop calls. But you can also specify which type of pop calls (as they are parametrized by the return value), such as not allowing empty pops to be relaxed. This makes sense in stacks and queues, but I wonder if it is as easy for more complicated structures.

A neat extension is that they define a lateness relaxation in connection with the cost defined by out-of-order. A lateness k-stack (relaxed pops only) will, at least once every k pops pop the top. To formalize this they create a cost function for path segments with k pop operations, such that the cost is the minimum transition cost for any pop. This path cost must then always be 0.

Furthermore, this out-of-order relation can be applied similarly to other data structures such as counters, queues, priority queues and so on. The [article](https://dl.acm.org/doi/10.1145/2480359.2429109) goes more into depth on more versions of the out-of-order relaxation, as well as defining a stuttering relaxation. But for brevity, I chose to skip that.

# The relaxed k-Stack

Their relaxed stack builds on an [earlier relaxed FIFO queue](https://link.springer.com/chapter/10.1007/978-3-642-33078-0_20) and its idea to keep a list of unordered *k-segments* in place of nodes in the easiest linked lists (such as Treiber Stack for stacks). A k-segment is simply an array with k atomic items (really empty, or an item such as a pointer). So like the Treiber stack the k-stack is a linked list of k-segments updated atomically. To push (pop) an item you select one empty (filled) index in the top segment and linearize the operation with a CAS.

The complicated part with the stack is if the top segment is full or empty, and we need to push a new one, or pop the current one, to allow further operations. Popping the top segment while another thread tries to push an item in it could lead to lost items, so they have to take extra care that this does not happen, which becomes a bit messy.

In the ideal case, this stack allows k concurrent operations, each accessing a separate index. But there are some problems. Changing the top will be sequential, but should happen quite rarely compared to normal operations (if relaxation is high enough). The main problem is that each operation needs to find a valid index to operate on. This can take time when the top is either almost full or empty. If you are interested in this problem I recommend reading about the [relaxed 2D framework](https://drops.dagstuhl.de/opus/volltexte/2019/11338/pdf/LIPIcs-DISC-2019-31.pdf), which is similar, but sort of incorporates a depth into each segment to make it easier to find a valid index.

Furthermore, they add a linearizable emptiness check (as opposed to being allowed to return empty if there are fewer than k items). This is achieved by linearly looping over all indexes in the top, and if they are empty during the first pass and have not been updated on the second pass, then it is truly empty. This corresponds theoretically to only relaxing non-empty pop operations, which is neat and very useful for many applications.

# Conclusions

This has been quite a dense summary, of a rather theoretical paper, so well done if you made it this far. Usually, this level of theory is not needed, but if you want to come up with new relaxations it is probably a good idea to have a solid grasp on some theory to anchor it in (although, you could just define a simpler one yourself). For example, I wanted to know if it was possible to easily relax both pushes and nonempty-pops for an out-of-order stack, and rereading this I see that you can, which is nice (now comes the hard part about proving the correctness of such a stack...).

On the practical side, the k-stack is a nice stack and scales well. It has some tricky checks for popping the top segment, but other than that it is quite elegant.

If you are interested in further reading I recommend [Distibutionally Lenarizable Data Structures](https://dl.acm.org/doi/10.1145/3210377.3210411) for a similar paper about the theory of randomized relaxed data structures. If you want to read about some relaxed designs I recommend the [relaxed 2D framework](https://drops.dagstuhl.de/opus/volltexte/2019/11338/pdf/LIPIcs-DISC-2019-31.pdf) for stacks and queues etc., or the [MultiQueue](https://dl.acm.org/doi/10.1145/2755573.2755616) which is an elegant relaxed priority queue achieving performance and relaxation through [the power of two](https://ieeexplore.ieee.org/document/963420).
