# Calc

Over Enggineered Calc (Slang for Calculator)

Can be run on an interpreter, bytecode VM, and LLVM JIT compiler

built with the help of [the greats](https://createlang.rs)

## Features

- It can calc [+, -, *, /, ()]

## Setup

### Prerequisites

- rust 2021+
- for JIT features: LLVM 20.1 and rust nightly

### Installation

```bash
git clone https://github.com/Rational-Idiot/calc
cd calc
```

## Usage

### REPL (Interactive Mode)

```bash
# Interpreter
cargo run --bin repl

# Bytecode VM
cargo run --bin repl --no-default-features --features vm

# JIT
cargo run --bin repl --no-default-features --features jit
```

### Execute from File

```bash
echo "<Enter an Expression>" > test.calc

#interpreter
cargo run --bin main -- test.calc

#VM
cargo run --bin main --no-default-features --features vm -- test.calc

#JIT
cargo run --bin main --no-default-features --features jit -- test.calc
```

## Grammar

```pest

Program = _{ SOI ~ Expr ~ EOF }

Expr = { Term ~ ((Add | Subtract) ~ Term)* }
Term = { Factor ~ ((Multiply | Divide) ~ Factor)* }
Factor = { UnaryExpr | Primary }
Primary = { Float | Int | "(" ~ Expr ~ ")" }

UnaryExpr = { UnaryOp ~ Factor }
UnaryOp = @{ "+" | "-" }

Add      = { "+" }
Subtract = { "-" }
Multiply = { "*" }
Divide   = { "/" }

Int   = @{ ASCII_DIGIT+ }
Float = @{ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT+ }

WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
EOF = _{ EOI | ";" }
```
