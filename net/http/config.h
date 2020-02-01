#ifndef CONFIG_H
#define CONFIG_H

#include <utility>
#include <chrono>

class configuration {
public:
		configuration() {
		}
private:
		size_t nreq_max;
		std::pair<size_t, size_t> nworker;
		size_t nevent;
		std::chrono::milliseconds event_tmo;
};

#endif//CONFIG_H
