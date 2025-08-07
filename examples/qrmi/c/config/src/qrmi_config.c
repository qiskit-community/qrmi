/*
 * This code is part of Qiskit.
 *
 * (C) Copyright IBM 2025.
 *
 * This code is licensed under the Apache License, Version 2.0. You may
 * obtain a copy of this license in the LICENSE.txt file in the root directory
 * of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
 *
 * Any modifications or derivative works of this code must retain this
 * copyright notice, and modified files need to carry a notice indicating
 * that they have been altered from the originals.
 */
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

#include "qrmi.h"

int main(int argc, char *argv[]) {
  if (argc != 3) {
    fprintf(stderr, "qrmi_config <config file> <name>\n");
    return 0;
  }

  QrmiConfig *cnf = qrmi_config_load(argv[1]);
  if (!cnf) {
    fprintf(stderr, "Failed to load QRMI config file: %s.\n", argv[1]);
    return -1;
  }

  size_t num_names = 0;
  char **names = NULL;
  int rc = qrmi_config_resource_names_get(cnf, &num_names, &names);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    for (int i = 0; i < num_names; i++) {
      printf("[%s]\n", names[i]);
    }
    qrmi_string_array_free(num_names, names);
  }

  QrmiResourceDef *res = qrmi_config_resource_def_get(cnf, argv[2]);
  if (res != NULL) {
    printf("%s %d\n", res->name, res->type);
    QrmiEnvironmentVariables envvars = res->environments;
    for (int j = 0; j < envvars.length; j++) {
      QrmiKeyValue envvar = envvars.variables[j];
      printf("%s = %s\n", envvar.key, envvar.value);
    }
    qrmi_config_resource_def_free(res);
  } else {
    printf("resource %s not found.", argv[2]);
  }
  qrmi_config_free(cnf);

  return 0;
}
