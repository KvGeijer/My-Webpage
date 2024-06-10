+++
title = "My Research"
description = "A few highlights from my time in academia."
template = "standalone-page.html"
+++

# Publications

- How to Relax Instantly: Elastic Relaxation of Concurrent Data Structures (Euro-Par 2024). Kåre von Geijer, Philippas Tsigas.
  - Relaxed concurrent data structures have previously been introduced as a way to parallelize inherently sequential data structures, such as queues, as the cost of their sequential semantics. This paper introduces the concept of *elastic relaxation*, where the degree of relaxation can be dynamically adjusted at run-time, and extends a previous relaxed queue and stack to encompass elasticity.
  - Accepted manuscript with better references available [here](./how-to-relax-instantly.pdf).
  - Extended ArXiv pre-print available [here](https://arxiv.org/abs/2403.13644), containing a few extra experiments and proofs.

# Master Thesis Supervision

I have had the pleasure to supervise a few students through their master's theses. The exchange with students during these projects has been very enjoyable and led to some interesting results.

Here is the list of projects I've supervised:

- [Relaxed Priority Queue & Evaluation of Locks](https://odr.chalmers.se/server/api/core/bitstreams/a667a9ce-1ccd-415c-a779-e788c2d10033/content) (2023). Andreas Rudén, Ludvig Andersson.
  - This project explored the creation of a novel relaxed priority queue with discrete priority values. Furthermore, it tried replacing the lock-free sub-queues within the relaxed concurrent queues of the [2D framework](https://doi.org/10.4230/LIPIcs.DISC.2019.31) with cache-efficient lock-based ones.