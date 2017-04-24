# woodpecker - Logging for [Rust][rust]

### Table of Contents

* [Status](#status)
* [`woodpecker` crate in your project](#in-your-project)
* [Features](#features)
* [License](#license)
* [Credits](#credits)

### Introduction

`woodpecker` is a logging framework for [Rust][rust].
The goal is to have a fast, extensible and easy logging in [Rust][rust] application.

### Status

The project is currently under development and doesn't provide a lot of features.
Although the basic feature, logging is well supported!

### Features
The main feature is almost zero overhead if no filtering rules are defined and log is not produced.

Currently supported:
* pluggable format function
* multiple log consumers
* filtering by module (any part of the module path)
* filtering by file (any part of the file path)
* logging to stdout/stderr
* logging to a file
* log file rotation (by size)


### In your project

In Cargo.toml:

```
[dependencies]
woodpecker = "0.1"
```

In your `main.rs`:

```
#[macro_use]
extern crate woodpecker;
```

See [examples/tee.rs](examples/tee.rs) for quick overview.

### License
Woodpecker project is licensed under Apache-2.0 license.
Logo is licensed under Creative Commons Attribution (CC BY).

### Credits
Sprockets for logo are provided by Jon Daiello and Ray Uribe from the Noun Project under Creative Commons Attribution (CC BY).