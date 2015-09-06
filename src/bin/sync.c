#include <unistd.h>
#include <stdlib.h>

int main(int argc __unused, char *argv[] __unused) {
	sync();

	exit(0);
}
