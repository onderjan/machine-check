/*
 * stack.c
 *
 * Created: 16.07.2024 16:20:08
 * Author : Jan Onderka
 */ 

#include <avr/io.h>

int factorial(uint8_t n) {
	if (n == 0) {
		return 1;
	}
	return n * factorial(n - 1);
}

int main(void)
{
    DDRD |= 0xFF;
	while (1)
    {
		//uint8_t read = 5;
		uint8_t read = (uint8_t) PINB & 0x07;
		//uint8_t read = (uint8_t) PINB & 0xFF;
		//uint8_t read = (uint8_t) PINB;
		//uint8_t write =(uint8_t)(0) - (uint8_t)(1);
		//if (read <= 5) {
			uint8_t write = factorial(read);
		//}
		PORTD = write;
    }
}

