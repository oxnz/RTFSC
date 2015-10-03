#include <atomic>

class spinlock_mutex {
public:
	spinlock_mutex() : flag(ATOMIC_FLAG_INIT) {}
	void lock() {
		while (flag.test_and_set(std::memory_order_acquire)) ;
	}
	void unlock() {
		flag.clear(std::memory_order_release);
	}
	bool try_lock() {
		return flag.test_and_set(std::memory_order_acquire) == false;
	}
private:
	std::atomic_flag flag;
};

#include <iostream>
#include <thread>
#include <vector>

spinlock_mutex mutex;
int v = 0;

void f() {
	std::lock_guard<spinlock_mutex> lock(mutex);
	std::cout << std::this_thread.id << std::endl;
	for (int i = 0; i < 10; ++i) {
		std::cout << ++v << std::endl;
	}
}

int main() {
	std::vector<std::thread> tv;
	for (int i = 0; i < 5; ++i)
		tv.push_back(std::thread(f));
	for (std::thread &t : tv)
		t.join();

	exit(0);
}
