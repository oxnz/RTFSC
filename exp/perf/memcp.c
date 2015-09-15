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

#define N 1000000

void memcp1(void * restrict src, void * restrict dst, int n) {
	char *esi = src;
	char *edi = dst;
	while (n--)
		*edi++ = *esi++;
}

void memcp2(void * restrict src, void * restrict dst, int n) {
	long *p1 = (long *)src;
	long *p2 = (long *)dst;
	int i;
	for (i = 0; i < n / 8; ++i) {
		*p2++ = *p1++;
	}
	char *esi = (char *)p1;
	char *edi = (char *)p2;
	for (i = 0; i < n % 8; ++i)
		*edi++ = *esi++;
}

int main(int argc, char *argv[]) {
	char *p1 = malloc(sizeof(char)*N);
	char *p2 = malloc(sizeof(char)*N);
	struct timeval t0, t1, t2;
	int i;
	for (i = 0; i < 4; ++i) {
		gettimeofday(&t0, NULL);
		memcp1(p1, p2, N);
		gettimeofday(&t1, NULL);
		memcp2(p1, p2, N);
		gettimeofday(&t2, NULL);
		printf("%lu vs %lu\n", tdiff(&t0, &t1), tdiff(&t1, &t2));
	}
	exit(0);
}
