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

#include <stdlib.h>
#include <errno.h>

#include "ring_buffer.h"

struct ring_buffer* ring_buffer_create(size_t count) {
	struct ring_buffer *buf = malloc(sizeof(struct ring_buffer));
	if (buf) {
		buf->buf = malloc(sizeof(void *) * count);
		if (buf->buf == NULL) {
			free(buf);
			return NULL;
		}
		buf->size = count;
		buf->in = 0;
		buf->out = 0;
		pthread_mutex_init(&buf->mutex, NULL);
		sem_init(&buf->nelem_sem, 0, 0);
		sem_init(&buf->nempt_sem, 0, count);
	}
	return buf;
}

int ring_buffer_destroy(struct ring_buffer* buf) {
	free(buf->buf);
	free(buf);
	return 0;
}

void ring_buffer_enqueue(struct ring_buffer *buffer, void *val) {
	sem_wait(&buffer->nempt_sem);

	pthread_mutex_lock(&buffer->mutex);
	buffer->buf[buffer->in] = val;
	++(buffer->in);
	if (buffer->in == buffer->size)
		buffer->in = 0;
	pthread_mutex_unlock(&buffer->mutex);

	sem_post(&buffer->nelem_sem);
}

void* ring_buffer_dequeue(struct ring_buffer *buffer) {
	sem_wait(&buffer->nelem_sem);

	pthread_mutex_lock(&buffer->mutex);
	void *val = buffer->buf[buffer->out];
	++(buffer->out);
	if (buffer->out == buffer->size)
		buffer->out = 0;
	pthread_mutex_unlock(&buffer->mutex);

	sem_post(&buffer->nempt_sem);
	return val;
}

void* ring_buffer_try_dequeue(struct ring_buffer *buffer) {
	errno = 0;
	sem_trywait(&buffer->nelem_sem);
	if (errno == EAGAIN)
		return NULL;
	pthread_mutex_lock(&buffer->mutex);
	void *val = buffer->buf[buffer->out];
	++(buffer->out);
	if (buffer->out == buffer->size)
		buffer->out = 0;
	pthread_mutex_unlock(&buffer->mutex);

	sem_post(&buffer->nempt_sem);

	return val;
}
