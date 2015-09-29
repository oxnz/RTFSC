#include "utils.h"
#include <stdint.h>
#include <ctype.h>
#include <stdio.h>

void hexdump(void *p, int len) {
	uint8_t *str = (uint8_t *)p;
	int line = len / 16 + (len % 16 > 0);
	int remainCount = len;
	int limit;
	int l;
	int i;
	for (l = 0; l < line; ++l) {
		printf("%08x  ", l << 4);
		limit = (remainCount > 16 ? 16 : remainCount);
		for (i = 0; i < limit; ++i) {
			if (i == 8)
				putchar(' ');
			printf("%02x ", str[(l << 4) + i]);
		}
		if (i <= 8)
			putchar(' ');
		for (i = 0; i < 16 - limit; ++i)
			printf("   ");
		printf(" |");
		for (i = 0; i < limit; ++i) {
			if (isprint(str[(l << 4) + i]))
				printf("%c", str[(l << 4) + i]);
			else
				putchar('.');
		}
		printf("|");
		printf("\n");
		remainCount -= limit;
	}
}

