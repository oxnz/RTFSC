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

#include "config.h"
#include "processor.h"
#include "server_socket.h"

struct server {

    server(configuration& config);
    ~server();

    void serve();
    bool running() const {return m_state == state::RUNNING;}
    const configuration& config;
    server_socket socket;
private:
    enum class state {
        RUNNING,
        PAUSED,
        STOPPING_LISTENER,
        STOPPING_WORKER,
        STOPPED,
    } m_state;
    std::mutex m_state_mutex;
    std::vector<processor> m_processors;
    std::vector<std::thread> m_workers;
};

void* accept_request(server* server);
void* dispatch_request(server* server);

#endif//_SERVER_H_
