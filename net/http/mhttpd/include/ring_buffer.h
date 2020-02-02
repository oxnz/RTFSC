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
#include <array>

const size_t RBMAXSIZE = 1024;

template<typename T, std::size_t N>
struct ring_buffer {
		std::array<T, N> buf;
		int in;
		int out;
		pthread_mutex_t mutex;
		sem_t nelem_sem;
		sem_t nempt_sem;
		ring_buffer(): in(0), out(0) {
				pthread_mutex_init(&mutex, NULL);
				sem_init(&nelem_sem, 0, 0);
				sem_init(&nempt_sem, 0, N);
		}
		~ring_buffer() {
				sem_destroy(&nelem_sem);
				sem_destroy(&nempt_sem);
		}
		void enqueue(const T& val) {
				sem_wait(&nempt_sem);

				pthread_mutex_lock(&mutex);
				buf[in] = val;
				++in;
				if (in == buf.size()) in = 0;
				pthread_mutex_unlock(&mutex);

				sem_post(&nelem_sem);
		}
		T dequeue() {
				T val;
				sem_wait(&nelem_sem);

				pthread_mutex_lock(&mutex);
				std::swap(val, buf[out]);
				++out;
				if (out == buf.size()) out = 0;
				pthread_mutex_unlock(&mutex);

				sem_post(&nempt_sem);
				return val;
		}
		bool try_dequeue(T& val) {
				errno = 0;
				sem_trywait(&nelem_sem);
				if (errno == EAGAIN) return false;
				pthread_mutex_lock(&mutex);
				std::swap(val, buf[out]);
				++out;
				if (out == buf.size()) out = 0;
				pthread_mutex_unlock(&mutex);

				sem_post(&nempt_sem);

				return true;
		}

		/*
		 * TODO: refine API
		 * void* ring_buffer_dequeue(struct ring_buffer *buffer, int block);
		 * ssize_t ring_buffer_resize(struct ring_buffer *buffer, size_t capacity);
		 * size_t ring_buffer_clear(struct ring_buffer *buffer);
		 */

};

#endif//_RING_BUFFER_H_
