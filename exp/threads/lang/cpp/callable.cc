#include <thread>
#include <iostream>

class task {
	private:
		void foo() const {
			std::cout << "foo" << std::endl;
		}
		void bar() const {
			std::cout << "bar" << std::endl;
		}
	public:
		void operator() () const {
			foo();
			bar();
		}
};

int main() {
	std::thread t((task()));
	t.join();
}
