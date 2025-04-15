;
; Set to undescribed
;
; Writing to an undescribed memory location should result
; in the inherent property being disproved.
;

; Replace with your application code
start:
	out OCR0A, r16 ; timer/counter 0 output compare register A is not described
    rjmp start
