// FUNCTION function_name n_vars
//
//  *Responsibilities: 
//      -Label the start location of the function.
//      -Read n_vars.
//      -set LCL and allocate n_vars
//
//  *Required functionality: 
//      -Asm emission
//      -Function head label generator from function_name
//      -Index parsing
//
//  *Assembly:
@ASM


// CALL function_name n_args
//
//  *Responsibilities:
//      -Set function return label at the call site
//      -read n_args
//      -mark segment of length n_args as ARG
//      -Push return location to stack
//      -Save previous frame (LCL, ARG, THIS, THAT)
//      -Set new stack pointer'
//      -Jump PC to function label
//
//  *Required functionality:
//      -Asm emission
//      -Function head label generator from function_name
//      -Return label generator from function_name and counter
//      -Index parsing
//
//  *Assembly:
@ASM


// RETURN 
//
//  *Responsibilities:
//      -Store stack top in first of previous ARG
//      -Write caller block back into memory
//      -Clear block
//      -Get return address from caller block and jump. No labels necessary (thank fuck)
//
//  *Required functionality:
//      -Asm emission
//
//  *Assembly:
@ASM

