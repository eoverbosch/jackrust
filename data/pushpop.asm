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

//push (example i=2) local/argument/this/that/temp
// get LCL + i address 
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


//pop (example i=2) local/argument/this/that/temp
// Get LCL + i address
@2
D=A
@LCL
D=D+M
// Store in R13
@R13
M=D
// Load and decrement stack pointer
@SP
AM=M-1
// Get memory value
D=M
// Load LCL+i
@R13
A=M
// Set memory value
M=D


// push pointer 0/1-THIS/THAT: push whatever value is in this/that to stack
@THISTHAT
D=M
//...

