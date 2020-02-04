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

    enum event_flag {
        EF_READABLE = 1<<0,
        EF_WRITABLE = 1<<1,
        EF_ERROR = 1<<2,
        EF_EOF = 1 << 3,
        };

class event : kevent {
public:
    bool readable() const {
        return filter == EVFILT_READ;
    }
    bool writable() const {
        return filter == EVFILT_WRITE;
    }
    bool error() const {
        return flags & EV_ERROR;
    }
    bool eof() const {
        return flags & EV_EOF;
    }
    int fd() const {
        return ident;
    }
    std::uintptr_t ptr() const {
        return reinterpret_cast<std::uintptr_t>(udata);
    }
    std::string repr() const {
        std::stringstream ss;
        ss << "event<" << this
           << ">[" << "fd=" << ident << ","
           << "flags=" << (readable() ? "r" : "-")
           << (writable() ? "w" : "-")
           << (error()? "e" : "-")
           << (eof()? "f" : "-")
           << "]";
        return ss.str();
    }
};

class event_loop {
public:
    event_loop(size_t n, const struct timespec& interval);
    event_loop(const event_loop&) = delete;
    event_loop(event_loop&&) = default;
    event_loop& operator=(const event_loop&) = delete;
    ~event_loop();

    void event(const event& ev);
    void operator()(int fd, size_t flags, size_t data);
    template<typename Fn>
    void operator()(Fn& fn) {
        m_interval.tv_sec = 2;
        int n = kevent(m_fd, (struct kevent*)m_changes, size, (struct kevent*)m_events, capacity, &m_interval);
        size = 0;
        if (-1 == n) throw std::runtime_error(strerror(errno));
        for (int i = 0; i < n; ++i) {
            auto& ev = m_events[i];
            syslog(LOG_DEBUG, "[event_loop] %s", ev.repr().c_str());
            fn(ev);
        }
    }
    bool full() const { return size >= capacity; }
private:
#ifdef __APPLE__
    class event* m_changes;
    class event* m_events;
    size_t size;
    size_t capacity;
    struct timespec m_interval;
#elif __linux__
    m_events(new struct epoll_event[capacity]),
#endif
    int m_fd;
};

#endif//_EVENT_H_


