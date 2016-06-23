/*
 * Filename:	T.java
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-06-15 11:39:00 CST]
 * Last-update:	2015-06-15 11:39:00 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License: 
 * Copyright (c) 2013 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

public class T {

	static int N = 10000;

	public static void f1(int[][] m) {
		for (int i = 0; i < N; ++i)
			for (int j = 0; j < N; ++j)
				m[i][j] = i*j;
	}

	public static void f2(int[][] m) {
		for (int i = 0; i < N; ++i)
			for (int j = 0; j < N; ++j)
				m[j][i] = i*j;
	}

	public static void main(String []args) {
		int[][] m = new int[N][N];
		for (int i = 0; i < 4; ++i) {
		long ts0 = System.currentTimeMillis();
		f2(m);
		long ts1 = System.currentTimeMillis();
		f1(m);
		long ts2 = System.currentTimeMillis();
		System.out.println("" + (ts1 - ts0) + " vs " + (ts2 - ts1));
		}
	}
}
