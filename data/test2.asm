// Computes R0 = 2 + 3  (R0 refers to RAM[0])

	(Start)
@2
D = A

@hello
D = D + A

@0
M = D
	(END)
@Start // Loop around
0;JMP