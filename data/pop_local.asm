//pop local i:

//addr = LCL + i
//SP--
//addr* = SP*

// Store i in D
@2
D=A

// Add LCL to i
@LCL
D = D+A

// Reduce value of SP by one
@SP
AM = M-1

