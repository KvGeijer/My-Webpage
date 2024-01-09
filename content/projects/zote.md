+++
title = "Zote"
date=2024-01-09

[extra]
weight = 1
show_on_homepage = true
image = { src = "projects/zote.png", alt = "My programming language!" }
link = "https://github.com/KvGeijer/zote"
+++

This is my own programming language with a mantra that the programmer should be able to "write like they think". This ended up with a focus on pipe operators, as well as most things being an expression, enabling some nice combinations.

Are you also tired of writing Python code like `print(sum(lines.split("\n")))`? Then start piping in Zote and instead write `lines >> split("\n") >> sum >> print`. Since most things are expressions (even if/loop bodies), the language does not prevent you from writing monstrosities such as `for x in xs if x != [1, 2] for y in ys if y > x { f(x, y) >> print }`. In my opinion, these design decisions makes it really easy to quickly write correct code for your small scripts, such as [Advent of Code](https://adventofcode.com/).