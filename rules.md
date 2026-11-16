# Project Rules

These rules help us to write more uninfied Rust code so that all the code we submit has the same structure.

## No unwrap (unless safe)

```rs
fn foo() -> Result<(), std::Error> {..}

// Don't do this!
fn main() {
  let result = foo().unwrap();
}

// We have a couple of other options
let Ok(_) = foo() else {
  println!("Something went wrong!");
};

// If the above code is not an option, use this:
let result = foo();

if result.is_ok() {
  // Unwrap here is safe
  let result = result.unwrap();
} else {
  println("Something went wrong!");
}
```

The same rules can be applied to `Options`

## Generics

```rs

// Let say we have a trait called Area
trait Area {
  fn area(&self) -> f32
}

// And some structs that implement said trait
struct Circle {
  ..
}

struct Rect {
  ..
}

impl Area for Circle {
  ..
}

impl Area for Rect {
  ..
}

// When defining a function which takes a generic parameter that
// is also bound to some trait, use this syntax
fn print_area<T>
where:
  T: Area
{
  let area = T.area();
  println("The area of the shape is: {}", area);
}
```
## Formatting
Always use `rustfmt`

When downloading rust using `rustup`, this is already included
