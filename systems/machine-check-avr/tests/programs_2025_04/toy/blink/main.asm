;
; Toggles output pin PINB.0 periodically.
;

.equ clock_freq = 1000000 ; we assume 1 MHz clock

; .equ toggle_period_ms = 1
; .equ delay_cycles = (toggle_period_ms*250)-2

; Only 0.5 ms period is used in the thesis supplementary material version so it
; can be checked relatively fast for demonstration purposes.

.equ toggle_period_ms_tenths = 5
.equ delay_cycles = (toggle_period_ms_tenths*25)-2


start:
	nop ;pad
	nop ;pad

	sbi DDRB, 0 ; set pin as output
	eor r16, r16
	ldi r17, 0x1

loop:
	ldi r25, HIGH(delay_cycles) ; load 16-bit counter
	ldi r24, LOW(delay_cycles)
delay:
	sbiw r24, 1 ; subtract 1
	brne delay ; loop till it is zero

	sbi PINB, 0
	nop
	nop
	nop

    rjmp loop
