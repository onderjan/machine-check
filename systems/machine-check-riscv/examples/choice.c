// A branching-choice example for use with Renesas FPB-R9A02G021.
//
// Unlike the basic example, the button chooses exactly one
// of the user LEDs to be lit within the loop. The choice is
// stored in the new 'output' volatile variable for further
// inspection.
//
// The condition tends to produce a branch without optimisation.


// Whether the button has been pressed.
volatile unsigned short button_pressed;

volatile unsigned short output;

// Port 1 output data register.
volatile unsigned short *PORT1_PODR = (volatile unsigned short*) 0x40040020;

// Port 1 data direction register.
volatile unsigned short *PORT1_PDR = (volatile unsigned short*) 0x40040022;

// Port 1 input data register.
volatile unsigned short *PORT1_PIDR = (volatile unsigned short*) 0x40040026;


int main(void) {
	// make port 1 pin 0 and 7 into an output
    main_start: *PORT1_PDR = *PORT1_PDR | (1 << 7) | 1;

    main_loop: while(1) {
    	// the button is pressed whenever port 1 pin 8 is zero
    	assign1: button_pressed = ((*PORT1_PIDR & (1 << 8)) == 0);

    	// make the button control which user LED lights up (one of two)
    	if (button_pressed) {
        	// set port 1 pin 0 output value to 1
    		// and port 1 pin 7 output value to 0
    		output = (*PORT1_PODR & ~(1 << 7)) | 1;
    	} else {
        	// set port 1 pin 0 output value to 0
    		// and port 1 pin 7 output value to 1
    		output = (*PORT1_PODR & ~1) | (1 << 7);
    	}

    	// actually set the output to the variable value
    	assign2: *PORT1_PODR = output;
    }
    // should never be reached
    main_return: return 0;
}
