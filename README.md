# Dahlia 🌺
A tiny low-level, memory-safe programming language. Written in Rust, compiles to C.

## Data Types
```rs
// i32, i64, u8, u16, u32, u64, f32, f64, bool, char, str
var hitchhikers: i32 = 42
var pi: f64 = 3.14159
var name: str = "charlotte"

// const too!
const PI_2: f64 = 3.14159 * 2
const MAX_SIZE: i32 = 1024
```

## Memory Model
Dahlia relies on a topological model (the entire program is represented as a simplified Grothendieck topos). Each block has a named region (the sheaf), and `local('region)` is used to allocate memory in that region.`

`'a ~> 'b` is the promotion operator and it is used to promote a value from one region to another. Functions can be generic over regions using spatial polymorphism, and they can specify the direction of the morphism between related sheaves.

```rs
fn alloc_test() void 'alloc_test {
  var final_str: Str 'alloc_test = Str.new('alloc_test, "")

  'inner {
    var inner_str: Str 'inner = Str.new('inner, "hello")
    final_str = inner_str 'inner ~> 'alloc_test 
  }

  return final_str 'alloc_test
}

// A simple identity function that is generic over a region 'r0.
fn identity['r0: sheaf] ('r0, value: i32) i32 'r0 {
  // return the value in the same region it was passed in
  return value 'r0
}

// This function is generic over regions 'r0 and 'r1.

// - 'q ~~> 'r is a forwards morphism
// - 'q <~~ 'r is a backwards morphism
// - 'q <~> 'r is a bidirectional morphism
fn dependency['r0: sheaf, 'r1: sheaf, A] (morph 'r0 ~~> 'r1, value: A) A 'r1 {
  // promote a value between sheaves 'r0 and 'r1
  return promote('r0, 'r1, value) 'r1
}
```


### Pointers

```rs
var a_string: str = "hello"
var a_string_ptr: *str = &a_string
a_string_ptr[0] = 'H' // a_string is now "Hello"

// const pointers cannot be dereferenced to modify the value they point to
const a_string_ptr2: const *str = &a_string
```

## Type System
The type system is based on Hindley Milner.

There are no objects, but there are structs and type classes (Rust/Haskell traits). Dahlia is a statically typed language, and all types must be known at compile time. `[]` is used to denote generic type parameters and arrays are indexed using `.[idx]` as a consequence.

## Plans
- Core language features
- Generics
- Memory management (arena checking, somehow)
- Module system
- C interop