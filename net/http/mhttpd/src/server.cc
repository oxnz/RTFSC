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

#include <signal.h>

#include "server.h"
#include "processor.h"
#include "process_request.h"

server::server(configuration& config) : socket(config.addr()),state (SVR_RUNNING),config(config) {
    for (size_t i = 0; i < config.concurrency(); ++i)
        m_workers.emplace_back(processor(std::string("worker-") + std::to_string(i), *this));
}

server::~server() {
    syslog(LOG_INFO, "[server] stopping");
    state = SVR_STOPPING_LISTENER;
    state = SVR_STOPPING_WORKER;
    std::for_each(m_workers.begin(), m_workers.end(), std::mem_fn(&std::thread::join));
    state = SVR_STOPPED;
    syslog(LOG_INFO, "[server] stopped");
}

void server::serve() {
    sigset_t sigset;
    siginfo_t siginfo;
#ifdef __linux__
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
            syslog(LOG_INFO, "[signal]: %d", siginfo.si_signo);
            if (siginfo.si_signo == SIGINT) {
                syslog(LOG_INFO, "[interrupted] -> stopping");
                break;
            }
        } while (1);
    }
}
