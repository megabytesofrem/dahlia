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
arena heap allocator(1 * MIB)

// allocate a new Str in the arena
fn alloc_str_test() void
  var s: Str = Str.new_in("hello", &allocator)
  // do something with s
  // ...
  // free the Str when done
  allocator.destroy()
end
```

`Str` must be managed, so `str` is preferred. You can convert from `Str` to str with `Str.to_str()`, but you lose
the ability to modify the string in place. 

## Allocation
Dahlia has first-class support for arena allocators. There is no garbage collection.

```
const MIB: i32 = 1024 * 1024
arena heap allocator(1 * MIB)

fn alloc_test() void
  var a: *u8 = allocator.alloc(4)
  var b: *u8 = allocator.alloc(4)

  // destroy all allocations made by this arena at the end of the function
  allocator.destroy()
end
```

Additionally, it also features pointers
```
var a_string: str = "hello"
var a_string_ptr: *str = &a_string
a_string_ptr[0] = 'H' // a_string is now "Hello"

// const pointers cannot be dereferenced to modify the value they point to
const a_string_ptr2: const *str = &a_string
```

## Plans
- Core language features
- Generics
- Memory management (arena checking, somehow)
- Module system
- C interop