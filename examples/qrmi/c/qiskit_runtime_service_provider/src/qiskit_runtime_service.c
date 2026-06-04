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
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

#include "qrmi.h"

extern void load_dotenv();
extern const char *read_file(const char *);

int main(int argc, char *argv[]) {

  load_dotenv();

  QrmiResourceProvider *provider = qrmi_provider_new(QRMI_RESOURCE_TYPE_QISKIT_RUNTIME_SERVICE);
  if (provider == NULL) {
    const char *err = qrmi_get_last_error();
    printf("error: %s\n", err);
    qrmi_string_free((char *)err);
  }

  QrmiQuantumResources resources = {0};
  QrmiReturnCode rc = qrmi_provider_resources(provider, "num_qubits=127", &resources);
  if (rc == QRMI_RETURN_CODE_SUCCESS) {
    for (size_t i = 0; i < resources.length; i++) {
      char *id = NULL;
      qrmi_resource_id(resources.resources[i], &id);
      printf("resource: %s\n", id);

      bool is_accessible = false;
      rc = qrmi_resource_is_accessible(resources.resources[i], &is_accessible);
      if (rc == QRMI_RETURN_CODE_SUCCESS) {
        if (is_accessible == false) {
          fprintf(stderr, "%s cannot be accessed.\n", id);
          qrmi_string_free(id);
          goto error;
        } else {
          fprintf(stdout, "%s can be accessed.\n", id);
	}
      } else {
        const char* last_error = qrmi_get_last_error();
        fprintf(stderr, "qrmi_resource_is_accessible() failed. %s\n", last_error);
        qrmi_string_free((char *)last_error);
        qrmi_string_free(id);
        goto error;
      }
      qrmi_string_free(id);
    }
    qrmi_provider_resources_free(&resources);
  }

  qrmi_provider_free(provider);
  return EXIT_SUCCESS;

error:
  qrmi_provider_free(provider);
  return EXIT_FAILURE;
}
