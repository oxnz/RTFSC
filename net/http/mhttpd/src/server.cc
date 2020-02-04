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

#include <csignal>

#include "server.h"
#include "processor.h"
#include "process_request.h"

server::server(configuration& config)
    :
    config(config),
    socket(config.addr()),
    m_state(state::RUNNING) {
    syslog(LOG_DEBUG, "[server] worker: %ld", config.concurrency());
    //    for (size_t i = 0; i < config.concurrency(); ++i)
    for (size_t i = 0; i < 1; ++i) {
        m_processors.emplace_back(i, *this);
        m_workers.emplace_back(std::ref(m_processors.back()));
    }
}

server::~server() {
    syslog(LOG_INFO, "[server] stopping");
    m_state = state::STOPPING_LISTENER;
    m_state = state::STOPPING_WORKER;
    std::for_each(m_workers.begin(), m_workers.end(), std::mem_fn(&std::thread::join));
    m_state = state::STOPPED;
    syslog(LOG_INFO, "[server] stopped");
}

void server::serve() {
    sigset_t sigset;
#ifdef __linux__
    siginfo_t siginfo;
    struct timespec tmo = config.signal_timeout();
#endif
    { // blocking all signals
        sigfillset(&sigset);
        if (pthread_sigmask(SIG_BLOCK, &sigset, NULL) != 0) {
            throw std::runtime_error("pthread_sigmask");
        }
    }
    { // signal handling
        sigemptyset(&sigset);
        sigaddset(&sigset, SIGINT);
        sigfillset(&sigset);
        do {
#ifdef __linux__
            int signo = sigtimedwait(&sigset, &siginfo, &tmo);
#elif __APPLE__
            int signo;
            int ret = sigwait(&sigset, &signo);
            if (ret == -1) signo = ret;
#endif
            if (signo == -1) {
                if (errno == EAGAIN || errno == EINTR) {
                    continue;
                }
                syslog(LOG_ERR, "sigwaitinfo");
                break;
            }
            //dispatch_signal(siginfo);
            syslog(LOG_INFO, "[signal]: %d", signo);
            if (signo == SIGINT || signo == SIGTERM) {
                syslog(LOG_INFO, "[interrupted] -> stopping");
                break;
            }
        } while (1);
    }
}
