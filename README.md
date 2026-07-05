# Dahlia 🌺
A tiny low-level, memory-safe programming language. Written in Rust, compiles to C.

## Data Types
```
// i32, i64, u8, u16, u32, u64, f32, f64
// bool, char, str
var hitchhikers: i32 = 42
var pi: f64 = 3.14159
var name: str = "charlotte"

// const too
const PI_2: f64 = 3.14159 * 2
const MAX_SIZE: i32 = 1024
```

## Memory
Hibiscus has first-class support for arena allocators.

```
const MIB: i32 = 1024 * 1024
arena ATest(1 * MIB)

fn alloc_test() void
  var a: *u8 = ATest.alloc(4)
  var b: *u8 = ATest.alloc(4)

  // destroy all allocations made by this arena
  ATest.destroy()
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
- Memory management (arena checking)
- Module system