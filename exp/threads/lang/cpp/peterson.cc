bool flag[2] = {false, false};
int turn;

void thread_f1() {
	flag[0] = true;
	turn = 1;
	while (flag[1] && turn == 1)
	{
		// busy wait
	}
	// critical section
	// ...
	// end of critical section
	flag[0] = false;
}

void thread_f2() {
	flag[1] = true;
	turn = 0;
	while (flag[0] && turn == 0)
	{
		// busy wait
	}
	// critical section
	// ...
	// end of critical section
	flag[1] = false;
}

int main() {

}
