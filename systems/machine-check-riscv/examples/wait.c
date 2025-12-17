// A busy-wating example for use with Renesas FPB-R9A02G021.
//
// Unlike the basic example, a busy-waiting (spinning) loop is 
// added so that the pin 7 toggling is perceivable as blinking
// and not dimming through phase-width modulation. This increases
// the amount of states dramatically.

// Whether the button has been pressed.
volatile unsigned short button_pressed;

// Port 1 output data register.
volatile unsigned short *PORT1_PODR = (volatile unsigned short*) 0x40040020;

// Port 1 data direction register.
volatile unsigned short *PORT1_PDR = (volatile unsigned short*) 0x40040022;

// Port 1 input data register.
volatile unsigned short *PORT1_PIDR = (volatile unsigned short*) 0x40040026;

// commented-out register for easy inspection of cycle counter
/* volatile unsigned int mtime; */


int main(void) {
	// make port 1 pin 0 and 7 into an output
    main_start: *PORT1_PDR = *PORT1_PDR | (1 << 7) | 1;

    // commented-out cycle counter enable
    /* volatile unsigned int *MACTCR = (volatile unsigned int*) 0x4001A000;
    *MACTCR = 0x3; */


    main_loop: while(1) {
    	// the button is pressed whenever port 1 pin 8 is zero
    	assign1: button_pressed = ((*PORT1_PIDR & (1 << 8)) == 0);

    	// set port 1 pin 0 output value to button_pressed
    	// this makes the LED light up whenever the user button is pressed
    	// on the development kit
        assign2: *PORT1_PODR = (*PORT1_PODR & ~1) | button_pressed;

		// invert the port 1 pin 7 output value
		assign3: *PORT1_PODR = *PORT1_PODR ^ (1 << 7);

    	// commented-out cycle counter value fetch
        /* mtime = *((volatile unsigned int*)0xE6000000); */

    	// Delay for 100 ms.
    	//
    	// On reset, the R9A02G021 uses the internal 8 MHz oscillator (MOCO).
    	// This is fed to System Clock via a clock divider, 1/16 on reset
    	// i.e. the frequency on reset is approx. 500 kHz.
    	//
    	// We use a loop that should take 5 cycles (confirmed by measurement).
        // The iteration dispatch frequency is 500 kHz / 5 = 100 kHz.
    	// We use a 10k divisor to obtain 100 kHz / 10k = 10 Hz, i.e. 100 ms.
    	//
        // This means that the actual frequency of blinking is 5 Hz,
        // as a single iteration of the main loop flips the phase once
        // and two flips need to be done for a full blink.

    	asm volatile("li a0, 10000\n"
    			"1: addi a0, a0, -1\n"
    			"nop\n"
    			"nop\n"
    			"nop\n"
    			"bnez a0, 1b": : : "a0");
    }
    // should never be reached
    main_return: return 0;
}
