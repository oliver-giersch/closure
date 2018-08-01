# closure! - A macro for individually capturing variables

This crate provides a macro which lets you write closures that can capture individually either by moving, referencing, mutably referencing of cloning.

## Usage

Start by adding an entry to your `Cargo.toml`:

```toml
[dependencies]
closure = "*"
```

Then import the crate with macro use enabled:
```rust
#[macro_use]
extern crate closure;
```

Then you can write closures like so:
```rust
let string = String::from("move");
let x = 10;
let mut y = 20;
let rc = Rc::new(5);

let closure = closure!(move string, ref x, ref mut y, clone rc {
    ...
});

```