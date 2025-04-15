;
; gate array
;
; Periodically updates output pins to mirror these operations:
; PB1 <= PB0
; PB2 <= not PD7
; PB3 <= PD5 and PD6
; PB4 <= PB6 or PB7
; PC0 <= PC1 xor PC2
;
; The pins are selected to be near each other or have gate input 
; on the opposite side of the chip from the output
; in the physical ATmega328P DIP pinout.
; This would probably be done by a reasonable board designer.
;
; They are also selected to test all GPIO ports (B, C, D)
; of ATmega328P chips and be reasonably different to explore
; different behaviours.
;
; The code is also written to use different constructs for each gate
; that should result in the fulfillment of the specification.
;

start:
; set PB1, PB2, PB3, PB4 as output
	ldi r16, 0x1E
	ldi r20, 1
	out DDRB, r16

; set PC0 as output
	sbi DDRC, 0

; loop forever
loop:
	eor r17, r17 ; load current PINB values to r17 for PORTB computation

	; --- BUFFER ---
	
	sbic PINB, 0 ; skip if PB0 input is zero
	ori r17, 0x02 ; set PB1 to 1 -> this will make it the same value as PB0

	; --- INVERTER ---

	sbis PIND, 7 ; skip if PD7 input is one
	ori r17, 0x04 ; set PB2 to 1 -> this will make it a negation of PD7
	

	; --- AND ---
	
	sbic PIND, 5 ; skip if PD5 is zero
	jmp half_and
	
after_half_and:

	; --- OR ---
	ori r17, 0x10
	sbis PINB, 6 ; skip if PB6 is one
	jmp half_or
	
after_do_or:

	out PORTB, r17 ; write computed PB output values to PORTB
	
	; --- XOR ---
	
	ldi r19, 0
	sbic PINC, 1 ; skip if PC1 input is zero
	eor r19, r20
	sbic PINC, 2 ; skip if PC2 input is zero
	eor r19, r20

	out PORTC, r19 ; set as PC output values

    rjmp loop

half_and: ; PD5 is 1
	sbic PIND, 6 ; skip if PD6 is zero
	ori r17, 0x08 ; set PB3 to 1 -> this will make it (PD5 and PD6)
	jmp after_half_and ; jump back

half_or: ; PB4 should be set to 1
	sbis PINB, 7 ; skip if PB7 is one
	andi r17, 0xEF ; clear the appropriate bit
	jmp after_do_or



