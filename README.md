# Dahlia 🌺
A tiny low-level, memory-safe programming language. Written in Rust, compiles to C.

## Data Types
```
// i32, i64, u8, u16, u32, u64, f32, f64, bool, char, str
var hitchhikers: i32 = 42
var pi: f64 = 3.14159
var name: str = "charlotte"

// const too!
const PI_2: f64 = 3.14159 * 2
const MAX_SIZE: i32 = 1024
```

## Strings
Dahlia has both the primitive `str` type (unboxed), and the boxed `Str` type. 

The `str` type is a pointer to a string in memory, while the `Str` type is a struct that contains a pointer to the string and its length.

```
var s: str = "hello" // s is a pointer to the string "hello" in memory

// boxed strings must be allocated and freed, either manually or with an arena allocator

const GPA: allocator GPAllocator(1 * MIB) // 1 MiB arena allocator

// allocate a new Str in the arena
fn alloc_str_test() void with GPA {
  var s: Str = Str.new_in("hello", &allocator)
  defer GPA.delete(s)
}
```

`Str` must be managed, so `str` is preferred. You can convert from `Str` to str with `Str.to_str()`, but you lose
the ability to modify the string in place. 

## Allocation
Dahlia has first-class support for arena allocators. There is no garbage collection.

```
const MIB: i32 = 1024 * 1024

const GPA: allocator GPAllocator(1 * MIB) // 1 MiB arena allocator

// Classical style allocation and deallocation
fn alloc_test() void with GPA {
  var a: *u8 = new u8[4] // allocate 4 bytes in the arena
  defer GPA.delete(a)
  
  var b: *u8 = new u8[8] // allocate 8 bytes in the arena
  defer GPA.delete(b)
}

```

Additionally, it also features pointers
```
var a_string: str = "hello"
var a_string_ptr: *str = &a_string
a_string_ptr[0] = 'H' // a_string is now "Hello"

// const pointers cannot be dereferenced to modify the value they point to
const a_string_ptr2: const *str = &a_string
```

## Type System
The type system is based on Hindley Milner.

There are no objects, but there are structs and type classes (Rust/Haskell traits). Dahlia is a statically typed language, and all types must be known at compile time. `[]` is used to denote generic type parameters and arrays are indexed using `.[idx]` as a consequence.

```
// Type alias does not create a new type, it just gives a new name to an existing type
type MyInt i32

// Type class defined using 'typeclass' and 'has' keywords
typeclass Eq[A] has {
  required fn eq(a: A, b: A) bool
  required fn not_eq(a: A, b: A) bool
}

impl Eq for i32 {
  fn eq(a: i32, b: i32) bool {
    return a == b
  }

  fn not_eq(a: i32, b: i32) bool {
    return a != b
  }
}

fn is_equal[A: Eq](a: A, b: A) bool {
  return a.eq(b)
}
```


## Plans
- Core language features
- Generics
- Memory management (arena checking, somehow)
- Module system
- C interop