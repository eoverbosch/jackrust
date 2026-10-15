# jackrust

My Rust toolchain for the Hack computer from the course [*The Elements of Computing Systems* (nand2tetris)](https://www.nand2tetris.org/): an assembler, a VM translator, and the HDL chips the computer is built from.

Nand2tetris builds a complete computer from first principles: logic gates, then an ALU, memory and a CPU, then the software stack on top: an assembler, a stack-based virtual machine, a compiler for a small object-oriented language called Jack, and an operating system. The course provides the specifications and test programs; the implementations here are my own.

## Status

Includes all hardware implementations of the course in the .hdl files in hardware/, according to the format that the online course IDE requires.
Includes a complete CLI implementation of the hackasm assember.
Includes a complete CLI implementation of the jackVM virtual machine translator.

## Getting started

Requires Rust 1.85 or newer (the crate uses the 2024 edition).

Running `cargo build` produces two binaries: `hackasm` and `jackVM`.

## Usage

### `hackasm`: Assembler

Translates an .asm file to a .hack file, which contain binary instructions for the hack platform.

```sh
hackasm Prog.asm              # writes Prog.hack
hackasm Prog.asm -o Out       # writes Out.hack
hackasm Prog.asm --dump       # also writes Prog.dump (preprocessed code and symbol table)
```

### `jackVM`: VM translator

Translates a .vm file, containing a set of virtual stack machine instructions, to hack assembly code in a .asm file. It should be run from inside a directory of `.vm` files. The directory must contain `Sys.vm`, which defines the entry point `Sys.init`. 

```sh
cd MyProgram/
jackVM                        # writes MyProgram.asm
jackVM -o Out                 # writes Out.asm
jackVM --dump                 # also writes MyProgram.vm: the merged, comment-stripped VM code
```

The generated assembly starts with the standard bootstrap code (`SP = 256`, then `call Sys.init`).

# 

## Design

### Assembler

The assembler works with two passes:

1. **First pass:** strip comments and whitespace, record each label `(LOOP)` with the address of the instruction that follows it, and drop the label lines.
2. **Second pass:** translate each instruction. A-instructions (`@value`) resolve symbols through the symbol table, allocating new variables from RAM[16] upwards. C-instructions (`dest=comp;jump`) are split into their three fields, and each field is looked up in its own table and combined into the 16-bit word.

### VM translator

The translator processes a program in three stages:

1. **File discovery.** Collects the `.vm` files in the current directory. Files are only read when needed.
2. **Call graph.** Starting from `Sys.init`, scans each function body for `call` statements and build a directed call graph (using the petgraph crate). A called
   function's file is found from its name (`Math.double` is in `Math.vm`), so only files that are actually reached get read.
3. **Code generation.** Walk the graph depth-first from `Sys.init`, and translate only the functions it reaches. 

Because only reachable functions are translated, unused code (such as unused OS library functions) never reaches the output. Hack programs have to fit in 32K words of ROM, so this keeps the output smaller.

Each VM command is parsed into a typed operation (`Arithmetic`, `Compare`, `StackOperation`, `BranchOperation`, `FunctionOperation`) with an `emit_asm` method. The compiler's exhaustive `match` checking then guarantees every command and segment combination is handled.

Labels are generated as per the standard format described in the course slides. 

## References

Course and specifications: https://www.nand2tetris.org/


