#ifndef REQUEST_POOL_H
#define REQUEST_POOL_H

#include <unordered_map>
#include <string>
#include <unistd.h>
#include <pthread.h>
#include <semaphore.h>
#include <array>
#include <vector>
#include <mutex>

enum method { HEAD, GET, POST };

enum state {
		CONN_ESTABLISHED,
		READING_REQ_HEADER,
		READING_REQ_BODY,
		REQ_RCVD,
		SENDING_RESP_HEADER,
		SENDING_RESP_BODY,
		RESP_SENT,
		CONN_ABORTED,
};

const size_t REQMAXLEN = 512;
const size_t RSPMAXLEN = 512;
const size_t URIMAXLEN = 1024;

struct response {
		int code;
		std::string reason;
		char header[RSPMAXLEN];
		off_t header_offset;
		size_t header_length;
		off_t body_offset;
		size_t body_length;

		const char *content_type;
		const char *charset;
		int fd;
		int has_body;
};

struct request {
		std::string addr;
		int sockfd;
		char raw[REQMAXLEN];
		size_t offset;
		enum state state;
		enum method method;
		char *uri;
		struct response resp;
		bool readable;
		bool writable;
		request() : request(-1) {}
		request(int fd): sockfd(fd), raw("\0"), offset(0), state(CONN_ESTABLISHED), uri(NULL) { }
};

#include <shared_mutex>
#include <iostream>

template <typename T>
class concurrent_vector {
		public:
				template <class... Args>
						void emplace (Args&&... args) {
								//m_store.emplace(args...);
						}
				void enqueue(const T& val) {
						std::lock_guard<std::mutex> xlock(m_mutex);
						m_store.push_back(val);
				}
				bool dequeue(T& val) {
						std::lock_guard<std::mutex> xlock(m_mutex);
						if (m_store.empty()) return false;
						std::swap(m_store.back(), val);
						m_store.pop_back();
						return true;
				}
				bool try_dequeue(T& val) {
						std::unique_lock<std::mutex> xlock(m_mutex, std::try_to_lock);
						if(!xlock.owns_lock() || m_store.empty()) return false;
						std::swap(m_store.back(), val);
						m_store.pop_back();
						return true;
				}
				bool empty() const {
						std::shared_lock<std::shared_mutex> slock(m_mutex);
						return m_store.empty();
				}
				size_t size() const {
						std::shared_lock<std::shared_mutex> slock(m_mutex);
						return m_store.size();
				}
		private:
				std::mutex m_mutex;
				std::vector<T> m_store;
};

#endif//REQUEST_POOL_H
