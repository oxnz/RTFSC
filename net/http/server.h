/*
 * Filename:	server.h
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2016-06-26 18:20:22 CST]
 * Last-update:	2016-06-26 18:20:22 CST
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

#ifndef _SERVER_H_
#define _SERVER_H_

#include <pthread.h>

#include "ring_buffer.h"
#include "server_socket.h"

#define REQMAXLEN 512
#define RSPMAXLEN 512
#define URIMAXLEN 1024
#define TCP_SERVICE_PORT 8000
#define NWORKER 2
#define NEVENT 128
#define NEVTMO 1000

enum server_status {
	SVR_RUNNING,
	SVR_PAUSED,
	SVR_STOPPING_LISTENER,
	SVR_STOPPING_WORKER,
	SVR_STOPPED,
};

struct server {
	struct server_socket server_socket;
	enum server_status state;
	pthread_mutex_t state_mutex;
	struct ring_buffer *requests;
	ssize_t nrequest;
	ssize_t nrequest_max;
	pthread_t *workers;
	ssize_t nworker;
	ssize_t nworker_min;
	ssize_t nworker_max;
	pthread_t listener;
	size_t nevent;
	ssize_t event_tmo;
};

int server_init(struct server *svr);
int server_destroy(struct server *svr);

int server_startup(struct server *svr);
int server_shutdown(struct server *svr);
int server_restart(struct server *svr);

int server_pause(struct server *svr);
int server_resume(struct server *svr);

#endif//_SERVER_H_
