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
@SP    //A = SP (i.e. SP_0)
AM=A-1 //A = SP-1, SP--
D=M    //D = *SP
A=A-1  //A = SP-1 (or: SP_0-2)
M=D+M  //D = *SP[-1] + *SP[-2]

// Quote from the book: Three of the commands listed in figure 7.5 (eq, gt, lt) return Boolean values. 
// The VM represents true and false as -1 (minus one, 0xFFFF) and 0 (zero, 0x0000), respectively."