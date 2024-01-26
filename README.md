# single-header
a rust command line utility to generate portable C/C++ single header file


### Overview

This Rust program is designed to convert C/C++ files into portable single-header files.

- It processes C/C++ files by running the C preprocessor on it
- Call the C preprocessor with the `-fdirectives-only` option to limit system specific macros / includes
- Undoes the `#include` expansion of all system headers
- does so by relying on [gcc preprocessor output documentation](https://gcc.gnu.org/onlinedocs/cpp/Preprocessor-Output.html) as the expected proprocessor output
- replaces them with `#include` directives that are as close to the original as possible.
- Offers protection against multiple inclusions with either `#ifdef` or `#pragma once`.

Limitations:
- all preprocessor conditions (`#if`/`#else`/`#endif`) that occurs outside system headers will be evaluated.
  Only way to prevent this would be to implement a custom mock C-preprocessor.

### Example

for the following project
```c++
// test.hpp
#pragma once
#include "first.hpp"
void test() {}

// first.hpp
#pragma once
#include "second.hpp"
#include <cstddef>
void second_function() {}

// second.hpp
#pragma once
#include <type_traits>
void first_function() {}
```
will produce:
```bash
$> single-header test.hpp
#ifndef TEST_HPP_SINGLE_HEADER
# define TEST_HPP_SINGLE_HEADER
#include <type_traits>
void first_function() {}
#include <cstddef>
void second_function() {}
void test() {}
#endif // TEST_HPP_SINGLE_HEADER
```

### Installation

Using cargo via crates.io:
```bash
cargo install single-header
```

Manually:

```bash
git clone git@github.com:DaemonSnake/single-header.git
cd single-header
cargo install --path .
```

### Usage

```bash
Usage: single-header [OPTIONS] <FILE> [-- <CPP_OPTS>...]

Arguments:
  <FILE>
          path to c/c++ header file

  [CPP_OPTS]...
          additional parameters for the preprocessor

Options:
  -p, --preprocessor <PREPROCESSOR>
          [default: cpp]
          [possible values: cpp, gcc, clang]

  -i, --inline <INLINE_PATH>
          path / file that must allways be `#include` expanded (can provided multiple times)

  -x, --lang <LANG>
          [default: c++]
          [possible values: c, c++]

      --protect <PROTECTION>
          protect against multiple includes with `#ifdef` or `#pragma once`
          [default: ifdef]
          [possible values: ifdef, once]

  -h, --help
          Print help (see a summary with '-h')
```

### Requirements
- Rust
- C Preprocessor `cpp`

