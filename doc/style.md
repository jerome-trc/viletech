# Code Style Guidelines

## C

- Defer to the top-level .clang-format file.

- Annotate *all* functions with `/// @fn function_name` to make it easier to search for functions when the return type is not known.
- Do not declare fields as `const`.
- Enumeration variant names are in `UPPER_SNAKE_CASE`.
- Error return codes are 0 for "O.K", positive for user error, and negative for internal error.
- Function names are in `lower_snake_case`.
- Global variables have names prefixed with `g_`.
- If a function's only utility is its return value, annotate it as `[[nodiscard]]`.
- Local variable names are in `lower_snake_case`.
- Parameter names are in `lower_snake_case`.
- Preprocessor definition names are in `UPPER_SNAKE_CASE`.
- Static variables have names prefixed with `s_`.
- Structure and union field names are in `lower_snake_case`.
- Type names are in `PascalCase` (in acronyms, only capitalize the first letter).

## C++

- All C guidelines apply.

- Always dereference `this` explicitly.
- Any structure and class which does not explicitly require copy constructors/operators shall have them deleted by default.
- Avoid inheritance wherever possible. Prefer composition, static polymorphism, unions, or `std::variant`.
- Constexpr value names are in `UPPER_SNAKE_CASE`.
- Constructors must be qualified as `explicit`.
- Do not overload functions.
- Do not overload operators, except to provide arithmetic operators for numeric types (e.g. vectors).
- Do not use default arguments.
- Do not use the `class` keyword.
- Do not use the `friend` keyword.
- Do not use the `mutable` keyword.
- Do not use variable-size integer types (e.g. `int` or `char`).
- Mark any types to which the `final` keyword applies as such if it is meant to be a parent type.
- Namespace names are in `lower_snake_case`.
- Prefer aggregate initialization over the use of constructors.
- Prefer `using` over `typedef`.
- Scoped enumeration variant names are in `PascalCase` (in acronyms, only capitalize the first letter).
- Template parameter names follow the same guidelines as C type names.

## CMake

- Defer to the top-level .cmake-format file.

- Variable names are in `UPPER_SNAKE_CASE`.
- Function and macro names are in `lower_snake_case`.
