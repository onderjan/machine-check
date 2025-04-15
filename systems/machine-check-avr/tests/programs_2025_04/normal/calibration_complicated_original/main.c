/*
 * calibration.c
 *
 * Created: 16.07.2024 17:36:43
 * Author : Jan Onderka
 */ 

#define F_CPU 1000000

#include <avr/io.h>
#include <util/delay.h>

volatile uint8_t irrelevant[16];

volatile uint8_t pinb;
volatile uint8_t pinc;

int main(void)
{
	// demonstration: irrelevant reads do not have an effect on verification speed
	for (uint8_t i = 0; i < 8; ++i) {
		irrelevant[i] = PINB;
	}
	
	DDRC |= 0x01;
	DDRD |= 0xFF;
    while (1) 
    {
		// demonstration: replaced read with read to volatile, all 8 bits are read and then masked
		//while ((PINC & 0x2) == 0) {}
		while (1) {
			pinc = PINC;
			if ((pinc & 0x2) == 0) {
				break;
			}
		}
		
		// signify we are calibrating
		PORTC |= 0x01;
		
		// start with MSB of calibration
		uint8_t search_bit = 7;
		uint8_t search_mask = (1 << search_bit);
		uint8_t search_val = search_mask;
		
		while (1) {
			// wait a bit
			_delay_us(10);
			// write the current search value
			PORTD = search_val;
			// wait a bit
			_delay_us(10);
			
			// get input value and compare it to desired
			// demonstration: replaced read with read to volatile, all 8 bits are read and then masked
			//uint8_t input_value = PINB;
			pinb = PINB;
			register uint8_t input_value = pinb;
			
			if ((input_value & 0x80) == 0) {
				// input value lower than desired -> we should lower the calibration value
				search_val &= ~search_mask;	
			}
			
			if (search_bit == 0) {
				// all bits have been set, stop
				
				// FIX OF BUG FOUND BY TOOL: update the search value
				//PORTD = search_val;
				break;
			}
			
			search_bit -= 1;
			// continue to next bit
			search_mask >>= 1;
			// update the search value with the next bit set
			search_val |= search_mask;
			
		}
		
		// calibration complete, stop signifying that we are calibrating
		PORTC &= ~0x01;
    }
}

