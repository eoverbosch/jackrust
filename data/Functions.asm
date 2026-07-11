////////////////////////////////////////////////////
// FUNCTION function_name n_vars
//
//  *Responsibilities: 
//      X Label the start location of the function.
//      X Set LCL
//      X Allocate n_vars
//      X Zero n_vars
//
//  *Required functionality: 
//      - Asm emission
//      - Var zero-ing asm inliner (generates code n times rather than looping)
//      - Function head label generator from function_name and filename
//      - Index parsing
//
//  *Assembly:
//
// Set function head label
(label) // base address

// Get function head pointer and save to LCL i.e. set LCL = function head pointer
@SP   // A = SP_addr
D=M   // D = *SP_Addr == SP
@LCL  // A = LCL_addr
AM=D   // *LCL_addr == LCL = SP, A = LCL == SP

// Allocate n_vars
{"M=0\n\    // M == *SP == stacktop = 0
A=A+1\n\,"} // SP = SP+1 (in-memory pointer, not the actual SP)
// Above block known at compile time, so repeat n_vars times

// bump stack pointer to current A
D=A // D = SP+n_vars
@SP // A = SP_addr
M=D // *SP_addr == SP = SP+n_vars


////////////////////////////////////////////////////
// CALL function_name n_args
//
//  *Responsibilities:
//      X Set function return label right after call site
//      X Mark segment of length n_args as ARG
//      X Push return location to stack
//      X Save previous frame (LCL, ARG, THIS, THAT) i.e. push to stack in order
//      - Set new stack pointer'
//      - Jump PC to function label
//
//  *Required functionality:
//      - Asm emission
//      - Function head label generator from function_name
//      - Return label generator from fun tion_name and counter
//      - Index parsing
//
//  *Assembly (this little manoeuver is gonna cost us 17 years):
//
// Get current stack pointer, set ARG, move stack pointer by n_args
@ARG      // A = ARG_addr
D=M       // D = *ARG_addr == ARG
@R14      // A = R14 == 14
M=D       // *R14 = ARG
@SP       // A = SP_addr
D=M       // D = *SP_addr == SP
@{n_args} // A = int n_vars
D=D-A     // D == SP = SP - n_vars
@ARG      // A = ARG_addr
M=D       // *ARG_addr == ARG = D == SP - n_vars 

// Push to stack in order: RETLABEL, LCL, ARG, THIS, THAT
//    RETLABEL
@label$ret 
D=A   // D = label
@SP   // A = SP_addr
M=M+1 // SP = SP0+1 == SP1
A=M-1 // SP = (SP0+1)-1 == SP
M=D   // *SP == stacktop = D == label

//    LCL
@LCL  // A = LCL_addr
D=M   // D = *LCL_addr == LCL
@SP   // A = SP_addr
M=M+1 // SP = SP1+1
A=M-1 // SP = (SP1+1)-1 == SP
M=D   // *SP == stacktop = D == LCL

//    ARG
@R14  // A = R14
D=M   // D = *R14 == ARG
@SP   // A = SP_addr
M=M+1 // SP = SP1+1
A=M-1 // SP = (SP1+1)-1 == SP
M=D   // *SP == stacktop = D == ARG

//    THIS 
@THIS // A = THIS_addr
D=M   // D = *THIS_addr == THIS
@SP   // A = SP_addr
M=M+1 // SP = SP1+1
A=M-1 // SP = (SP1+1)-1 == SP
M=D   // *SP == stacktop = D == THIS

//    THAT 
@THAT // A = THAT_addr
D=M   // D = *THIS_addr == THAT
@SP   // A = SP_addr
M=M+1 // SP = SP1+1
A=M-1 // SP = (SP1+1)-1 == SP
M=D   // *SP == stacktop = D == THAT

// Jump to function body
@label
0;JMP

// Add return label after call site
(label$ret) // = file_name + "." + function_label + "$ret." + &counter.to_string()


////////////////////////////////////////////////////
// RETURN 
//
//  *Responsibilities:
//      X Store stack top in first of previous ARG
//      X Write caller block back into memory
//      X Clear block
//      X Get return address from caller block and jump. No labels necessary (thank fuck)
//
//  *Required functionality:
//      - Asm emission
//
//  *Assembly:

// Save temp pointer at return address and save value
@LCL    // A = LCL_addr
D=M     // D = *LCL_addr == LCL
@5      // A = 5
D=D-A   // D = LCL - 5
@R14    // A = R14_addr
AM=D    // R14 = LCL-5, A = LCL-5
D=M     // D = *(LCL-5) == ret_label
@R13    // A = R13_addrf
M=D     // R13 = ret_label

// Set LCL-5-1 = stacktop == result
@SP       // A = SP_addr
A=M-1     // A = *SP_Addr - 1 == SP-1
D=M       // D = *(SP-1) == last_stack_value 
@ARG      // A = ARG_addr
A=M       // A = ARG
M=D       // *ARG = last_stack_value

// Set stack pointer to LCL-5 (ARG_end + 1) and save pointer increment to R14
D=A+1   // D = (LCL-6) + 1 
@SP     // A = SP_Addr
AM=D    // *SP_addr == SP = LCL-5, A = LCL-5
D=A+1   // A = (LCL-5)+1
@R14    // A = R14_addr
AM=D     // R14 = LCL-4, A=LCL-4

// Restore previous stack frame
//   LCL
D=M    // D = *(LCL-4) == LCLs
@LCL   // A = LCL_addr
M=D    // *LCL_addr == LCLr = LCLs

@R14   // A = R14_addr
AM=M+1 // A = R14 + 1 == (LCL-4)+1, R14 = (LCL-4)+1

//   ARG
D=M    // D = *(LCL-3) == ARGs
@ARG   // A = ARG_addr
M=D    // *ARG_addr == ARGr = ARGs

@R14   // A = R14_addr
AM=M+1 // A = R14 + 1 == (LCL-3)+1, R14 = (LCL-3)+1

//   THIS
D=M    // D = *(LCL-2) == THISs
@THIS  // A = THIS_addr
M=D    // *THIS_addr == THISr = THISs

@R14   // A = R14_addr
AM=M+1 // A = R14 + 1 == (LCL-2)+1, R14 = (LCL-2)+1

//   THAT
D=M    // D = *(LCL-1) == THATs
@THAT  // A = THAT_addr
M=D    // *THAT_addr == THATr = THATs

// Get return address and jump 
@R13   // A = R13_addr
A=M    // A = R13 
0;JMP


/////// OLD ///////

// Save return label at ARG+n_args
@ARG    // A = ARG_addr
D=M     // D = *ARG_addr == ARG
@n_args // A = n_args
A=D+A   // A = ARG + n_args == ret_label_saved_addr
D=M     // D = *(ARG+n_args) == ret_label
@R13    // A = R13_addr
M=D     // R13 = ret_label

// Set ARG[0] = stacktop == result
@SP       // A = SP_addr
A=M-1     // A = *SP_Addr - 1 == SP-1
D=M       // D = *(SP-1) == last_stack_value 
@ARG      // A = ARG_addr
A=M       // A = *ARG_addr == ARG
M=D       // *ARG = D == last_stack_value

// Set stack pointer to ARG+1
D=A+1 // D = ARG + 1 
@SP   // A = SP_Addr
M=D   // *SP_addr == SP = ARG+1

// Store temp pointer starting at ARG+1+n_args
@n_args // A = n_args
D=D+A   // D = ARG+1+n_args == LCL_saved_addr. A is now a virtual stack pointer to saved LCL.
@R14    // A = R14_addr
AM=D    // R14 = temp_SP, A = temp_SP

// /LCL 
D=M     // D = *temp_SP = LCL_stored
@LCL    // A = LCL_addr
M=D     // LCL = LCL_stored

@R14
AM=M+1  // A = temp_SP + 1, temp_SP = temp_SP + 1

// /ARG
D=M     // D = *temp_SP = ARG_stored
@ARG    // A = ARG_addr
M=D     // ARG = ARG_stored

@R14
AM=M+1  // A = temp_SP + 1, temp_SP = temp_SP + 1

// /THIS
D=M     // D = *temp_SP = THIS_stored
@THIS   // A = THIS_addr
M=D     // ARG = THIS_stored

@R14
AM=M+1  // A = temp_SP + 1, temp_SP = temp_SP + 1

// /THAT
D=M     // D = *temp_SP = THAT_stored
@THAT   // A = THAT_addr
M=D     // ARG = THAT_stored

// Jump PC to stored address
@R13    // A = R13_addr
A=M     // A = *R13_addr = R13 == ret_label
0;JMP   // Jump to A
