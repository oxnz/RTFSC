#ifndef PROCESSOR_H
#define PROCESSOR_H

#include <unordered_set>
#include <vector>

#include "event.h"
#include "request_pool.h"

class server;

class processor {
public:
    using ident = size_t;
    processor(ident id, const server& server);
    processor(ident id, const server& server, size_t capacity);
    processor(const processor&) = delete ;
    processor(processor&&) = default;
    processor& operator=(processor&&) = delete;
    processor& operator=(const processor&) = delete;
    ~processor();

    ident id() const { return m_id; }
    std::string name() const { return m_name; }
    void operator()();
    bool operator()(const event& ev);
private:
    size_t capacity() const { return m_capacity; }
    bool full() const { return m_unused.empty(); }
private:
    const ident m_id;
    const std::string m_name;
    const server& m_server;
    event_loop m_evloop;
    std::unordered_set<size_t> m_unused;
    std::vector<request> m_requests;
    size_t m_capacity;
};

#endif//PROCESSOR_H
