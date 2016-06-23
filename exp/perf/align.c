/*
 * Filename:	memcp.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		2015-09-15 13:41:03 CST
 * Last-update:	2015-09-15 13:41:03 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License:
 * Copyright (c) 2015 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>

size_t tdiff(struct timeval *t1, struct timeval *t2) {
	if (t1 == NULL && t2 == NULL)
		return 0;
	size_t diff = t2->tv_sec - t1->tv_sec;
	diff *= 1000000;
	diff += t2->tv_usec - t1->tv_usec;
	return diff;
}

void alianed_read(long *p, int n) {
	volatile long v;
	long *pe = p + n;

	p += 1;
	printf("aligned read p = %p\n", p);
	while (p < pe)
		v = *p++;
}

void unaligned_read(long* p, int n) {
	volatile long v;
	long *pe = p + n;
	char *pc = (char *)p;
	pc += 1;
	p = (long *)pc;

	printf("unaligned read p = %p\n", p);
	while (p < pe)
		v = *p++;
}

#define N (1 << 20)

int main(int argc, char *argv[]) {
	long *p1 = malloc(sizeof(long)*N);
	long *p2 = malloc(sizeof(long)*N);
	struct timeval t0, t1, t2;

	gettimeofday(&t0, NULL);
	alianed_read(p2, N);
	gettimeofday(&t1, NULL);
	unaligned_read(p1, N);
	gettimeofday(&t2, NULL);
	printf("%lu vs %lu\n", tdiff(&t0, &t1), tdiff(&t1, &t2));

	exit(0);
}
