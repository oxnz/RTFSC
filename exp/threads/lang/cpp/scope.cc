#include <thread>
#include <iostream>

class scoped_thread {
public:
	explicit scoped_thread(std::thread t) : m_t(std::move(t)) {
		if (!m_t.joinable())
			throw std::logic_error("No thread");
	}
	~scoped_thread() {
		m_t.join();
	}
	scoped_thread(scoped_thread const&) = delete;
	scoped_thread& operator=(scoped_thread const&) = delete;
private:
	std::thread m_t;
};

void do_something(int i) {
	std::cout << i << std::endl;
}

struct func {
public:
	func(int& i) : m_i(i) {}
	void operator() () {
		for (int i = 0; i < 10; ++i) {
			do_something(m_i);
		}
	}
private:
	int& m_i;
};

void do_something_in_current_thread() {
	std::cout << "thread about to join" << std::endl;
}

int main() {
	int local_state;
	scoped_thread t((std::thread(func(local_state))));
	do_something_in_current_thread();
}
