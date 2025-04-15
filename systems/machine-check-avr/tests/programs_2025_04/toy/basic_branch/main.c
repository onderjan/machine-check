/*
 * Basic branch example.
 * Just set an output pin depending on an input pin.
 */ 

#include <avr/io.h>


int main(void)
{
	DDRB |= 1 << 1; // set port B pin 1 as output

	while(1)
	{
		if ((PINB & (1<<0)) == 1 ) { // if port B pin 0 input is set
			PORTB |= 1 << 1; // set port B pin 0 output value to 1
			} else {
			PORTB &= ~(1 << 1); // otherwise, set it to zero
		}
	}
	return 0;
}

