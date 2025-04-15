/*
 * calibration.c
 *
 * Created: 16.07.2024 17:36:43
 * Author : Jan Onderka
 */ 

#define F_CPU 1000000

#include <avr/io.h>
#include <util/delay.h>

int main(void)
{
	
	DDRC |= 0x01;
	DDRD |= 0xFF;
    while (1) 
    {
		while ((PINC & 0x2) == 0) {}
		
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
			uint8_t input_value = PINB;
			
			if ((input_value & 0x80) == 0) {
				// input value lower than desired -> we should lower the calibration value
				search_val &= ~search_mask;	
			}
			
			if (search_bit == 0) {
				// all bits have been set, stop
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

