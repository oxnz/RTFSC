/*
 * Filename:	ring_buffer.h
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2016-06-25 11:24:04 CST]
 * Last-update:	2016-06-25 11:24:04 CST
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

#ifndef _RING_BUFFER_H_
#define _RING_BUFFER_H_

#include <unistd.h>
#include <pthread.h>
#include <semaphore.h>

#define RBMAXSIZE 1024

struct ring_buffer {
	void** buf;
	size_t size;
	int in;
	int out;
	pthread_mutex_t mutex;
	sem_t nelem_sem;
	sem_t nempt_sem;
};

struct ring_buffer* ring_buffer_create(size_t count);
int ring_buffer_destroy(struct ring_buffer* buf);

void ring_buffer_enqueue(struct ring_buffer *buffer, void *val);
void* ring_buffer_dequeue(struct ring_buffer *buffer);
void* ring_buffer_try_dequeue(struct ring_buffer *buffer);

/*
 * TODO: refine API
 * void* ring_buffer_dequeue(struct ring_buffer *buffer, int block);
 * ssize_t ring_buffer_resize(struct ring_buffer *buffer, size_t capacity);
 * size_t ring_buffer_clear(struct ring_buffer *buffer);
 */

#endif//_RING_BUFFER_H_
