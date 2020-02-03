#ifndef REQ_HANDLER
#define REQ_HANDLER

#include "codec.h"
#include "request_pool.h"

struct request_handler {
    void operator()(request& req);
    const std::string pattern; // /path/to/xxx
};

template<typename T, typename U>
struct rpc_handler : public request_handler {
    void operate(request& req) {
        T in = m_codec.decode(req.raw);
        U out = operator()(in);
        std::string os = m_codec.encode(out);
    }
    U operator()(T& req) = delete;
    const codec<T, U> m_codec;
};

class dispatcher {
public:
    const request_handler dispatch()
    private:
        std::unordered_map<std::regex, request_handler> m_table;
};

#endif//REQ_HANDLER
