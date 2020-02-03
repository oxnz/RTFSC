/*
 * Filename:	event.h
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2020-02-03 20:40:41 CST]
 * Last-update:	2020-02-03 20:40:41 CST
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

#ifndef _EVENT_H_
#define _EVENT_H_

#include <unistd.h>
#include "config.h"

#ifdef __linux__
#include <sys/epoll.h>
#elif __APPLE__
#include <sys/types.h>
#include <sys/event.h>
#include <sys/time.h>
#endif

struct event {
    size_t data;
    bool readable;
    bool writable;
    bool eof;
    bool error;
};

struct event_ctx {
    event_ctx();
    ~event_ctx();
    int fd;
};

struct event_loop {
    event_loop(size_t n, const struct timespec& interval);
    virtual ~event_loop();
    void add_event(const event& ev);
    void del_event(const event& ev);
    void operator()(std::function<bool(const struct event& ev)>& fn);
#ifdef __linux__

#elif __APPLE__
    struct kevent* m_changelist;
    struct kevent* m_eventlist;
    size_t size;
    struct timespec m_interval;
    struct event_ctx m_ctx;
#endif
};

#ifdef __linux__
#elif __APPLE__

event_ctx::event_ctx() : fd(kqueue()) {

}

event_ctx::~event_ctx() {
    close(fd);
}

event_loop::event_loop(size_t n, const struct timespec& interval) : m_changelist(new struct kevent[n]), m_eventlist(new struct kevent[n]), size(n), m_interval(interval) {
}

event_loop::~event_loop() {
    delete[] m_changelist;
    delete[] m_eventlist;
}

void event_loop::add_event(const event& ev) {
    struct kevent& kev = m_eventlist[ev.data];
    kev.flags |= EV_ENABLE;
}

void event_loop::del_event(const event &ev) {
    struct kevent& kev = m_eventlist[ev.data];
    kev.flags |= EV_DISABLE;
}

void event_loop::operator()(std::function<bool(const struct event& ev)>& fn) {
    int n = kevent(m_ctx.fd, m_changelist, size, m_eventlist, size, &m_interval);
    if (-1 == n) throw std::runtime_error(strerror(errno));
    struct event ev;
    for (int i = 0; i < n; ++i) fn(ev);
}

#endif

#endif//_EVENT_H_


