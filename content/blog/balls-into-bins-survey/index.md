+++
title="The Power of Two Choices: A brief survey"
date=2024-07-04

[taxonomies]
categories = ["Research Insights"]
tags = ["balls-into-bins", "power-of-two"]

[extra]
toc = true
+++

Consider a process where _n_ balls are inserted at random into _n_ bins. Then, on average, the most heavily loaded bin will contain an order of _log(n)/log(log(n))_ balls. However, if balls instead are inserted into the least loaded of 2 randomly chosen bins, then the maximum load decreases to an order of _log(log(n))_. Essentially, choosing the best of two randomly selected choices gives a way more stable system! This _two-choice-paradigm_ has far reaching implications, and is used in settings ranging from [lock-free data structures](https://link.springer.com/chapter/10.1007/978-3-642-33078-0_20) to [hashing](https://en.wikipedia.org/wiki/Cuckoo_hashing).

<!-- more -->

Due to its far reaching implications, I came across this problem when researching the theoretical guarantees of the [MuliQueue](https://dl.acm.org/doi/10.1145/2755573.2755616), as analyzed in the paper [The Power of Choice in Priority Scheduling](https://dl.acm.org/doi/abs/10.1145/3087801.3087810). If you want to read more about this MultiQueue, which is an efficient relaxed priority queue, I already have an introductory post about it [here](@/blog/multiqueue-introduction/index.md). Instead, this post will mainly summarize [The Power of Two Random Choices: A Survey of Techniques and Results](https://www.eecs.harvard.edu/~michaelm/postscripts/handbook2001.pdf) from 2001, which is a survery paper about the balls-into-bins process by Michael Mitzenmacher et al.

# Introduction

The simplest use-case is that of dynamically assigning tasks to servers. Assuming all servers and tasks are identical, and that tasks arrive sequntially, then we naturally want to keep the maximum load (the number of un-completed tasks at a server) as low as possible. Ideally, we would assign each task to the least loaded server, but the load of all servers can be expensive to obtain. Instead, querying only two servers and assigning the task to the least loaded one achieves close to the same load balancing, at a fraction of the cost.

However, the majority of analysis has been done on the balls-into-bins problem, which other problem then can be reduced to. Here the problem is often divided into the _static_ and _dynamic_ versions. In _static_ scenarios, balls are inserted and never deleted. In _dynamic_ scenarios on the other hand, balls can also be deleted from the system. There are many different twists on this problem, but this is a recurring and useful distinction.

Another distinction is between _sequential_ and _parallel_ systems. In _sequential_ ones, tasks arrive and are assigned sequentially. In the parallel case, they instead arrive in batches, and if it is also _dynamic_ each bin removes one ball every iteration. The paper [Analysing an Infinite Parallel Job Allocation Process](https://link.springer.com/chapter/10.1007/3-540-68530-8_35), they analyze this _parallel dynamic_ setting from a queueing perspective, where they want to minize the maximum wait-time for each ball (task).

## Uses

Maybe go more into depth of some of the use cases. For example a paragraph or so on each.
- MultiQueue
- Cuckoo hashing
- More?


## Three main techniques

The paper highlights three main techniques used to analyze the balls-into-bins problems. In the following sections, we will look closer at each of them.
- **Layered induction**: Here we bound the maximum load by bounding the number of bins with higher load than *j*, via induction over *j*. This is a straightforward method, that provides _nearly_ tight results, and can for example handle a dynamic setting where an adversary removes balls.
- **Witness trees**: This technique inspects the occurence of a "bad event", where one bin is heavily loaded. From that state, it extracts its history as a tree of events, and bound the probability of the bad event by its occurence in the witness tree. This is the most challenging technique presented here, but is also the most powerful one, especially proving useful in dynamic settings.
- **Fluid limits via differential equations**: By letting the number of bins tend to infiinty, the resulting system can often be described as a family of differential equations. This is a common approach in queueing theory, and naturally fits the balls-into-bins problems which can map onto queueing ones. While slightly more situational than the other methods, it is the simplest and most flexible method when it fits the system.

# Layered Induction

This technique was first introduces in the paper [Balanced Allocations](https://epubs.siam.org/doi/10.1137/S0097539795288490) from 1999. The idea is to inductively bound the number of bins that contains more than _j_ balls conditioned on the number of bins that contain at least _j-1_ balls.

A strong use of this argument was used in the paper [On Balls and Bins with Deletions](https://link.springer.com/chapter/10.1007/3-540-49543-6_12) from 1998, where they managed to analyze systems with adversarial deletions.

## The Basic Approach

<!-- Maybe we want to separate this into several sub-sections? Now basic good enough? -->
The first use of this approach analyzed the most common sequential balls-into-bins problem (_d_ does not have to be 2, leading to a choice-of-_d_):

> **Theorem 1.** Suppose that _n_ balls are sequentially placed into _n_ bins. Each
ball is placed in the least full bin at the time of the placement, among _d_ bins,
_d ≥ 2_, chosen independently and uniformly at random. Then after all the
balls are placed, with high probability the number of balls in the fullest bin is
at most _log log n / log d + O(1)_.

### Intuitive proof sketch

First, we note that it is easier to study the number of bins with load _larger than_ _i_ than to study the ones with exact load _i_. They define the _height_ of a ball as one more than the number of balls already in the bin into which the ball is placed, which is a good definition for future use. Let {% katex(block=false) %} \beta_i {% end %} be an upper bound on the number of bins at least loaded with _i_ balls. This {% katex(block=false) %} \beta_i {% end %} is what we want to bound with high probability, using induction.

The induction goes as:
1. Suppose we know {% katex(block=false) %} \beta_i {% end %}, which bounds the number of bins with a load of at least _i_ during the entire process.
2. We with to find {% katex(block=false) %} \beta_{i+1} {% end %}, such that, with high probability, the number of bins with load at least *i+1* is bounded above by {% katex(block=false) %} \beta_{i+1} {% end %} over the entire process. This {% katex(block=false) %} \beta_{i+1} {% end %} is found by bounding the number of balls with height at least _i+1_.

The probability of a ball having height above *i+1* is conditioned on {% katex(block=false) %} \beta_i {% end %}, as its random choice of _d_ bins all have to have load above _i_. This leads to the probability of a ball having height at least _i_ to be {% katex(block=false) %} (\frac{\beta_i}{n})^d {% end %}. When {% katex(block=false) %} d \geq 2 {% end %}, this sequence of {% katex(block=false) %} \frac{\beta_i}{n}  {% end %} drops at least quadratically in size, and can be bounded by Bernoulli trials as {% katex(block=false) %} \beta_{i+1} \leq cn\large(\frac{\beta_i}{n}\large)^d{% end %} for some constant _c_. Therefore, if _j = O(log log n)_, {% katex(block=false) %} \beta_j \lt 1 {% end %}, meaning that there with high probability is no such heavily loaded bin.

#### Understanding the statistics

Looking closer at the statistics, they state that:
> The number of balls with height _i+1_ or more is stochastically dominated by a Bernoulli random variable, corresponding to the number of heads with _n_ (the number of balls) flips, with the probability of a head being {% katex(block=false) %} \large(\frac{\beta_i}{n}\large)^d{% end %} (the probability of a ball being placed in a bin with _i_
or more balls). We can find an appropriate {% katex(block=false) %} \beta_{i+1} {% end %} using standard bounds on Bernoulli trials, yielding {% katex(block=false) %} \beta_{i+1} \leq cn\large(\frac{\beta_i}{n}\large)^d{% end %}, for some constant _c_.

So, for those of us who have forgotten statistics: What is a [Bernoulli random variable](https://en.wikipedia.org/wiki/Bernoulli_process)? It is essentially a random variable with two outcomes, where each outcome has a static probability. This can be seen a flipping a potentially unfair coin.

In this situation, the Bernoulli variable models wether ball _j_ has {% katex(block=false) %} \textit{height} \geq \beta_{i+1} {% end %}. They model this probability as static through the whole process as {% katex(block=false) %} p = \large(\frac{\beta_i}{n}\large)^d {% end %}. This is a simplification, as this essentially only holds for the last ball and that the first couple of balls have _0_ probability of reaching height _i_.

Now that we have a Bernoulli random variable with a static probability _p_, we want to bound the number of successful ({% katex(block=false) %} \textit{height} \gt \beta_{i+1} {% end %}) outcomes. This is modeled as a [binomial distribution](https://en.wikipedia.org/wiki/Binomial_distribution), which essentially models a random variable for the number of successfull outcomes of a Bernoulli variable. The binomial random variable thus has two variables _B(n, p)_: the number of trials _n_, and the probability of success _p_.

In summary, they key part is that they model the probability of each ball heigt being larger than {% katex(block=false) %} \beta_{i+1} {% end %} as a Bernoulli variable with probability {% katex(block=false) %} (\frac{\beta_i}{n})^d {% end %}. The number of balls with {% katex(block=false) %} \textit{height} \geq \beta_{i+1} {% end %} can be modeled as a binomial random variable {% katex(block=false) %} B(n, (\frac{\beta_i}{n})^d) {% end %}. Finally, they claim there is a standard Bernoulli trial bound that yields {% katex(block=false) %} \beta_{i+1} \leq cn\large(\frac{\beta_i}{n}\large)^d{% end %}, for some constant _c_. That means {% katex(block=false) %} \frac{\beta_i}{n} \leq c\large(\frac{\beta_i}{n}\large)^d {% end %} drops quadratically each step (we should need some guarantee on {% katex(block=false) %} c \le 1{% end %} as well?). Therefore after only _O(log n)_ steps, the fraction has dropped below _1/n_! (**TODO** Why is it not _log(n)_?).

{% katex(block=false) %} \frac{\beta_{i+1}}{n} \leq (\frac{\beta_{i}}{n})^2 {% end %}. Therefore, if _j = log(n)_, {% katex(block=false) %} \beta_{j+1} \le n {% end %}

**TODO**: Here are just some thoughts about how to dig deeper.

If one wants to investigate this binomial distribution further, its probability density function, becomes {% katex(block=false) %} f(k, n, p) = \textit{Pr}(X = k) = \binom{n}{k}p^k(1-p)^{n-k} = \binom{n}{k}(\frac{\beta_i}{n})^{d k}(1-(\frac{\beta_i}{n})^d)^{n-k} {% end %}, where _k_ is the number of successes.

We also know that {% katex(block=false) %} \beta_0 = n {% end %}, which means that {% katex(block=false) %} \beta_{i+1} \leq  {% end %}

### More technical proof

Notation:
- Let the state at time _t_ be the state of the system after ball _t_ is placed.
- _B(n, p)_ is a binomial random variable (they call it Bernoulli in the paper) with parameters _n_ nad _p_.
- _h(t)_ denotes the height of ball _t_.
- {% katex(block=false) %} \nu_i(t) {% end %} denotes the number of bins with load at least _i_ at time _t_.
- {% katex(block=false) %} \mu_i(t) {% end %} denotes the number of balls with height at least _i_ at time _t_.
- They use {% katex(block=false) %} \nu_i, \mu_i {% end %} as {% katex(block=false) %} \nu_i(n), \mu_i(n) {% end %} when the meaning is clear.
<!--
They start out with two helpfull and elementary lemma. The first one describes sums of Bernoulli variables and binomial distributions with coupled random variables:

> **Lemma 2** Let {% katex(block=false) %} X_1 {% end %}, {% katex(block=false) %} X_2 {% end %}, ... {% katex(block=false) %} X_n {% end %} be a sequence of random variables in an arbitrary domain, and let {% katex(block=false) %} Y_1 {% end %}, {% katex(block=false) %} Y_2 {% end %}, ... {% katex(block=false) %} Y_n {% end %} be a sequence of binary random variables, with the property that {% katex(block=false) %} Y_i = Y_i(X_1,...,X_{i-1}) {% end %}. If
{% katex(block=true) %} \textit{Pr}(Y_i = 1| X_1,...,X_{i-1}) \leq p, {% end %}
then
{% katex(block=true) %} \textit{Pr}(\sum_{i=1}^n Y_i \geq k) \leq \textit{Pr}(B(n, p) \geq k), {% end %}
and similarly, if
{% katex(block=true) %} \textit{Pr}(Y_i = 1| X_1,...,X_{i-1}) \geq p, {% end %}
then
{% katex(block=true) %} \textit{Pr}(\sum_{i=1}^n Y_i \leq k) \leq \textit{Pr}(B(n, p) \leq k). {% end %}

The second lemma describes Chernoff-type bounds

> **Lemma 3** If {% katex(block=false) %} X_i ~~ (1 \leq i \leq n){% end %} are independent binary random variables, {% katex(block=false) %} \textit{Pr}(X_i = 1) = p {% end %}, then the following hold:
{% katex(block=true) %} \textit{For} t \geq np, \hspace{2em} \textit{Pr}(\sum_{i=1}^nX_i \geq t) \leq (\frac{np}{t}^te^{t-np}). {% end %}
{% katex(block=true) %} \textit{For} t \leq np, \hspace{2em} \textit{Pr}(\sum_{i=1}^nX_i \leq t) \leq (\frac{np}{t}^te^{t-np}). {% end %}
In particular, we have
{% katex(block=true) %} \textit{Pr}(\sum_{i=1}^nX_i \geq enp) \leq e^{np}. {% end %}
{% katex(block=true) %} \textit{Pr}(\sum_{i=1}^nX_i \leq \frac{np}{e}) \leq e^{(\frac{2}{e} - 1)np}. {% end %} -->

Now we can tackle **Theorem 1** properly. To not bore you as the reader, I will here also just keep a sketch, outlining interesting tidbits deviating from the sketch above, and recommend the [original article](https://www.eecs.harvard.edu/~michaelm/postscripts/handbook2001.pdf) for the full details.

With the new notation, we want to construct values {% katex(block=false) %} \beta_i {% end %} such that {% katex(block=false) %} \nu_i(n) \leq \beta_i {% end %} for all _i_ whp. They start out with letting {% katex(block=false) %} \beta_6 = \frac{n}{2e} {% end %} and {% katex(block=false) %} \beta_{i+1} = \frac{e\beta_i^d}{n^{d-1}}, {% end %} for {% katex(block=false) %} 6 \le i \lt i^* {% end %}, where {% katex(block=false) %} i^* {% end %} is to be determined.

#### Induction up to {% katex(block=false) %} i^* {% end %}

They define {% katex(block=false) %} \epsilon_i {% end %} as the event that {% katex(block=false) %} \nu_i(n) \leq \beta_i {% end %}, which essentially says that the {% katex(block=false) %} \beta {% end %} actually bounds the bins. With basic math {% katex(block=false) %} \epsilon_6 {% end %} holds, and they use induction to prove that it holds for all other _i_ up to {% katex(block=false) %} i^* - 1 {% end %}.
