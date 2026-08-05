#include <stdio.h>

int tambah(int a, int b) {
    printf("[C] Menghitung %d + %d\n", a, b);
    return a + b;
}

void sapa(const char* nama) {
    printf("[C] Halo dari C, %s!\n", nama);
}
