; 
; Undescribed instruction
;
; The RETI instruction (return from interrupt) is not described.
; This should result in the inherent property being disproved.
; 

start:
    reti
    rjmp start
