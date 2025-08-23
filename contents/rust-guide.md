---
title: "Rust Programming Guide"
description: "A comprehensive guide to Rust programming language with examples and best practices"
author: "Typstify Team"
tags: ["rust", "programming", "systems", "guide"]
draft: false
---

## Introduction

Welcome to the comprehensive Rust programming guide! Rust is a systems programming language that focuses on safety, speed, and concurrency.

## Basic Syntax

### Variables

In Rust, variables are immutable by default:

```rust
fn main() {
    let x = 5; // immutable
    let mut y = 10; // mutable
    
    println!("x = {}, y = {}", x, y);
    
    y = 15; // This is allowed because y is mutable
    println!("y is now {}", y);
}
```

### Functions

Functions in Rust are declared with the `fn` keyword:

```rust
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b // No semicolon means this is returned
}

fn main() {
    let result = add_numbers(5, 3);
    println!("5 + 3 = {}", result);
}
```

### Ownership and Borrowing

One of Rust's key features is its ownership system:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // s1 is moved to s2
    
    // println!("{}", s1); // This would cause a compile error
    println!("{}", s2); // This works
    
    let s3 = String::from("world");
    let len = calculate_length(&s3); // Borrowing
    println!("The length of '{}' is {}.", s3, len);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

## Error Handling

Rust uses `Result<T, E>` for error handling:

```rust
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    let f = File::open("hello.txt");

    let f = match f {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating the file: {:?}", e),
            },
            other_error => {
                panic!("Problem opening the file: {:?}", other_error)
            }
        },
    };
}
```

## Structs and Enums

### Structs

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
    
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    println!("The area of the rectangle is {} square pixels.", rect1.area());
}
```

### Enums

```rust
enum IpAddrKind {
    V4(u8, u8, u8, u8),
    V6(String),
}

fn main() {
    let home = IpAddrKind::V4(127, 0, 0, 1);
    let loopback = IpAddrKind::V6(String::from("::1"));
}
```

## Traits

Traits define shared behavior:

```rust
trait Summary {
    fn summarize(&self) -> String;
}

struct NewsArticle {
    headline: String,
    location: String,
    author: String,
    content: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}, by {} ({})", self.headline, self.author, self.location)
    }
}

struct Tweet {
    username: String,
    content: String,
    reply: bool,
    retweet: bool,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}
```

## Concurrency

Rust makes concurrent programming safer:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("hi number {} from the spawned thread!", i);
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..5 {
        println!("hi number {} from the main thread!", i);
        thread::sleep(Duration::from_millis(1));
    }

    handle.join().unwrap();
}
```

## Useful Tips

> **Note**: Always prefer using `cargo` to manage your Rust projects. It handles dependencies, building, and testing automatically.

### Common Commands

- `cargo new project_name` - Create a new project
- `cargo build` - Build the project
- `cargo run` - Build and run the project
- `cargo test` - Run tests
- `cargo doc --open` - Generate and open documentation

### Best Practices

1. **Use descriptive variable names**
2. **Write tests for your functions**
3. **Handle errors explicitly**
4. **Use the type system to your advantage**
5. **Follow the Rust API guidelines**

## Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/stable/rust-by-example/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/)
