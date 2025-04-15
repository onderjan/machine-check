;
; Momentary selection switch
; 
; If momentary selection is set to 0, action switch value should be copied to PB2
; if momentary selection is set to 1, action switch rising edge should toggle PB2 

; This is an example testing rule failure:
; 10 ms debouncing is used on action switch rising edge computation,
; but maximum 5 ms of delay is wanted when momentary mode is used.
; As the value of selection switch is not checked during the debouncing delay,
; it cannot fulfill the specification.

; selection switch is on pin PB0
; action switch is on pin PB1

; r16 corresponds to current selection switch value
; r17 corresponds to current action switch value

.equ clock_freq = 1000000 ; we assume 1 MHz clock

; Only 1 ms period is used in the thesis supplementary material version so it
; can be checked relatively fast for demonstration purposes.
; The deadline is also set to 1 ms there so that it just so barely does not
; meet the deadline in worst case.

.equ delay_freq = 1000

.equ delayval = (clock_freq/(4*delay_freq))


start:
	sbi DDRB, 2 ; set port B as output

	ldi r16, 0x00 ; previous selection switch value
	
loop:	
	ldi r20, 0x00 ; set current selection switch value to 0
	sbic PINB, 0
	ldi r20, 0x01 ; if the pin is 1, set it to 1

	mov r22, r16 ; test if previous and current selection switch value is the same
	sub r22, r20
	brne update_momentary_selection_mode ; if they are not, update
after_update_momentary_selection_mode:

	mov r16, r20 ; move current selection switch value to previous

	ldi r21, 0x00 ; set current action switch value to 0
	sbic PINB, 1
	ldi r21, 0x01 ; if the pin is 1, set it to 1

	tst r20
	breq update_with_action_switch_momentary
	
	; the action switch should function as switcher on rising edge
	; return to cycle if the action switch already is logic 1
	tst r21
	brne loop

	; wait 10 ms

	ldi r25, HIGH(delayval) ; load 16-bit counter
	ldi r24, LOW(delayval)
delay:
	sbiw r24, 1 ; subtract 1
	brne delay ; loop till it is zero

	; if the pin is still logic zero, return to cycle
	sbis PINB, 1
	jmp loop

	; a rising edge was detected, flip output value

	sbi PINB, 2

	jmp loop

update_momentary_selection_mode:
	tst r20 ; momentary selection mode is updated from 1 to 0, new action switch value is copied automatically
	breq after_update_momentary_selection_mode
	; momentary selection mode is updated from 0 to 1
	; zero the output
	cbi PORTB, 2
	jmp after_update_momentary_selection_mode

update_with_action_switch_momentary:
	mov r22, r21 ; load r21 to temporary
	lsl r22
	lsl r22
	out PORTB, r22
	jmp loop

