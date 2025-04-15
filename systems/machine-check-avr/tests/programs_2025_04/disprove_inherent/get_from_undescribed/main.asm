;
; Get from nonaliased
;
; Reading from a nonaliased memory location should fail.
; Nonaliased memory locations must not be manipulated
; because their behaviour is not described.
;

start:
	in r16, OCR0A ; timer/counter 0 output compare register A is not described
    rjmp start
