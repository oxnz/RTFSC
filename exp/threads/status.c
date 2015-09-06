#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <err.h>

void *tf1(void *arg) {
	printf("thread 1 returning\n");
	return (void *)1;
}

void *tf2(void *arg) {
	printf("thread 2 returning\n");
	return (void *)2;
}

int main(void) {
	int i;
	void * (*f[2])(void*) = {tf1, tf2};
	pthread_t tid[2];
	void *ret;

	for (i = 0; i < 2; ++i) {
		if (pthread_create(&tid[i], NULL, f[i], NULL))
			err(1, "%s", "pthread_create");
	}
	for (i = 0; i < 2; ++i) {
		if (pthread_join(tid[i], &ret))
			err(1, "%s", "pthread_join");
		printf("thread %d exit code: %ld\n", i, (long)ret);
	}
	exit(0);
}
