;
; Set global interrupt flag
;
; This should result in disproving the inherent property
; since enabling interrupts is not described.
;

start:
    sei ; set global interrupt flag
    rjmp start
