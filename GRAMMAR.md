## Dahlia Grammar 🌺

EBNF syntax inspired by [Pythons EBNF syntax](https://docs.python.org/3/reference/grammar.html).

## EBNF
```coffee

# Notes
# ----------------------
# * Digit is defined as a single character in the range 0-9
# * Letter is defined as a single character in the ranges a-z and A-Z
# 
# 
# EBNF syntax primer:
# -------------------
#   rule_name: expression - Define a rule
#   e1 | e2               - Alternation (e1 or e2)
#   e1 e2                 - Concatenation (match e1 followed by e2)
#   [ e ]                 - Optionally match e
#   e*                    - Match zero or more occurrences of e
#   e+                    - Match one or more occurrences of e
# ----------------------

# Starting Rules
# --------------
program: toplevel_stmt*
toplevel_stmt:
    | fn_declaration
    | struct_declaration
    | stmt


# Building Blocks
# ---------------
alphanumeric: letter | digit
literal:
    | numeric_literal
    | string_literal
    | boolean_literal
    | array_literal

int_literal: digit*
float_literal: digit* "." digit*

numeric_literal: int_literal | float_literal
string_literal: '"' .* '"'
boolean_literal: "true" | "false"
array_literal: "[" [ expr ("," expr)* ] "]"

binary_op:
    | "+"
    | "-"
    | "*"
    | "/"
    | "%"
    | "=="
    | "!="
    | "<"
    | "<="
    | ">"
    | ">="

unary_op:
    | "-"
    | "!"
    | "&"  # address-of operator
    | "*"  # dereference operator


# Types
# ------------
type:
    | signed_int_type
    | unsigned_int_type
    | float_type
    | bool_type
    | str_type
    | pointer_type
    | array_type
    | user_defined_type

signed_int_type: "i8" | "i16" | "i32" | "i64"
unsigned_int_type: "u8" | "u16" | "u32" | "u64"
float_type: "f32" | "f64"
bool_type: "bool"
str_type: "str"
pointer_type: "*" type
array_type: type "[" [number] "]"
user_defined_type: identifier

# Identifiers
# ------------

# Identifiers can start with _ and contain letters, digits, and underscores
identifier_start: letter | "_"
identifier: identifier_start (alphanumeric | "_")*
typed_identifier: identifier ":" type

dotted_name:
    | identifier ("." identifier)*

name: dotted_name

# Expressions
# ------------
index_expr: 
    | expr "." "[" expr "]"

pointer_dereference: 
    | "*" expr

pointer_address_of: 
    | "&" expr

function_call: 
    | name "(" [ expr ("," expr)* ] ")"

if_expr: 
    | "if" expr "then" expr ["else" expr] "end"

expr:
    | literal
    | name
    | expr binary_op expr
    | unary_op expr
    | index_expr
    | pointer_dereference
    | pointer_address_of
    | function_call
    | if_expr

# Statements
# -----------
var_declaration: 
    | "var" typed_identifier "=" expr

const_declaration: 
    | "const" typed_identifier "=" expr

var_assign:
    | name "=" expr

fn_declaration: 
    | "fn" identifier "(" [ typed_identifier ("," typed_identifier)* ] ")" [":" type] "{" stmt* "}"

struct_declaration: 
    | "struct" identifier typed_identifier* "end"

enum_declaration: 
    | "enum" identifier typed_identifier* "end"

for_statement: 
    | "for" identifier ":" expr "do" stmt* "end"

while_statement: 
    | "while" expr "do" stmt* "end"

return_statement: 
    | "return" expr

break_statement: 
    | "break"

stmt:
    | expr
    | var_declaration
    | const_declaration
    | var_assign
    | for_statement
    | while_statement
    | return_statement
    | break_statement
```