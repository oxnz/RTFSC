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

#include <exception>
#include <error.h>
#include <errno.h>
#include <syslog.h>
#include <pthread.h>

#include "thread_pool.h"
#include "server_socket.h"
#include "process_request.h"

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
		server_socket socket;
		enum server_status state;
		pthread_mutex_t state_mutex;
		concurrent_vector<request> requests;
		ssize_t nrequest;
		ssize_t nrequest_max;
		ssize_t nworker;
		ssize_t nworker_min;
		ssize_t nworker_max;
		size_t nevent;
		ssize_t event_tmo;
		server();
		~server();

		void startup();
		void shutdown();
		void restart();
		void pause();
		void resume();
		void serve();
private:
		multiprocessing::thread_pool m_pool;
};

void* accept_request(server* server);
void* dispatch_request(server* server);

inline void require(bool cond, const char* msg) {
		if (!cond) throw std::invalid_argument(msg);
}

#endif//_SERVER_H_
