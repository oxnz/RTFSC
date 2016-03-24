/*
 * Filename:	test.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-09-23 11:19:50 CST]
 * Last-update:	2015-09-23 11:19:50 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License: 
 * Copyright (c) 2013 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

struct T {
	union {
		uint16_t v;
		struct {
		uint8_t a:4;
		uint8_t b:4;
		uint8_t c:4;
		uint8_t d:4;
		};
	};
};

int main() {
	struct T t = {
		.a = 1,
		.b = 2,
		.c = 3,
		.d = 4,
	};

	printf("v = 0x%04x, a = %d, b = %d, c = %d, d = %d\n", t.v, t.a, t.b, t.c, t.d);

	exit(0);
}
