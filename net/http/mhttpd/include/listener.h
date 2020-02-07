#ifndef LISTENER_H
#define LISTENER_H
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <errno.h>
#include <unistd.h>
#include <err.h>

#include <netinet/tcp.h>
#include <sys/types.h>
#include <sys/stat.h>

#include <fcntl.h>
#include <pthread.h>
#include <semaphore.h>

#include "process_request.h"
#include "request_pool.h"
#include "event.h"
#include "server.h"

class listener {
    struct event_loop evloop;
public:
    listener() : evloop(10, timespec{1, 2}) {
    }
    bool operator()(const struct event& ev) {
        if (ev.readable()) {
            int sockfd = accept(ev.data, NULL, NULL);
        }

        return true;
    }
    void operator()(server* p) {
        int sockfd;
        int nev;
        int tmo = 10;
        int optval = 1;
        int i;
        server& server = *p;

        struct event ev{12, 12};
        if (!evloop.add_event(ev)) {
            throw std::runtime_error("add_event");
        }

        for (;server.state != SVR_STOPPING_LISTENER;) {
            evloop(*this);


        }
    }
};

#endif//LISTENER_H
