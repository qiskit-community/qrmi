/*
 * This code is part of Qiskit.
 *
 * Copyright (C) 2025 IBM, UKRI-STFC (Hartree Centre)
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void load_dotenv() {
  FILE *fp;
  char str[1024];

  fp = fopen(".env", "r");
  if (fp == NULL) {
    return;
  }

  while (fgets(str, sizeof(str), fp) != NULL) {
    char *tok_ptr = NULL;
    char *key = strtok_r(str, "=", &tok_ptr);
    char *value = strtok_r(NULL, "\n", &tok_ptr);
    if (value == NULL) {
      continue;
    }
    setenv(key, value, 1);
  }
  fclose(fp);
}

const char *read_file(const char *filename) {
  FILE *fp = fopen(filename, "rb");
  if (!fp) {
    fprintf(stderr, "Failed to open a file.\n");
    return NULL;
  }

  fseek(fp, 0, SEEK_END);
  size_t size = ftell(fp);
  rewind(fp);

  char *buffer = (char *)malloc(size + 1);
  if (!buffer) {
    fprintf(stderr, "Failed to allocate a buffer.\n");
    fclose(fp);
    return NULL;
  }
  buffer[size] = '\0';

  size_t read_size = fread(buffer, sizeof(char), size, fp);
  fclose(fp);

  if (read_size != size) {
    return NULL;
  }

  return buffer;
}
