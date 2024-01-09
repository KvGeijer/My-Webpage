+++
title = "Zote"
date=2024-01-09

[extra]
weight = 1
show_on_homepage = true
image = { src = "projects/zote.png", alt = "My programming language!" }
link = "https://github.com/KvGeijer/zote"
+++

This is my own programming language with a mantra that the programmer should be able to "write like they think". This ended up with a focus on pipe operators and an expressive syntax.

Are you also tired of writing Python code like `print(sum(lines.split("\n")))`? Then start piping in Zote with `lines >> split("\n") >> sum >> print`. Since most grammar rules are expressions, you can simply write double for loops with a filter predicate as `for x in xs if x > 2 for y in ys { ... }`.