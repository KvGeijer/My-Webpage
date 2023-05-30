+++
title = "Welcome to my page!"
description = "I am Kåre and here I'll talk about everything from parallel algorithms to rock climbing"
+++

This page is mainly a collection of posts about varying things. The [Blog](./blog) contains a variety of topics I find interesting, such as personal projects or experiences. For example, I have a [post](./blog/confounding-lifetimes/) about lifetimes in Rust. Then the [Research](./research) feed contains summaries and posts about interesting research I have come across and is less approachable than the [blog](./blog). But if you want to try a research summary I recommend [Data Structures in the Multicore Age](./research/data-structures-in-the-multicore-age/) which introduces the field of multicore programming and the need for relaxed data structures.

I'm currently a PhD student in the Distributed Computing and Systems group at Chalmers University of Technology. Here I research how to speed up massively parallel computations by relaxing semantics of algorithms, such as allowing queues to return any of the top ten elements instead of the top. Previously I briefly worked at [Cisco](https://www.cisco.com) in Stockholm, developing their network automation products. I have a masters in Engineering mathematics from Lund University where I did a lot of fun things during my five years.

### Some projects I have done

I started writing my own programming language [zote](https://github.com/KvGeijer/zote) at the start of 2023. I enjoyed my course in compilers and thought this would be a perfect side project. It is a dynamically typed scripting language with the idea of being able to write code in an ergonomic way. Here is an example of how you can solve the [first day of Advent of Code 2022](https://adventofcode.com/2022/day/1) in the language:
```rust 
read("input_file_path")
    >> split("\n")
    >> split("")
    >> map(elf -> elf >> map(int) >> sum)
    >> sort
    >>: elfs;

elfs[0] >> print;
elfs[0:3] >> sum >> print; 
```

I wrote a small [programming puzzle](https://github.com/KvGeijer/Arborist-Puzzle) for my roommate, inspired by [Advent of Code](https://adventofcode.com/), which requires you to implement a very simple interpreter.

The past three years I have had the pleasure of doing [Advent of Code](https://adventofcode.com/) every December, which is an advent calendar of programming puzzles with increasing difficulty. I find it quite fun to compare my solutions over the years as it shows how I have changed as a programmer.
* The [first year](https://github.com/KvGeijer/Advent_of_Code_2020) I started writing Haskell. But then I got a job as a Python developer and switched to Python. The code is mostly quite clever, but some of the [later days](https://github.com/KvGeijer/Advent_of_Code_2020/blob/main/Day20/day20.py) have rather ugly solutions.
* The [second year](https://github.com/KvGeijer/Advent_of_Rust) I did it in Rust, as I had been exposed to the language in a lab at university. I discussed the solutions with two friends who did the same which was fun, and I focused on writing clean code.
* The [third year](https://github.com/KvGeijer/Advent-of-Julia) I programmed in Julia and tried switching from VS Code to the terminal-based editor [Helix](https://helix-editor.com/). In addition, I tried to solve them quickly for the first time which was a ton of fun. I did ok, but have a lot to learn about competitive programming.
* I have also started a [repo](https://github.com/KvGeijer/Advent-of-Variety) for solving every puzzle from 2017 in different (new) languages. We will see if I will finish the last couple of days someday.
