;
; Independent nondeterminism
; Branching is performed as many times as necessary to get as many states as needed.
;
; 6 branching points are used in the thesis supplementary material version so it
; can be checked relatively fast for demonstration purposes.
;

start:
	eor r16, r16
	eor r17, r17
loop:
	sbis PINB, 0
	ori r16, 0x01
	
	sbis PINB, 0
	ori r16, 0x02
	
	sbis PINB, 0
	ori r16, 0x04
	
	sbis PINB, 0
	ori r16, 0x08
	
	sbis PINB, 0
	ori r16, 0x10
	
	sbis PINB, 0
	ori r16, 0x20
	
	;sbis PINB, 0
	;ori r16, 0x40
	
	;sbis PINB, 0
	;ori r16, 0x80
	
	;sbis PINB, 0
	;ori r17, 0x01
	
	;sbis PINB, 0
	;ori r17, 0x02
	
	;sbis PINB, 0
	;ori r17, 0x04
	
	;sbis PINB, 0
	;ori r17, 0x08
	
	;sbis PINB, 0
	;ori r17, 0x10
	
	;sbis PINB, 0
	;ori r17, 0x20
	
	;sbis PINB, 0
	;ori r17, 0x40
	
	;sbis PINB, 0
	;ori r17, 0x80

    rjmp loop
