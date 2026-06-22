// Generalised two-parameter operations:
@SP
AM=M-1
D=M
A=A-1 // up until here, provides access to both x and y as x=M and y=D
// M= operation

// Generalised one-parameter operations:
@SP
A=M-1 // Requires no stack pointer shifting
// M= -M or !M

//Example:
//add
@SP    // A = SP (i.e. SP_0)
AM=A-1 // A = SP-1, *SP = SP-1
D=M    // D = *SP // 
A=A-1  // A = SP-1 (or: SP_0-2)
M=D+M  // M = *SP[-1] + *SP[-2] // actual add operation


// Quote from the book: Three of the commands listed in figure 7.5 (eq, gt, lt) return Boolean values. 
// The VM represents true and false as -1 (minus one, 0xFFFF) and 0 (zero, 0x0000), respectively."

Order of comparison:
Stack:
...
x
y
Operation:
x COMP y

@SP    // A = SPaddr (i.e. SP_0) |
AM=M-1 // A = SP-1, SP = SP-1    x |
D=M    // D = *SP (= y)            |
A=A-1  // A = SP-1                 x (now M(A) = (SP-1)* = x)
// (Now you compare and jump accordingly)\
D = M - D // x-y now stored in D 
// D needs to be static for at least one check (maybe both). The moment we load a label, 
// M is no longer available. 
// A: Set AREG jumptrue (C), check true and jumpif C
// B: Set AREG jumpfalse (D), set DREG false and jump D
// C: Set DREG true and jump D (jump D is default behaviour)
// D: Get stack pointer and replace x with true/false. We should check if we've done stack pointer correctly and if we can save a shift.


@SP
AM=M-1
D=M
A=A-1
D=M-D
@truelabel
D; JLT/JEQ/JGT 
@falselabel
D=0
0; JMP
(truelabel)
D=-1
(falselabel)
@SP
A=M-1
M=D

New idea:

                                 A-lifetime
@SP    // A = SPaddr             | (address of SP, typically 0)
AM=M-1 // A = SP-1, SP = SP-1    x | (deref address of SP to get the actual stack pointer - 1)
D=M    // D = *SP (= y)            |
A=A-1  // A = SP-1                 | (Same thing, just decremented again)
D = M - D // x-y now stored in D   | 
M = -1                             | (replace M(SP-2) = x with -1 (true))
@truelabel                       | x (A = label) Garbage. 
D; JLT/JEQ/JGT                   |
@SP                              x | (address of SP, typically 0)
A = M-1                            | (deref address of SP to get the actual stack pointer - 1, again)
M = 0                              | set false
(truelabel)                        |
...                               ...
# yay
@SP
AM=M-1
D=M
A=A-1
D=M-D
M=-1
@truelabel
D; JLT/JEQ/JGT
@SP
A=M-1
M=0
(truelabel)