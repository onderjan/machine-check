;
; Set to restricted
;
; On AVR, there are registers where only a few bits are restricted.
; These bits should not be written to 1 for compatibility with future devices.
; This is used for port B which lacks the highest bit.
; Setting it should disprove the inherent property as implemented.
;

start:
	ldi r16, 0x80
    out PORTC, r16 ; set highest bit in PORTC
    rjmp start
