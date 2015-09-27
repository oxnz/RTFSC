#include <iostream>
#include <thread>

void greet() {
	std::cout << "Hello Concurrent World" << std::endl;
}

int main(int argc, char *argv[]) {
	std::thread t(greet);
	t.join();
}
