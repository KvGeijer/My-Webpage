+++
title="Early vs Late Variable Binding"
date=2023-09-26

[taxonomies]
categories = ["Programming"]
tags = ["compiler", "zote"]

[extra]
toc = false
+++

Do you lay awake at night contemplating the best scheme for variable bindings? Neither do I. However, I have thought a bit about it recently and realized it is not as simple or obvious as I once thought. In this post I give a short introduction to the late and early binding schemes and discuss their respective advantages and disadvantages.

<!-- more -->

# What differentiates early and late bindings?

We will look at a couple of scripts to illustrate the difference between these two schemes, and I want you to guess/suggest what they will/should print. These scripts will be in [Zote](https://github.com/KvGeijer/zote), which is a small scripting languagu I am currently tinkering with. Here is a simple example program:

``` rust
x := "initial";     // declares a new variable
print(x);

x = "modified";     // assigns to a variable
print(x);

x := "shadowed";    // declares a new variable, shadowing the old x
fn f() -> {         // declares a closure (function)
    print(x);
}
f();
```

This will of course print `initial`, `modified`, `shadowed`.

The following program is less clear. What do you think it should print?

``` rust
x := "initial";
fn f() -> {
    print(x);
}

x = "modified";
f();

x := "shadowed";
f();
```

In both schemes you will first print `modified`, as you print the value of `x`, which in turn has been modified since the declaration of `f`. However, there are two declared `x` variables in the scope at the second call. Which one will be used?
* Using early binding, the definition of `f` will capture the current (first) `x` variable. Therefore, the second invocation will print `modified`.
* Conversely, late binding does not capture `x` at the definition of `f`, but instead looks for the value of `x` in its scope at the time of the function call. Here, this would be `shadowed`.

Both of these schemes can be used, and you would for the most part not notice the difference. Early binding is often considered to be the desired behaviour, causing fewer bugs. However, it also has its issues, and the choice of which one to use often depends on the complexity of the interpreter/compiler.

# Simple implementation of late binding

Late binding is often used in toy interpreters which directly interpret syntax trees, such as school projects or prototypes, as it is very simple to implement. For example, this is what I did in my [first interpreter for Zote](https://github.com/KvGeijer/zote/ast_interpreter).

The implementation keeps a stack of dictionaries (hash maps) that maps variable names to their values. When entering a new local scope (such as a loop body), a new dictionary is pushed to the stack, and when a variable `x` is declared we create a new entry in the topmost dictionary binding `x` to its value. When leaving a local scope, the top of the stack is popped, effectively de-allocating all variables in that scope. Finally, when accessing or assigning to a variable, we search for a binding with its name in the dictionary stack, starting at the top and continuing down until we find a match.

As this dictionary stack does all its lookups by variable name at access time, it becomes late binding. However, it could be made to use early binding by extending each variable name with a unique number, similarly to how [static single assignment](https://en.wikipedia.org/wiki/Static_single-assignment_form) is implemented in compilers to convert all variables to constants.

The main disadvantage to this dynamic variable lookup is that it is incredibly slow. We spread memory all over the place in numerous dictionaries, which are created and allocated constantly. Furthermore, even though hash maps have O(1) complexity for operations, the hashing still has some overhead, and we often have to do several lookups if the variable is not declared in the top-most scope.

# Compiling with early binding

When compiling a program, either to bytecode or native code, almost all compilers resolve variable bindings compile time. This is done to minimize the overhead of dynamic lookups, and also aligns perfectly with early variable bindings.

Exactly how this is implemented varies a bit, especially when it comes to the capturing of variables in closures. But the main idea is that all variables are stored at a stack, that is made up of a _stack frame_ for each function call. The offset of each variable from the current stack frame can be known at compile time, so all variable lookups basically just add this offset to the current stack frame address to get the exact address. As shadowing a variable declares a new one and leaves the old one on the stack, they will both co-exist with different addresses. The compilation will make their references use the different offsets and we will therefore get early binding.

## Problem with early binding

Early binding is used in many languages, but has one issue in my opinion. To illustrate it. What do you think should happen here?

```rust
fn f1(x) -> {
    if x == 0 {
        print("base-case");
    } else {
        print(x);
        f1(x - 1);
    }
}

fn f2(x) -> {
    f1(x);
}

f2(2);
```

Here `f1` is a recursive function, and I hope we can agree that this should print `2`, `1`, `base-case`. However, what happens if two functions happened to be mutually dependent? Take this example:

```rust
fn f1(x) -> {
    if x == 0 {
        print("base-case f1");
    } else {
        print(x);
        f2(x - 1);
    }
}

fn f2(x) -> {
    if x == 0 {
        print("base-case f2");
    } else {
        print(x);
        f1(x-1);
    }
}

f2(2);
```

In this toy example we want `f1` and `f2` to call each other. The programmer wanted the effect to be the same as the last script, but for this to print differently depending on where the base case is reached. However, do you see why this would be problematic to run in a language with early binding?

When we compile `f1`, we must now know the offset of `f2` on the stack. But as we compile `f1` before `f2`, this becomes impractical. Compilers such as the ones for C only traverse the code once, generating code as they go, and would give a compile error for code like this as `f2` is referenced before its declaration. This is be acceptable in C, and can be  avoided by separating the function definition and declaration, but it is in my opinion not great if you want to implement a pleasant scripting language.

## Early binding in Rust

This is a bit of a tangent, but similarly to C, Rust uses early binding, and I think it has a really nice solution to the problem above. How do you think it does this?

The way they solve this is by making a distinction between functions (defined with `fn(args) {body}`) and closures (defined with `|args| body`). I won't go into detail about how this works here, but functions don't close over their contained variables, cannot be returned out of scope, and are a bit more light-weight.

Here is a rust program, using global functions, equivalent to the Zote example above:

```rust
fn main() {
    f2(2)
}

fn f1(x: usize) {
    if x > 0 {
        println!("{x}");
        f2(x - 1)
    } else {
        println!("base-case f1")
    }
}

fn f2(x: usize) {
    if x > 0 {
        println!("{x}");
        f1(x - 1)
    } else {
        println!("base-case f2")
    }
}
```

This works as the programmer intended and outputs `2`, `1`, `base-case f2`. However, if we were to do this with local closures, it would not work, as Rust variables (including closures) use early binding.

```rust
fn main() {
    let f1 = |x| {
        if x > 0 {
            println!("{x}");
            f2(x - 1)
        } else {
            println!("base-case f1")
        }
    };

    let f2 = |x| {
        if x > 0 {
            println!("{x}");
            f1(x - 1)
        } else {
            println!("base-case f2")
        }
    };

    f2(2)
}
```

This fails in compilation and complains that `f2` is unknown.

Now you might assumed that this is because the global scope is handled differently from local scope. After all, Rust is not a scripting language and we cannot declare variables in global scope. So what would happen if we use local functions?

```rust
fn main() {
    fn f1(x: usize) {
        if x > 0 {
            println!("{x}");
            f2(x - 1)
        } else {
            println!("base-case f1")
        }
    }

    fn f2(x: usize) {
        if x > 0 {
            println!("{x}");
            f1(x - 1)
        } else {
            println!("base-case f2")
        }
    }

    f2(2)
}
```

This actually works, giving the same output as with global functions! Essentially, the compiler looks ahead in each scope for function definitions and assigns their offsets before it starts compiling the functions, where the offsets are needed. To make this play nice, Rust forbids declaring two functions with the same names in each scope.

Rust can do this as its compiler first build up a syntax tree of the whole code base before starting to output compiled code. On the other hand, as C was created in the days when such a syntax tree would not fit in memory, it outputs compiled code directly when scanning the code. This means that C would have to look an indefinite amount of tokens forward before starting the compilation of each function if they wanted to do a similar lookahead to Rust, which would not be practical.

# Choice of binding scheme for Zote

So, how should I do variable binding for my virtual machine for Zote? I want mutually dependent functions to work out of the box, but I also want to keep the language small, not separating closures and functions as Rust did.

Instead, I observe that closures defined in local scopes are usually not mutually dependent with other closures. On the other hand, closures defined at global level are often more complicated and can be mutually dependent (or just defined in a different order than their dependencies). Therefore, I will use late binding for all global variables, and early binding for all locals (implemented with a lookahead on the syntax tree, with re-declarations overwriting the old value), which I hope will create a good user experience.

This choice of distinguishing global and local variables is also practical, as store globals are stored in a different persistent region of the stack than locals, as they must be persistent between REPL inputs.

# Takeaways

This topic, like many others in the area, is more intricate than one would think at first. Early binding probably gives less error-prone code, but can cause some headaches for mutually dependent functions. Late binding, aside from probably being a bit more error-prone, is also not obvious how to implement as efficiently as early binding.

My main takeaway is that early binding probably is the way to go, but that it needs some extra help to not be dependent on the order of function definitions. Furthermore, Rust has a great solution to this, which does not need to distinguish between local and global scopes, and seems like it would work perfectly for scripting languages as Zote.



