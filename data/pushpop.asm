//push (example i=2) constant
// get constant
@2
D=A
// Load and increment stack pointer
@SP
M=M+1
A=M-1
// Set stack value 
M=D


//MAP: local-LCL/argument-ARG/this-THIS/that-THAT/temp-5/static-xxx.i

//push (example: push local 2) local/argument/this/that/temp
// get *LCL + i address 
@2
D=A
@LCL
A=D+M
// load M into D 
D=M
// Load and increment stack pointer
@SP
M=M+1
A=M-1
// Set stack value 
M=D

// for temp and static: You don't dereference the address. The address is either 5+i or filename.i
@{5+i}
D=M
@SP
M=M+1
A=M-1
M=D

//pop (example: pop local 2) local/argument/this/that/temp
// Step 1: Get LCL + i address
@2 // i
D=A
@LCL // segment
D=D+M
// Step 2: Store in R13
@R13
M=D
// Step 3: Load and decrement stack pointer
@SP
AM=M-1
// Step 4: Get memory value
D=M
// Step 5: Load LCL+i
@R13
A=M
// Step 6: Set memory value
M=D

// for temp and static: You don't dereference the address. The address is either 5+i or filename.i
@{5+i}
D=A
@R13
M=D
@SP
AM=M-1
D=M
@R13
A=M
M=D


// push/pop pointer 0/1-THIS/THAT: push whatever value is in this/that to stack
@THISTHAT
D=M
//...

