# closure! - A macro for individually capturing variables

This crate provides a macro which lets you write closures that can capture individually either by moving, referencing, mutably referencing of cloning.

## Usage

Start by adding an entry to your `Cargo.toml`:

```toml
[dependencies]
closure = "0.2.0"
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

let closure = closure!(move string, ref x, ref mut y, clone rc || {
    ...
});

```

Unlike standard Rust closures, closures used in the `closure!` macro will alays be move closures, i.e.
unless specified otherwise, any variable will be moved into the closure.

The macro accepts any valid Rust closure in the appropriate position. The only exception is, that no `move`
specifier is allowed before the `| ... |` tokens, because the resulting closure will automatically be a move
closure, anyways.