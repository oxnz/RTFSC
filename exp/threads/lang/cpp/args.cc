#include <thread>
#include <iostream>

class task {
	public:
		void pargs(const std::string &s) const {
			std::cout << s << std::endl;
		}
};

int main() {
	task tsk;
	std::string s("Hello World");
	std::thread t(&task::pargs, &tsk, s);
	t.join();
}
