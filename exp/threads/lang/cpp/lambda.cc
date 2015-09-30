#include <thread>
#include <iostream>

void foo() {
	std::cout << "foo" << std::endl;
}
void bar() {
	std::cout << "bar" << std::endl;
}

int main() {
	std::thread t([] () {
			foo();
			bar();
			});
	t.join();
}
