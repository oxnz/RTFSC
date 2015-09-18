/*
 * Filename:	ringbuffer.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-09-18 12:04:44 CST]
 * Last-update:	2015-09-18 12:04:44 CST
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

struct circ_buf {
	char *buf;
	int head;
	int tail;
};

/* Return count in buffer.  */
int circ_buf_cnt(int head, int tail, int size) {
	return (head - tail) & (size - 1);
}

/* Return space available, 0..size-1.  We always leave one free char
   as a completely full buffer has head == tail, which is the same as
   empty.  */
int circ_buf_space(int head, int tail, int size) {
	return circ_buf_cnt(tail, head+1, size);
}

/* Return count up to the end of the buffer.  Carefully avoid
   accessing head and tail more than once, so they can change
   underneath us without returning inconsistent results.  */
int circ_buf_cnt_to_end(int head, int tail, int size) {
	int end = (size) - (tail);
	int n = ((head) + end) & ((size)-1);
	return n < end ? n : end;
}

/* Return space available up to the end of the buffer.  */
int circ_buf_space_to_end(int head, int tail, int size) {
	int end = size - 1 -head;
	int n = (end + tail) & (size - 1);
	return n <= end ? n : end+1;
}
