// A basic RISC-V program example for use with Renesas FPB-R9A02G021.
//
// We manipulate the port 1 in the following fashion. 
// Pin 0 and 7 (which control LEDs on the board) are set to output. 
// Then, in a loop, the outputs for LEDs are manipulated so that 
// the pin 0 LED is lit (logic high) whether the button is pressed 
// (pin 8 logic low) and pin 7 LED toggles on each iteration.


// Whether the button has been pressed.
volatile unsigned short button_pressed;

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

    	// set port 1 pin 0 output value to button_pressed
    	// this makes the LED light up whenever the user button is pressed
    	// on the development kit
        assign2: *PORT1_PODR = (*PORT1_PODR & ~1) | button_pressed;

        // invert the port 1 pin 7 output value
        assign3: *PORT1_PODR = *PORT1_PODR ^ (1 << 7);
    }
    // should never be reached
    main_return: return 0;
}
