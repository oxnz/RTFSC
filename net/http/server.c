/*
 * Filename:	server.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		2016-06-26 18:25:58 CST
 * Last-update:	2016-06-26 18:25:58 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License:
 * Copyright (c) 2016 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#include <stdio.h>
#include <stdlib.h>

#include "server.h"
#include "process_request.h"

static int server_validate(struct server *server) {
	if (server == NULL)
		return -1;
	if (server->nrequest_max < 1)
		return -1;
	if (server->nworker_min < 1)
		return -1;
	if (server->nworker_max < 1)
		return -1;
	if (server->nworker_max < server->nworker_min)
		return -1;
	if (server->nevent < 1)
		return -1;
	if (server->event_tmo < 1)
		return -1;
	return 0;
}

int server_init(struct server *server) {
	int i;
	if (server_validate(server)) {
		perror("malformed server");
		return -1;
	}
	server->nworker = 0;
	server->state = SVR_STOPPED;
	if (pthread_mutex_init(&server->state_mutex, NULL)) {
		perror("pthread_mutex_init");
		return -1;
	}
	if (server_socket_init(&server->server_socket)) {
		perror("server_socket_init");
		return -1;
	}
	server->requests = ring_buffer_create(server->nrequest_max);
	if (server->requests == NULL) {
		perror("ring_buffer_create");
		return -1;
	}
	server->workers = malloc(sizeof(pthread_t) * server->nworker_max);
	if (server->workers == NULL) {
		return -1;
	}
	for (i = 0; i < server->nworker_min; ++i) {
		if (pthread_create(&server->workers[i], NULL,
					(void *(*)(void *))dispatch_request, server) != 0) {
			perror("pthread_create");
		}
		++(server->nworker);
	}
	if (pthread_create(&server->listener, NULL, (void *(*)(void *))accept_request,
				&server->server_socket) != 0) {
		perror("pthread_create");
		return -1;
	}
	return 0;
}

int server_destroy(struct server* server) {
	if (server == NULL)
		return -1;
	if (server->state == SVR_RUNNING) {
		if (server_shutdown(server)) {
			perror("server_shutdown");
			return -1;
		}
	}
	if (pthread_mutex_destroy(&server->state_mutex)) {
		perror("pthread_mutex_destroy");
		return -1;
	}
	if (server_socket_destroy(&server->server_socket)) {
		perror("server_socket_destroy");
		return -1;
	}
	if (ring_buffer_destroy(server->requests)) {
		perror("ring_buffer_destroy");
		return -1;
	}
	if (server->workers) {
		free(server->workers);
	}
	return 0;
}

int server_startup(struct server *server) {
	server->state = SVR_RUNNING;
}

int server_shutdown(struct server *server) {
	int i;
	server->state = SVR_STOPPING_LISTENER;
	pthread_join(server->listener, NULL);
	printf("accept quit\n");
	server->state = SVR_STOPPING_WORKER;
	for (i = 0; i < server->nworker; ++i) {
		pthread_join(server->workers[i], NULL);
	}
	printf("workers exit\n");
	ring_buffer_destroy(server->requests);
	server->state = SVR_STOPPED;
}

int server_pause(struct server *server) {
	server->state = SVR_PAUSED;
}

int server_resume(struct server *server) {
	server->state = SVR_RUNNING;
}
