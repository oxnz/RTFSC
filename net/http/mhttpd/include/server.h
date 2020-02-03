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
//#include <error.h>
#include <errno.h>
#include <syslog.h>

#include "config.h"
#include "helper.h"
#include "logger.h"
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

    server(configuration& config);
    ~server();

    void serve();
    const configuration& config;
private:
    std::mutex m_state_mutex;
    std::vector<std::thread> m_workers;
};

void* accept_request(server* server);
void* dispatch_request(server* server);

#endif//_SERVER_H_
